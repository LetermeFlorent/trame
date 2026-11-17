$SameProcessS = 0.001
$StopPassesMax = 10
$Untracked = @{}
$Gone = @{}
$Unreadable = @{}

function Test-Born($a, $b) { [math]::Abs(($a - $b).TotalSeconds) -lt $SameProcessS }

function Get-Snapshot { @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name, CreationDate, CommandLine) }

$SharedPatterns = @('claude-drive-watcher', 'progress-board[\\/]server\.py', 'graphify_run\.py')

function Test-SharedLine([string]$line) { @($SharedPatterns | Where-Object { $line -match $_ }).Count -gt 0 }

function Read-CommandLine($id) { [string](Get-CimInstance Win32_Process -Filter "ProcessId=$([int]$id)" -ErrorAction SilentlyContinue).CommandLine }

function Set-Gone($row) { $script:Gone[[int]$row.ProcessId] = $row.CreationDate }

function Register-Process($row, $seen) {
  if (Test-SharedLine $row.CommandLine) { return }
  $held = $null
  try {
    $held = [Diagnostics.Process]::GetProcessById([int]$row.ProcessId)
    [void]$held.Handle
    if (-not (Test-Born $held.StartTime $row.CreationDate)) { $held.Dispose(); Set-Gone $row; return }
    $line = Read-CommandLine $row.ProcessId
    if (-not $line) { $exited = try { $held.HasExited } catch { $false }; $held.Dispose(); if ($exited) { Set-Gone $row } else { $script:Unreadable[[int]$row.ProcessId] = $row.CreationDate }; return }
    if (Test-SharedLine $line) { $held.Dispose(); return }
  } catch {
    if ($held) { $held.Dispose() }
    $ex = if ($_.Exception.InnerException -and $_.Exception -is [Management.Automation.RuntimeException]) { $_.Exception.InnerException } else { $_.Exception }
    if ($ex -is [ArgumentException] -or $ex -is [InvalidOperationException]) { Set-Gone $row }
    else { $script:Untracked[[int]$row.ProcessId] = $row.CreationDate; Write-Warning "bench: pid $($row.ProcessId) not tracked: $($_.Exception.Message)" }
    return
  }
  $seen[[int]$row.ProcessId] = [pscustomobject]@{ ProcessId = $row.ProcessId; ParentProcessId = $row.ParentProcessId; Name = $row.Name; CreationDate = $row.CreationDate; CommandLine = $line; Held = $held }
}

function Add-Tree($proc, $seen) {
  $all = Get-Snapshot
  $script:Gone = @{}
  if (-not $seen.ContainsKey($proc.Id)) {
    $root = $all | Where-Object { $_.ProcessId -eq $proc.Id } | Select-Object -First 1
    if ($root -and (Test-Born $root.CreationDate $(try { $proc.StartTime } catch { [datetime]::MinValue }))) { Register-Process $root $seen }
  }
  do {
    $before = $seen.Count + $script:Gone.Count
    $lost = @($script:Untracked.GetEnumerator() | ForEach-Object { $u = $_; $now = $all | Where-Object { $_.ProcessId -eq $u.Key }; if ($now -and (Test-Born $now.CreationDate $u.Value)) { [pscustomobject]@{ ProcessId = $u.Key; CreationDate = $u.Value } } })
    $exited = @($script:Gone.GetEnumerator() | ForEach-Object { [pscustomobject]@{ ProcessId = $_.Key; CreationDate = $_.Value } })
    foreach ($e in @($seen.Values) + $lost + $exited) {
      $kids = @($all | Where-Object { $_.ParentProcessId -eq $e.ProcessId -and $_.ProcessId -ne $e.ProcessId -and $_.CreationDate -ge $e.CreationDate -and -not $seen.ContainsKey([int]$_.ProcessId) -and -not $script:Gone.ContainsKey([int]$_.ProcessId) })
      foreach ($k in $kids) { Register-Process $k $seen }
    }
  } while ($seen.Count + $script:Gone.Count -gt $before)
}

function Stop-Entry($e) {
  try {
    $h = [Diagnostics.Process]::GetProcessById([int]$e.ProcessId)
    try {
      [void]$h.Handle
      if ($h.HasExited -or -not (Test-Born $h.StartTime $e.CreationDate)) { return $false }
      $h.Kill()
      [void]$h.WaitForExit(5000)
      return $true
    } finally { $h.Dispose() }
  } catch { return $false }
}

