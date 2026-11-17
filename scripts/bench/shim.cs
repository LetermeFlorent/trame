using System;
using System.Diagnostics;
using System.IO;
public static class P {
  public static int Main(string[] a) {
    string hook = a.Length > 3 ? a[3] : "";
    string afterHook = File.ReadAllText(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "after.txt"));
    string shell = Path.Combine(Environment.SystemDirectory, @"WindowsPowerShell\v1.0\powershell.exe");
    var psi = new ProcessStartInfo(shell, "-NoLogo -NoExit -ExecutionPolicy Bypass -Command \"" + hook + "; " + afterHook + "\"");
    psi.UseShellExecute = false;
    var p = Process.Start(psi);
    p.WaitForExit();
    return p.ExitCode;
  }
}
