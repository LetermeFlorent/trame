function Wait-Shell($proc, $seen, $known, $launch, $limitS) {
  while (((Get-Date) - $launch).TotalSeconds -lt $limitS) {
    if (@(Get-Process powershell -ErrorAction SilentlyContinue | Where-Object { $known -notcontains $_.Id })) {
      Add-Tree $proc $seen
      $shell = @($seen.Values | Where-Object { $_.Name -eq 'powershell.exe' } | Sort-Object CreationDate)
      if ($shell.Count) { return [int]($shell[0].CreationDate - $launch).TotalMilliseconds }
    }
    Start-Sleep -Milliseconds 10
  }
  throw 'shell timeout'
}

function Get-Sample($proc, $phase, $seen) {
  $direct = @($seen.Values | Where-Object { $_.ParentProcessId -eq $proc.Id -and $_.Name -notmatch '^(pwsh|powershell|cmd)\.exe$' })
  $ids = @($proc.Id) + @($direct | ForEach-Object { $_.ProcessId })
  $cpu = 0.0; $ws = 0.0; $priv = 0.0
  foreach ($p in @(Get-Process -Id $ids -ErrorAction SilentlyContinue)) {
    $entry = $seen[[int]$p.Id]
    if (-not $entry) { continue }
    try {
      if (-not (Test-Born $p.StartTime $entry.CreationDate)) { continue }
      $cpu += $p.TotalProcessorTime.TotalSeconds; $ws += $p.WorkingSet64 / 1MB; $priv += $p.PrivateMemorySize64 / 1MB
    } catch {
      $exited = try { $p.HasExited } catch { $true }
      if (-not $exited) { Write-Warning "bench: sample skipped on pid $($p.Id): $($_.Exception.Message)" }
    }
  }
  [pscustomobject]@{
    t = [Diagnostics.Stopwatch]::GetTimestamp() / [Diagnostics.Stopwatch]::Frequency
    phase = $phase; cpu = $cpu; ws = $ws; priv = $priv
  }
}

function Get-PhaseStat($samples, $phases) {
  $idx = @(0..($samples.Count - 1) | Where-Object { $phases -contains $samples[$_].phase })
  if (-not $idx.Count) { return [pscustomobject]@{ cpu = $null; ws = $null; priv = $null } }
  $g = @($idx | ForEach-Object { $samples[$_] })
  $from = if ($phases[0] -like 'idle*') { $g[0] } else { $samples[[math]::Max($idx[0] - 1, 0)] }
  $dt = $g[-1].t - $from.t
  [pscustomobject]@{
    cpu = if ($dt -gt 0.2) { [math]::Round(100 * ($g[-1].cpu - $from.cpu) / $dt, 1) } else { $null }
    ws = [int]($g | Measure-Object ws -Maximum).Maximum
    priv = [int]($g | Measure-Object priv -Maximum).Maximum
  }
}

function Get-Median($values) {
  $v = @($values | Where-Object { $null -ne $_ } | Sort-Object)
  if ($v.Count -eq 0) { return $null }
  $mid = [int][math]::Floor($v.Count / 2)
  if ($v.Count % 2) { $v[$mid] } else { [math]::Round(($v[$mid - 1] + $v[$mid]) / 2, 1) }
}

function Add-Sample($samples, $proc, $phase, $seen) {
  if ($proc.HasExited) { throw "target exited during the run, phase $phase" }
  if (-not $samples.Count -or $samples[$samples.Count - 1].phase -ne $phase) { Add-Tree $proc $seen }
  [void]$samples.Add((Get-Sample $proc $phase $seen))
}

function Wait-Quiet($samples, $proc, $phase, $seen, $seconds) {
  for ($i = 0; $i -lt $seconds; $i++) { Add-Sample $samples $proc $phase $seen; Start-Sleep -Seconds 1 }
}

function Wait-BenchPhases($proc, $samples, $seen, $cfg) {
  Set-Content "$($cfg.bench)\go.txt" 'go'
  $run = [Diagnostics.Stopwatch]::StartNew()
  $phase = 'start'
  while ($run.Elapsed.TotalSeconds -lt $cfg.runTimeoutS -and $phase -ne 'fin') {
    if (Test-Path "$($cfg.bench)\phase.txt") {
      try { $v = [IO.File]::ReadAllText("$($cfg.bench)\phase.txt").Trim(); if ($v) { $phase = $v } } catch [IO.IOException], [UnauthorizedAccessException] { $v = '' }
    }
    if ($phase -eq 'start' -and $run.Elapsed.TotalSeconds -gt $cfg.phaseStartTimeoutS) { throw 'inner script never started in the terminal' }
    if ($phase -ne 'fin') { Add-Sample $samples $proc $phase $seen; Start-Sleep -Milliseconds $cfg.sampleMs }
  }
  if ($phase -ne 'fin') { throw "run did not finish, last phase $phase" }
}

function Set-RunMetrics($r, $samples, $res, $floodBoth, $codeTask) {
  $idle = Get-PhaseStat $samples @('idle-before')
  $r.idle_priv_mb = $idle.priv
  $r.idle_ws_mb = $idle.ws
  $r.flood_peak_priv_mb = (Get-PhaseStat $samples @('flood_plain', 'flood_color')).priv
  $r.idle_cpu_before_pct = $idle.cpu
  $r.idle_cpu_after_pct = (Get-PhaseStat $samples @('idle-after')).cpu
  $r.flood_plain_ms = $res.flood_plain_ms
  $r.flood_color_ms = $res.flood_color_ms
  $r.flood_plain_cpu_pct = (Get-PhaseStat $samples @('flood_plain')).cpu
  $r.flood_color_cpu_pct = (Get-PhaseStat $samples @('flood_color')).cpu
  $r.clm_version_ms = $res.clm_version_ms
  $r.clm_list_ms = $res.clm_list_ms
  if ($floodBoth) { $r.flood_plain_fn_ms = $res.flood_plain_fn_ms }
  if ($codeTask) { $r.code_task_ms = $res.code_task_ms; $r.code_task_ok = $res.code_task_ok }
}

function Write-BenchReport($raw, $metrics, $info) {
  $runs = $raw.Count
  $medians = [ordered]@{}
  foreach ($m in $metrics) {
    $medians[$m] = Get-Median ($raw | ForEach-Object { $_[$m] })
    $missing = @($raw | Where-Object { $null -eq $_[$m] }).Count
    if ($missing) { Write-Warning "$m has no value in $missing of $runs runs" }
  }
  if ($info.json) {
    [ordered]@{ target = $info.target; binary = $info.binary; desktop = $info.desktop; runs = $runs; medians = $medians; raw = $raw } | ConvertTo-Json -Depth 5
    return
  }
  Write-Host "$($info.target) window on virtual desktop $($info.desktop) of $(Get-DesktopCount)"
  for ($i = 0; $i -lt $runs; $i++) { Write-Host "window log run $($i + 1)"; $raw[$i].window_log | Format-Table -AutoSize | Out-String | Write-Host }
  $metrics | ForEach-Object {
    $row = [ordered]@{ metric = $_ }
    for ($i = 0; $i -lt $runs; $i++) { $row["run$($i + 1)"] = $raw[$i][$_] }
    $row.median = $medians[$_]
    [pscustomobject]$row
  } | Format-Table -AutoSize
}