function Stop-Seen($proc, $seen) {
  $left = @()
  try {
    for ($i = 0; $i -lt $StopPassesMax -and -not $seen.ContainsKey($proc.Id) -and -not $proc.HasExited; $i++) {
      try { Add-Tree $proc $seen } catch { Write-Warning "bench: tree walk failed before the stop: $($_.Exception.Message)" }
      if (-not $seen.ContainsKey($proc.Id)) { Start-Sleep -Milliseconds 100 }
    }
    try { if (-not $proc.HasExited) { $proc.Kill(); [void]$proc.WaitForExit(5000) } } catch { Write-Warning "bench: root $($proc.Id) not stopped: $($_.Exception.Message)" }
    for ($pass = 0; $pass -lt $StopPassesMax; $pass++) {
      try { Add-Tree $proc $seen } catch { Write-Warning "bench: tree walk failed during the stop: $($_.Exception.Message)" }
      $killed = 0
      foreach ($e in @($seen.Values)) { if (Stop-Entry $e) { $killed++ } }
      if (-not $killed) { break }
    }
    try { Add-Tree $proc $seen } catch { Write-Warning "bench: tree walk failed before the final check: $($_.Exception.Message)" }
    foreach ($e in @($seen.Values)) {
      $alive = try { -not $e.Held.HasExited -and (Test-Born $e.Held.StartTime $e.CreationDate) } catch { $true }
      if ($alive) { $left += "$($e.Name)/$($e.ProcessId)" }
    }
    foreach ($id in @($script:Untracked.Keys)) { $left += "untracked/$id" }
    foreach ($id in @($script:Unreadable.Keys)) { $left += "unreadable/$id" }
    $script:Untracked.Clear()
    $script:Unreadable.Clear()
    if ($left.Count) { Write-Warning ('bench: processes still running: ' + ($left -join ', ')) }
  } finally {
    foreach ($e in @($seen.Values)) { if ($e.Held) { $e.Held.Dispose() } }
  }
  return $left.Count
}

function Wait-Shell($proc, $seen, $known, $sw, $limitS, $launch) {
  while ($sw.Elapsed.TotalSeconds -lt $limitS) {
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
    try {
      if (-not (Test-Born $p.StartTime $seen[$p.Id].CreationDate)) { continue }
      $cpu += $p.TotalProcessorTime.TotalSeconds; $ws += $p.WorkingSet64 / 1MB; $priv += $p.PrivateMemorySize64 / 1MB
    } catch { }
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

function Get-WindowDesktop($hwnd) {
  try { Get-DesktopIndex (Get-DesktopFromWindow $hwnd) } catch { -1 }
}

function Add-WindowLog($log, $sw, $tag, $hwnd) {
  if (-not $hwnd) { $hwnd = [IntPtr]::Zero }
  [void]$log.Add([pscustomobject]@{
    ms = [int]$sw.ElapsedMilliseconds; step = $tag; visible = [Win]::IsWindowVisible($hwnd); clipped = [Win]::Clipped($hwnd); foreground = (Get-Process -Id ([Win]::ForegroundPid()) -ErrorAction SilentlyContinue).ProcessName + '/' + [Win]::ForegroundPid()
    window_desktop = (Get-WindowDesktop $hwnd) + 1; current_desktop = (Get-DesktopIndex (Get-CurrentDesktop)) + 1
  })
}

function Move-ToDesktop($hwnd, $number) {
  $deadline = [Diagnostics.Stopwatch]::StartNew()
  while ($true) {
    try { Move-Window -Desktop (Get-Desktop ($number - 1)) -Hwnd $hwnd | Out-Null; break } catch { if ($deadline.Elapsed.TotalSeconds -gt 5) { throw } }
    Start-Sleep -Milliseconds 50
  }
  if ((Get-WindowDesktop $hwnd) -ne $number - 1) { throw "window not on desktop $number" }
}

function Remove-Retry($paths) {
  for ($i = 0; $i -lt 6 -and @($paths | Where-Object { Test-Path $_ }); $i++) {
    Remove-Item $paths -Recurse -Force -ErrorAction SilentlyContinue
    if (@($paths | Where-Object { Test-Path $_ })) { Start-Sleep -Milliseconds 500 }
  }
  @($paths | Where-Object { Test-Path $_ })
}
