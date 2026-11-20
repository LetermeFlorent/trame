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
  Start-Hidden $Binary ('--user-data-dir "' + "$Bench\zed-data" + '" "' + $dir + '"') $false
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
    Start-Hidden $Binary ('"' + $dir + '"') $true
  } finally {
    foreach ($n in $names) { [Environment]::SetEnvironmentVariable($n, $saved[$n]) }
  }
}

function Reset-BenchWork {
  $left = Remove-Retry @("$Bench\phase.txt", "$Bench\res.json", "$Bench\go.txt", "$Bench\appdata", "$Bench\work", "$Bench\zed-data")
  if ($left) {
    $holders = @(Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like "*$Bench*" -and $_.ProcessId -ne $PID } | ForEach-Object { "$($_.Name)/$($_.ProcessId)" }) -join ', '
    throw ('run files still locked: ' + ($left -join ', ') + ', held by: ' + $holders)
  }
  New-Item -ItemType Directory "$Bench\appdata", "$Bench\work" | Out-Null
  Copy-Item "$PSScriptRoot\modele\*", "$PSScriptRoot\inner.ps1" "$Bench\work"
}

function Enter-BenchDesktop($proc, $sw, $windowLog) {
  $hwnd = [Win]::Claim($proc.Id, $WindowTimeoutS * 1000)
  $windowMs = [int]$sw.ElapsedMilliseconds
  Add-WindowLog $windowLog $sw 'claimed clipped' $hwnd
  [void][Win]::ShowWindow($hwnd, 7)
  Add-WindowLog $windowLog $sw 'minimized' $hwnd
  Move-ToDesktop $hwnd $Desktop
  Add-WindowLog $windowLog $sw 'moved' $hwnd
  [Win]::Reveal($hwnd)
  if ([Win]::Exposed) { throw "bench window was visible on the user's desktop ($([Win]::Exposed) samples)" }
  Add-WindowLog $windowLog $sw 'shown no activate' $hwnd
  [pscustomobject]@{ hwnd = $hwnd; window_ms = $windowMs }
}

function Invoke-Run {
  Assert-BenchDesktopFree
  Reset-BenchWork
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
    $claimed = Enter-BenchDesktop $p $sw $windowLog
    $r.window_ms = $claimed.window_ms
    $r.shell_ms = Wait-Shell $p $seen $known $launch $WindowTimeoutS
    Wait-Quiet $samples $p 'settle' $seen $SettleS
    Add-WindowLog $windowLog $sw 'settled' $claimed.hwnd
    if ([Win]::IsIconic($claimed.hwnd)) { Write-Warning 'bench window is minimized' }
    if ([Win]::GetForegroundWindow() -eq $claimed.hwnd) { Write-Warning 'bench window took the focus' }
    Wait-Quiet $samples $p 'idle-before' $seen $IdleSamples
    Wait-BenchPhases $p $samples $seen @{ bench = $Bench; runTimeoutS = $RunTimeoutS; phaseStartTimeoutS = $PhaseStartTimeoutS; sampleMs = $SampleMs }
    Wait-Quiet $samples $p 'settle' $seen $SettleS
    Wait-Quiet $samples $p 'idle-after' $seen $IdleSamples
    $res = Get-Content "$Bench\res.json" -Raw | ConvertFrom-Json
    Set-RunMetrics $r $samples $res $FloodBoth $CodeTask
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
Write-BenchReport $raw $Metrics @{ target = $Target; binary = $Binary; desktop = $Desktop; json = [bool]$Json }
