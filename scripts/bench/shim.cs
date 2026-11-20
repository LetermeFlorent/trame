using System;
using System.Diagnostics;
using System.IO;
public static class Program {
  enum Arg { Hook = 3 }
  public static int Main(string[] args) {
    string hook = args.Length > (int)Arg.Hook ? args[(int)Arg.Hook] : "";
    string afterHook = File.ReadAllText(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "after.txt"));
    string shell = Path.Combine(Environment.SystemDirectory, @"WindowsPowerShell\v1.0\powershell.exe");
    var startInfo = new ProcessStartInfo(shell, "-NoLogo -NoExit -ExecutionPolicy Bypass -Command \"" + hook + "; " + afterHook + "\"");
    startInfo.UseShellExecute = false;
    var process = Process.Start(startInfo);
    process.WaitForExit();
    return process.ExitCode;
  }
}
