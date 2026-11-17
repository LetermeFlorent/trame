Add-Type @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
public static class Win {
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct SI { public int cb; public string reserved, desktop, title; public int x, y, w, h, cw, ch, fill, flags; public short show, cb2; public IntPtr res2, hin, hout, herr; }
  [StructLayout(LayoutKind.Sequential)]
  struct PI { public IntPtr hp, ht; public int pid, tid; }
  [StructLayout(LayoutKind.Sequential)]
  struct SIX { public SI si; public IntPtr attrs; }
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern bool CreateProcessW(string app, StringBuilder cmd, IntPtr pa, IntPtr ta, bool inherit, uint flags, IntPtr env, string cwd, ref SIX si, out PI pi);
  [DllImport("kernel32.dll", SetLastError = true)] static extern bool InitializeProcThreadAttributeList(IntPtr list, int count, int flags, ref IntPtr size);
  [DllImport("kernel32.dll", SetLastError = true)] static extern bool UpdateProcThreadAttribute(IntPtr list, uint flags, IntPtr attr, IntPtr value, IntPtr size, IntPtr prev, IntPtr ret);
  [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr OpenProcess(uint access, bool inherit, int pid);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr h, uint cmd);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RC r);
  [DllImport("user32.dll")] static extern int GetWindowLongW(IntPtr h, int i);
  [DllImport("user32.dll")] static extern int SetWindowLongW(IntPtr h, int i, int v);
  [DllImport("user32.dll")] static extern int SetWindowRgn(IntPtr h, IntPtr rgn, bool redraw);
  [DllImport("user32.dll")] static extern int GetWindowRgn(IntPtr h, IntPtr rgn);
  [DllImport("gdi32.dll")] static extern IntPtr CreateRectRgn(int l, int t, int r, int b);
  [StructLayout(LayoutKind.Sequential)]
  struct RC { public int l, t, r, b; }
  [StructLayout(LayoutKind.Sequential)]
  struct PT { public int x, y; }
  [StructLayout(LayoutKind.Sequential)]
  struct WP { public int length, flags, showCmd; public PT min, max; public RC normal; }
  [DllImport("user32.dll")] static extern bool GetWindowPlacement(IntPtr h, ref WP p);
  [DllImport("user32.dll")] static extern bool SetWindowPlacement(IntPtr h, ref WP p);
  delegate bool EnumProc(IntPtr h, IntPtr p);
  public static IntPtr Find(int pid) {
    IntPtr best = IntPtr.Zero;
    long area = 0;
    EnumWindows((h, p) => {
      uint owner;
      GetWindowThreadProcessId(h, out owner);
      if (owner != pid || GetWindow(h, 4) != IntPtr.Zero) return true;
      if ((GetWindowLongW(h, -20) & 0x80) != 0) return true;
      RC r;
      GetWindowRect(h, out r);
      long a = (long)(r.r - r.l) * (r.b - r.t);
      if (a > area) { area = a; best = h; }
      return true;
    }, IntPtr.Zero);
    return best;
  }
  const int ExStyle = -20, ExNoActivate = 0x08000000;
  static volatile bool watching;
  static System.Threading.Thread watcher;
  public static volatile int Exposed;
  public static int ForegroundPid() { uint pid; GetWindowThreadProcessId(GetForegroundWindow(), out pid); return (int)pid; }
  static IntPtr probeRgn = CreateRectRgn(0, 0, 1, 1);
  public static bool Clipped(IntPtr h) { return GetWindowRgn(h, probeRgn) == NullRegion; }
  const int NullRegion = 1, AwayPx = 20000, RevealPx = 100;
  public static IntPtr Claim(int pid, int timeoutMs) {
    var clock = System.Diagnostics.Stopwatch.StartNew();
    IntPtr h = IntPtr.Zero;
    while (h == IntPtr.Zero) {
      if (clock.ElapsedMilliseconds > timeoutMs) throw new TimeoutException("window timeout");
      h = Find(pid);
      if (h == IntPtr.Zero) System.Threading.Thread.Sleep(1);
    }
    RC now;
    bool shown = IsWindowVisible(h) && !IsIconic(h) && GetWindowRect(h, out now) && now.l < AwayPx / 2;
    SetWindowLongW(h, ExStyle, GetWindowLongW(h, ExStyle) | ExNoActivate);
    SetWindowRgn(h, CreateRectRgn(0, 0, 0, 0), false);
    Exposed = shown ? 1 : 0;
    watching = true;
    watcher = new System.Threading.Thread(() => {
      while (watching) {
        RC w;
        if (IsWindowVisible(h) && !IsIconic(h) && GetWindowRect(h, out w) && w.l < AwayPx / 2 && GetWindowRgn(h, probeRgn) != NullRegion) Exposed++;
        System.Threading.Thread.Sleep(1);
      }
    });
    watcher.IsBackground = true;
    watcher.Start();
    return h;
  }
  public static void Reveal(IntPtr h) {
    watching = false;
    watcher.Join();
    SetWindowRgn(h, IntPtr.Zero, true);
    var wp = new WP();
    wp.length = Marshal.SizeOf(typeof(WP));
    if (GetWindowPlacement(h, ref wp) && wp.normal.l >= AwayPx / 2) {
      wp.normal.r += RevealPx - wp.normal.l;
      wp.normal.l = RevealPx;
      wp.normal.b += RevealPx - wp.normal.t;
      wp.normal.t = RevealPx;
      SetWindowPlacement(h, ref wp);
    }
    ShowWindow(h, 4);
  }
  public static Process Launch(string exe, string args, int parentPid, bool offScreen) {
    var six = new SIX();
    six.si.cb = Marshal.SizeOf(typeof(SIX));
    six.si.flags = offScreen ? 1 | 4 : 1;
    six.si.x = AwayPx;
    six.si.y = RevealPx;
    six.si.show = 7;
    IntPtr size = IntPtr.Zero;
    InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
    six.attrs = Marshal.AllocHGlobal(size);
    IntPtr parent = OpenProcess(0x80, false, parentPid);
    if (parent == IntPtr.Zero) throw new Win32Exception();
    IntPtr slot = Marshal.AllocHGlobal(IntPtr.Size);
    Marshal.WriteIntPtr(slot, parent);
    var env = new StringBuilder();
    var vars = Environment.GetEnvironmentVariables();
    foreach (string k in vars.Keys) env.Append(k).Append('=').Append((string)vars[k]).Append((char)0);
    env.Append((char)0);
    IntPtr block = Marshal.StringToHGlobalUni(env.ToString());
    PI pi;
    try {
      if (!InitializeProcThreadAttributeList(six.attrs, 1, 0, ref size)) throw new Win32Exception();
      if (!UpdateProcThreadAttribute(six.attrs, 0, new IntPtr(0x20000), slot, new IntPtr(IntPtr.Size), IntPtr.Zero, IntPtr.Zero)) throw new Win32Exception();
      if (!CreateProcessW(exe, new StringBuilder("\"" + exe + "\" " + args), IntPtr.Zero, IntPtr.Zero, false, 0x80000 | 0x400, block, null, ref six, out pi)) throw new Win32Exception();
    } finally {
      CloseHandle(parent);
      Marshal.FreeHGlobal(slot);
      Marshal.FreeHGlobal(block);
      Marshal.FreeHGlobal(six.attrs);
    }
    Process proc;
    try {
      proc = Process.GetProcessById(pi.pid);
      if (proc.Handle == IntPtr.Zero) throw new Win32Exception();
      if (proc.HasExited) throw new InvalidOperationException("target exited at launch");
    } finally {
      CloseHandle(pi.hp);
      CloseHandle(pi.ht);
    }
    return proc;
  }
}
'@

function Start-Hidden($exe, $argLine, [bool]$OffScreen) {
  $parent = Get-Process sihost -ErrorAction SilentlyContinue | Where-Object { $_.SessionId -eq (Get-Process -Id $PID).SessionId } | Select-Object -First 1
  if (-not $parent) { throw 'sihost.exe is not running, no neutral parent process for the bench window' }
  [Win]::Launch($exe, $argLine, $parent.Id, $OffScreen)
}
