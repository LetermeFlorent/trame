param(
  [ValidateSet('trame', 'zed')][string]$Target = 'trame',
  [string]$Binary,
  [int]$Runs = 3,
  [switch]$CodeTask,
  [switch]$FloodBoth,
  [int]$Desktop = 3,
  [switch]$Json,
  [switch]$AllowFocusSteal
)
$ErrorActionPreference = 'Stop'
if ($Target -eq 'zed' -and -not $AllowFocusSteal) { throw 'Zed takes the keyboard focus on every launch route, nothing started; pass -AllowFocusSteal to run it anyway' }
. "$PSScriptRoot\lib.ps1"
. "$PSScriptRoot\launch.ps1"
Import-Module VirtualDesktop -WarningAction SilentlyContinue
if ((Get-DesktopCount) -lt $Desktop) { throw "virtual desktop $Desktop does not exist" }
function Assert-BenchDesktopFree { if ((Get-DesktopIndex (Get-CurrentDesktop)) -eq $Desktop - 1) { throw "the current desktop is the bench desktop $Desktop, nothing started" } }
Assert-BenchDesktopFree
if (-not $Binary) { $Binary = if ($Target -eq 'zed') { "$env:LOCALAPPDATA\Programs\Zed\Zed.exe" } else { "$env:LOCALAPPDATA\Programs\trame\trame.exe" } }
$Binary = (Resolve-Path $Binary).Path
if ($Target -eq 'zed' -and (Get-Process Zed -ErrorAction SilentlyContinue)) { throw 'Zed is already running, nothing started' }
$FloodLines = 160000
$WindowTimeoutS = 40
$RestAfterS = 30
$SettleS = $RestAfterS + 10
$IdleSamples = 10
$SampleMs = 300
$RunTimeoutS = 600
$PhaseStartTimeoutS = 30
$Leftover = 0
$Bench = Join-Path $env:TEMP ('trame-bench-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
$Metrics = @('window_ms', 'shell_ms', 'idle_priv_mb', 'idle_ws_mb', 'flood_peak_priv_mb', 'idle_cpu_before_pct',
  'idle_cpu_after_pct', 'flood_plain_ms', 'flood_color_ms', 'flood_plain_cpu_pct', 'flood_color_cpu_pct',
  'clm_version_ms', 'clm_list_ms')
if ($FloodBoth) { $Metrics += 'flood_plain_fn_ms' }
if ($CodeTask) { $Metrics += 'code_task_ms', 'code_task_ok' }

function Start-Zed($dir) {
  $afterArgs = '"-NoLogo","-NoExit","-ExecutionPolicy","Bypass","-Command",' + (ConvertTo-Json -InputObject ([IO.File]::ReadAllText("$Bench\shim\after.txt")))
  $shell = '"shell": {"with_arguments": {"program": "powershell.exe", "args": [' + $afterArgs + ']}}'
  New-Item -ItemType Directory "$Bench\zed-data\config" | Out-Null
  Copy-Item "$Bench\zed-seed\db" "$Bench\zed-data\db" -Recurse
  (Get-Content "$env:APPDATA\Zed\settings.json" -Raw).Replace('"shell": "system"', $shell) | Set-Content "$Bench\zed-data\config\settings.json"
  Start-Hidden $Binary "--user-data-dir `"$Bench\zed-data`" `"$dir`"" $false
}

function Start-Target($dir) {
  $names = 'APPDATA', 'PATH', 'TRAME_NO_UPDATE', 'BENCH_REAL_APPDATA', 'BENCH_REAL_PATH', 'BENCH_DIR', 'BENCH_CODE_TASK', 'BENCH_FLOOD_FN'
  $saved = @{}
  foreach ($n in $names) { $saved[$n] = [Environment]::GetEnvironmentVariable($n) }
  try {
    $env:BENCH_REAL_APPDATA = $saved.APPDATA
    $env:BENCH_REAL_PATH = $saved.PATH
    $env:BENCH_DIR = $Bench
    $env:BENCH_CODE_TASK = [int][bool]$CodeTask
    $env:BENCH_FLOOD_FN = [int][bool]$FloodBoth
    if ($Target -eq 'zed') { return Start-Zed $dir }
    $env:APPDATA = "$Bench\appdata"
    $env:PATH = "$Bench\shim;" + $saved.PATH
    $env:TRAME_NO_UPDATE = '1'
    Start-Hidden $Binary "`"$dir`"" $true
  } finally {
    foreach ($n in $names) { [Environment]::SetEnvironmentVariable($n, $saved[$n]) }
  }
}

function Add-Sample($samples, $proc, $phase, $seen) {
  if ($proc.HasExited) { throw "target exited during the run, phase $phase" }
  if (-not $samples.Count -or $samples[$samples.Count - 1].phase -ne $phase) { Add-Tree $proc $seen }
  [void]$samples.Add((Get-Sample $proc $phase $seen))
}

function Wait-Quiet($samples, $proc, $phase, $seen, $seconds) {
  for ($i = 0; $i -lt $seconds; $i++) { Add-Sample $samples $proc $phase $seen; Start-Sleep -Seconds 1 }
}

function Invoke-Run {
  Assert-BenchDesktopFree
  $left = Remove-Retry @("$Bench\phase.txt", "$Bench\res.json", "$Bench\go.txt", "$Bench\appdata", "$Bench\work", "$Bench\zed-data")
  if ($left) { throw ('run files still locked: ' + ($left -join ', ') + ', held by: ' + (@(Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like "*$Bench*" -and $_.ProcessId -ne $PID } | ForEach-Object { "$($_.Name)/$($_.ProcessId)" }) -join ', ')) }
  New-Item -ItemType Directory "$Bench\appdata", "$Bench\work" | Out-Null
  Copy-Item "$PSScriptRoot\modele\*", "$PSScriptRoot\inner.ps1" "$Bench\work"
  $r = [ordered]@{}
  $samples = New-Object System.Collections.ArrayList
  $seen = @{}
  $windowLog = New-Object System.Collections.ArrayList
  $p = $null
  $known = @(Get-Process powershell -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  try {
    Add-WindowLog $windowLog ([Diagnostics.Stopwatch]::StartNew()) 'before launch' $null
    $launch = Get-Date
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Target "$Bench\work"
    $hwnd = [Win]::Claim($p.Id, $WindowTimeoutS * 1000)
    $r.window_ms = [int]$sw.ElapsedMilliseconds
    Add-WindowLog $windowLog $sw 'claimed clipped' $hwnd
    [void][Win]::ShowWindow($hwnd, 7)
    Add-WindowLog $windowLog $sw 'minimized' $hwnd
    Move-ToDesktop $hwnd $Desktop
    Add-WindowLog $windowLog $sw 'moved' $hwnd
    [Win]::Reveal($hwnd)
    if ([Win]::Exposed) { throw "bench window was visible on the user's desktop ($([Win]::Exposed) samples)" }
    Add-WindowLog $windowLog $sw 'shown no activate' $hwnd
    $r.shell_ms = Wait-Shell $p $seen $known $sw $WindowTimeoutS $launch
    Wait-Quiet $samples $p 'settle' $seen $SettleS
    Add-WindowLog $windowLog $sw 'settled' $hwnd
    if ([Win]::IsIconic($hwnd)) { Write-Warning 'bench window is minimized' }
    if ([Win]::GetForegroundWindow() -eq $hwnd) { Write-Warning 'bench window took the focus' }
    Wait-Quiet $samples $p 'idle-before' $seen $IdleSamples
    Set-Content "$Bench\go.txt" 'go'
    $run = [Diagnostics.Stopwatch]::StartNew()
    $phase = 'start'
    while ($run.Elapsed.TotalSeconds -lt $RunTimeoutS -and $phase -ne 'fin') {
      if (Test-Path "$Bench\phase.txt") {
        try { $v = [IO.File]::ReadAllText("$Bench\phase.txt").Trim(); if ($v) { $phase = $v } } catch { }
      }
      if ($phase -eq 'start' -and $run.Elapsed.TotalSeconds -gt $PhaseStartTimeoutS) { throw 'inner script never started in the terminal' }
      if ($phase -ne 'fin') { Add-Sample $samples $p $phase $seen; Start-Sleep -Milliseconds $SampleMs }
    }
    if ($phase -ne 'fin') { throw "run did not finish, last phase $phase" }
    Wait-Quiet $samples $p 'settle' $seen $SettleS
    Wait-Quiet $samples $p 'idle-after' $seen $IdleSamples
    $res = Get-Content "$Bench\res.json" -Raw | ConvertFrom-Json
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
    if ($FloodBoth) { $r.flood_plain_fn_ms = $res.flood_plain_fn_ms }
    if ($CodeTask) { $r.code_task_ms = $res.code_task_ms; $r.code_task_ok = $res.code_task_ok }
  } finally {
    if ($p) { $script:Leftover += Stop-Seen $p $seen }
  }
  $r.window_log = $windowLog
  $r
}

try {
  New-Item -ItemType Directory "$Bench\shim" | Out-Null
  & python -I "$PSScriptRoot\prep.py" $Bench $FloodLines
  Add-Type -TypeDefinition (Get-Content -Raw "$PSScriptRoot\shim.cs") -OutputAssembly "$Bench\shim\pwsh.exe" -OutputType ConsoleApplication
  Set-Content "$Bench\shim\after.txt" "while (-not (Test-Path '$Bench\go.txt')) { Start-Sleep -Milliseconds 100 }; & .\inner.ps1" -NoNewline
  if ($Target -eq 'zed') { & python -I "$PSScriptRoot\zed_seed.py" "$env:LOCALAPPDATA\Zed\db" $Bench "$Bench\work" }
  $raw = @(1..$Runs | ForEach-Object { Invoke-Run })
} finally {
  Start-Sleep -Seconds 1
  if (Remove-Retry @($Bench)) { Write-Warning "sandbox not deleted: $Bench" }
}

if ($Leftover) { Write-Error "bench processes still running after the stop: $Leftover, run failed"; exit 1 }
$medians = [ordered]@{}
foreach ($m in $Metrics) {
  $medians[$m] = Get-Median ($raw | ForEach-Object { $_[$m] })
  $missing = @($raw | Where-Object { $null -eq $_[$m] }).Count
  if ($missing) { Write-Warning "$m has no value in $missing of $Runs runs" }
}
if ($Json) {
  [ordered]@{ target = $Target; binary = $Binary; desktop = $Desktop; runs = $Runs; medians = $medians; raw = $raw } | ConvertTo-Json -Depth 5
} else {
  Write-Host "$Target window on virtual desktop $Desktop of $(Get-DesktopCount)"
  for ($i = 0; $i -lt $Runs; $i++) { Write-Host "window log run $($i + 1)"; $raw[$i].window_log | Format-Table -AutoSize | Out-String | Write-Host }
  $Metrics | ForEach-Object {
    $row = [ordered]@{ metric = $_ }
    for ($i = 0; $i -lt $Runs; $i++) { $row["run$($i + 1)"] = $raw[$i][$_] }
    $row.median = $medians[$_]
    [pscustomobject]$row
  } | Format-Table -AutoSize
}
