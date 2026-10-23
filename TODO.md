# Work in progress

Task L (installer) is finished. Only the release is left.

## Done in the last commit

The Windows installer now opens on a choice between Install for me only, the default, and Install
for all users. The first keeps the old place, `%LOCALAPPDATA%\Programs\trame`, with no prompt. The
second needs an administrator and goes to `C:\Program Files\trame`, with the Start menu and desktop
shortcuts of every account. `/CURRENTUSER` and `/ALLUSERS` pick the mode without the window.

A copy that cannot write its own folder and sits next to the uninstaller is an install for all
users: it no longer tries to swap its executable at startup. It downloads
`trame-<version>-setup.exe` into `%TEMP%\trame-update` once a newer release exists, then offers the
Update button. The click saves every tab, runs the installer with
`/SILENT /SUPPRESSMSGBOXES /NORESTART /ALLUSERS /RELAUNCH=yes` and quits. `/RELAUNCH=yes` makes the
installer open trame again with `--updated`, as the user who clicked. Both install modes were
installed under Wine, and the choice window was checked there.

## Left to do

1. Release 0.3.6, published only, never installed on the main PC.
