$env:APPDATA = $env:BENCH_REAL_APPDATA
$env:PATH = $env:BENCH_REAL_PATH
$Out = $env:BENCH_DIR
$r = [ordered]@{}
function Phase($n) { [IO.File]::WriteAllText("$Out\phase.txt", $n) }
Phase 'flood_plain'
$s = [Diagnostics.Stopwatch]::StartNew()
cmd /c type "$Out\flood-plain.txt"
$r.flood_plain_ms = [int]$s.ElapsedMilliseconds
Phase 'flood_color'
$s = [Diagnostics.Stopwatch]::StartNew()
cmd /c type "$Out\flood-color.txt"
$r.flood_color_ms = [int]$s.ElapsedMilliseconds
Phase 'clm_version'
$s = [Diagnostics.Stopwatch]::StartNew()
clm --version
$r.clm_version_ms = [int]$s.ElapsedMilliseconds
Phase 'clm_list'
$s = [Diagnostics.Stopwatch]::StartNew()
clm --list
$r.clm_list_ms = [int]$s.ElapsedMilliseconds
if ($env:BENCH_FLOOD_FN -eq '1') {
  function Timed($key, [scriptblock]$block) {
    Phase $key
    $s = [Diagnostics.Stopwatch]::StartNew()
    & $block
    $r["${key}_ms"] = [int]$s.ElapsedMilliseconds
  }
  Timed 'flood_plain_fn' { cmd /c type "$Out\flood-plain.txt" }
}
if ($env:BENCH_CODE_TASK -eq '1') {
  Phase 'code_task'
  $work = (Get-Location).Path
  $s = [Diagnostics.Stopwatch]::StartNew()
  Push-Location $env:USERPROFILE
  $env:DISABLE_AUTOUPDATER = '1'
  try { clm -2 --no-update -p "Fix the bug in $work\calc.py so that running python $work\test_calc.py prints ok. Edit $work\calc.py only, do not touch the test. Reply with one word." --model haiku --permission-mode acceptEdits --add-dir $work } finally { Pop-Location }
  $r.code_task_ms = [int]$s.ElapsedMilliseconds
  $r.code_task_ok = [int](((python "$work\test_calc.py") -join '') -eq 'ok')
}
$r | ConvertTo-Json | Set-Content "$Out\res.json"
Phase 'fin'
