# The trame guide

Everything trame does, how to install it on each system, and how to build it. The [README](README.md) has the short version.

## Instances, tabs and splits

An instance is a named group of shells, usually one per project. Instances sit in a side bar on the left, and each one has its own tabs and its own splits. Switching from "trame" to "dev server" brings back that instance's tabs exactly as you left them.

The side bar can be resized with the mouse or folded into a thin rail of icons. A double click on the empty part of a tab bar opens a new tab in that pane.

A tab can be dragged:

- onto the tab bar of another pane, to join it;
- onto the middle of another pane, to trade places with the tab shown there;
- onto the edge of a pane, to split it;
- onto an instance in the side bar, to move it there.

`Escape` cancels a drag before the drop.

### Names, colours, folders

A double click on a tab or an instance renames it. A right click opens a menu to rename it, change its colour, change its folder or, for an instance, its icon.

Change folder opens a small window with the current folder, which cannot be typed in. Change picks another one in the system window and Apply keeps it, while Escape, the cross or a click outside leaves everything as it was. Once applied, the shells already open in that tab or instance start again there, and new tabs of an instance open in its folder.

### SSH connections

The same menu opens an SSH connection in a new tab. The window lists the saved connections and the hosts of `~/.ssh/config`: a click on one connects, and its pencil fills the form below to change it.

The form takes an address, a user, a port and a password, all optional but the address. Connect writes the host to `~/.ssh/config` (or changes its block there), keeps the password in the Windows Credential Manager on Windows, and opens the tab. trame types the system `ssh` command (OpenSSH, part of Windows 10 and 11) and types the saved password once, into the password prompt of that user and host only (the address and user set in `~/.ssh/config` count). A key passphrase or the password of a jump host is left to you, and trame stops waiting for the prompt after a minute or at the first line that is not part of connecting. Each connection used goes first in the list, and a long list scrolls with the wheel or its bar.

### Shells and themes

Every new tab opens the default shell of the system: PowerShell 7 when it is installed, Windows PowerShell otherwise, and `$SHELL` on Linux and macOS. The light or dark theme follows the system and changes when the system does.

When a program that asked for mouse reports quits without turning them off, after a crash or a dropped SSH connection, trame turns them off as soon as the shell is back. Moving the mouse no longer prints codes like `^[[<35;10;5M` in the prompt.

The startup command, empty by default, is typed and run in every new shell. A tab that comes back with its text skips it, since it picks up where you left it, and a tab opened for an SSH connection only runs `ssh`.

## What comes back at launch

At launch, trame reopens the last layout with each shell in the folder it was in, and each tab shows again the text it held when trame closed. Only the text comes back. A program that was running in the tab is not started again, except an SSH connection, which asks for its password again.

The folder is the last one the shell reported on this computer. trame ignores a network path such as `\\server\share`, or a folder reported by another machine, for example over SSH, and keeps the last local one.

Three switches in the settings change that, all on by default. "Remember instances" off starts a single new instance. "Remember terminals" off brings the instances back with their name, colour, icon and folder but with fresh terminals, as many as a new instance gets. "Restore tab text" off brings the tabs back empty.

Only one trame at a time keeps the layout. A second one, opened while the first still runs, starts with a single new instance and saves nothing, not even its settings: its title bar says "unsaved session" after the name.

Closing the window, or an instance from its cross in the side bar, while it holds two tabs or more first asks to confirm, with the number of tabs that will close. Enter or Close confirms; Escape, Cancel, the cross or a click outside keeps everything open. The switch "Ask before closing several tabs", on by default, turns the question off.

Pasting text that holds a line break asks first, because a shell runs each line as soon as it arrives. The question shows the first three lines and how many there are in all. It does not come up when the program in the tab has turned on bracketed paste, which marks the pasted text so the program does not run it on its own. trame removes control characters from every paste, keeping tabs and line breaks, and asks as well when the clipboard held some. Enter or Paste sends the text; Escape, Cancel, the cross or a click outside drops it. The switch "Ask before pasting several lines", on by default, turns the question off.

## Settings

The Settings button at the bottom of the side bar (a gear with a tooltip when the bar is folded) opens the settings window in the middle of the screen. It has seven tabs, and Escape, the cross or a click outside closes it.

General chooses what comes back at launch and the folder new instances open in.

Terminals sets the startup command and how many terminals a new instance opens with (1 to 8, 1 by default). It also sets how many lines each tab keeps above the screen, 1,000 by default, from 100 up to 10,000; past that the oldest lines go first. Click the number to type one. Click the startup command to change it: Enter or a click elsewhere keeps it, Escape leaves it as it was, and the cross removes it.

Display changes the size of the terminal text and of the interface (80% to 150%), the language (Auto follows the system, or force French or English), the side of the panel and the spacing.

System holds efficiency mode, on by default. trame slows down after 30 seconds without activity, and as soon as its window is minimized or behind another one it also lowers the priority of the programs running in its tabs, until the window comes back to the front. The switch under it, "Hidden tabs with no activity", also on by default, does the same for a tab you cannot see once it has printed nothing for 30 seconds, even while the window is in front, and gives back the memory that tab holds. On Windows, System can also show a notification when a tab waits for an answer. The switch "Let programs copy to the clipboard", on by default, lets the program in the tab you are working in replace the clipboard, which is how vim or tmux copy over SSH. trame ignores the request when it comes from another tab, while its window is not in front, or with more than 256 KB of text. On Windows and macOS it holds the update switch described in [Updates](#updates).

Instances shows numbers instead of icons and chooses whether they follow when an instance closes or moves.

Connections lists the saved SSH connections with a pencil to change one and a cross to forget it, which also removes its block from `~/.ssh/config`. When that block holds more than trame writes there (another name, a pattern, a comment or another setting), trame asks first, and a block shared with other names or patterns only loses this one. Add opens an empty form with Save and Cancel.

Shortcuts changes the keys of every action, see [Keyboard shortcuts](#keyboard-shortcuts).

## Install

### Windows

Download `trame-<version>-setup.exe` from the [latest release](https://github.com/LetermeFlorent/trame/releases/latest) and run it. Its first window asks who trame is for. Install for me only is the default: trame goes to `%LOCALAPPDATA%\Programs\trame` and no administrator rights are needed. Install for all users puts it in `C:\Program Files\trame` for every account on the PC, after an administrator prompt. Either way it adds trame to the Start menu, and to the desktop if you tick the box. Scripts can skip the question with `/CURRENTUSER` or `/ALLUSERS`.

If you would rather not install anything, the same release has `trame.exe` on its own. It runs from any folder.

### Linux

Debian, Ubuntu and their derivatives use the `.deb`:

```sh
sudo apt install ./trame_<version>-1_amd64.deb
```

Fedora and other RPM distributions use the `.rpm`:

```sh
sudo dnf install ./trame-<version>-1.x86_64.rpm
```

Both packages put `trame` in `/usr/bin` and add it to the application menu. They are built on Ubuntu 22.04, so any distribution with glibc 2.35 or newer can run them. trame needs a Vulkan or OpenGL driver and runs on X11 as well as Wayland.

The Linux build passes the whole test suite and is opened under a virtual display on every release, but it has seen far less daily use than the Windows one. Reports are welcome in the [issues](https://github.com/LetermeFlorent/trame/issues).

### macOS

Download `trame-<version>-macos.zip`, unzip it and drag `trame.app` into Applications. One build runs on Apple Silicon and Intel Macs with macOS 12 or newer, drawing through Metal.

The app has no Apple Developer ID, so the first launch from a browser download is refused. Open it once from System Settings, Privacy & Security, Open Anyway, or clear the download flag in a terminal:

```sh
xattr -dr com.apple.quarantine /Applications/trame.app
```

The macOS build is tested and opened on every release, like the Linux one, and has seen even less daily use.

## Updates

trame looks for a newer release on GitHub when it starts, then once an hour. What happens next depends on the system and on where trame is installed.

Nothing is installed before it is checked. Each release carries `SHA512SUMS`, the checksums of its files, and `SHA512SUMS.sig`, a signature made with the project's release key. trame holds the public half of that key, the one in `packaging/release-key.der`, and throws away a download whose checksum is missing from the signed list or differs from it. To check a file you downloaded yourself, put it next to those two and run `openssl pkeyutl -verify -rawin -pubin -keyform DER -inkey release-key.der -in SHA512SUMS -sigfile SHA512SUMS.sig`, then `sha512sum -c --ignore-missing SHA512SUMS`.

### Windows and macOS

"Update automatically", in the System tab of the settings and on by default, installs the new version in the background while you work. trame downloads it at low priority, straight to disk, and swaps its own program without stopping anything: `trame.exe` on Windows, the whole `trame.app` on macOS. Your shells keep running, and the next time you open trame it is the new version, with no update screen. Meanwhile a Restart into button shows at the bottom of the side bar with the new version number, for when you want it right away. Clicking it asks first, and warns that tabs and their text come back but commands that were running stop.

With the switch off, nothing is installed on its own. The button reads Update to and the version instead. Clicking it asks "Update to 0.4.0?" the same way, then Restart opens trame on an update screen: a bar fills with the download percentage, trame swaps its program and reopens every instance and tab with its text, exactly as you left them, even when the launch settings say to forget them. Until the screen fades out, only the window itself can be moved, resized or closed. Cancel leaves everything as it was.

On macOS this needs a `trame.app` in a folder you can write to, which `/Applications` is for an administrator account. A copy opened straight from the Downloads folder, still flagged by the browser, runs from a read-only place and does not update itself.

A Windows copy installed for all users cannot rewrite its own folder either, so it goes through the installer. It downloads the new `setup.exe` in the background at low priority and shows the Update button. Clicking it closes trame, asks for administrator rights, installs the new version with a small progress window and opens trame again with your tabs. If you refuse the prompt, nothing changes and trame stays closed until you open it.

### Linux

The `.deb` and the `.rpm` belong to the system's package manager, so installing a new one takes your password. trame downloads the matching package in the background into `~/.cache/trame/update` and shows the Update button. Clicking it asks first, saves every tab and closes trame. The system's password window opens (through `pkexec`), the package is copied into a folder only the administrator can write to and checked once more, the package manager installs that copy (`apt-get` on Debian and Ubuntu, `dnf`, `zypper` or `rpm` elsewhere), and trame opens again with your tabs. If you cancel the password window, trame opens again on the version you had.

A copy that is not the packaged `/usr/bin/trame`, such as one built from source, leaves updates to you.

### Turning it off

Set the environment variable `TRAME_NO_UPDATE` to any value and trame never looks for updates. Debug builds never update themselves.

## Keyboard shortcuts

These are the Windows and Linux bindings, as they come out of the box.

| Action | Keys |
| --- | --- |
| New tab | `Ctrl+Shift+T` |
| Close tab | `Ctrl+Shift+W` |
| New instance | `Ctrl+Shift+N` |
| Split right | `Ctrl+Shift+D` |
| Split down | `Ctrl+Shift+Alt+D` |
| Show or hide the side bar | `Ctrl+Shift+B` |
| Copy | `Ctrl+Shift+C`, and `Ctrl+C` on Windows |
| Paste | `Ctrl+Shift+V` or `Shift+Insert`, and `Ctrl+V` on Windows |
| Next and previous tab | `Ctrl+Tab` and `Ctrl+Shift+Tab`, or `Ctrl+PageDown` and `Ctrl+PageUp` |
| Next and previous instance | `Ctrl+Shift+PageDown` and `Ctrl+Shift+PageUp` |
| Go to tab 1 to 8 | `Alt+1` to `Alt+8` |
| Go to the last tab | `Alt+9` |
| Move to the pane on that side | `Ctrl+Alt+Arrow` |
| Bigger, smaller, reset text | `Ctrl+=`, `Ctrl+-`, `Ctrl+0` |
| Scroll one page | `Shift+PageUp` and `Shift+PageDown` |
| Scroll to the top or the bottom | `Ctrl+Shift+Home` and `Ctrl+Shift+End` |

On Windows, `Ctrl+C` copies only when text is selected. Without a selection it goes to the program as usual and stops it.

The Shortcuts tab of the settings lists every action with its keys. Click an action, or its pencil, to see its keys one per line: the cross removes one, "Add a key" waits for the next combination you press, and "Default" brings back the original keys. An action keeps up to four keys. A combination already used by another action is refused with the name of that action, and so is a key the terminal needs for typing, like a letter or `PageUp` with no `Ctrl` or `Alt`. `Escape` stops waiting. "Reset all", above the list, puts every action back. Changed keys are saved with the other settings, under `shortcuts` in `session.json`.

## Where trame keeps your layout

The layout of instances, tabs and splits, along with the window size, is saved once a second while it changes. The file sits in a different place on each system:

- Windows: `%APPDATA%\trame\session.json`
- Linux: `$XDG_STATE_HOME/trame/session.json`, which is `~/.local/state/trame/session.json` by default
- macOS: `~/Library/Application Support/trame/session.json`

The settings live in the same file. Deleting it makes trame start over with a single instance. A file trame cannot read is renamed `session.json.bad` and trame starts over. If even the rename fails, trame leaves the file as it is and saves nothing until the next launch, and its title bar says "unsaved session".

The text of each tab goes into the `text` folder next to it, once a minute when something changed and once more when trame closes. Each tab keeps its last 4 MB. Turning off "Restore tab text", or one of the two switches above it, deletes that folder.

On Linux and macOS, trame keeps its folder closed to the other accounts of the machine, and a `~/.ssh` folder or config file it has to create is readable by you only.

## How it is made

The terminal itself, from the escape sequence parser to the screen grid and the scrollback, is written from scratch in Rust. Drawing goes through the graphics card with [wgpu](https://wgpu.rs): DirectX 12 on Windows, Metal on macOS and Vulkan on Linux, with Vulkan on Windows and OpenGL on Linux when those are missing. Nothing is drawn while the window is hidden or minimized.

On Windows, `trame.exe` weighs under 7 MB. With a couple of tabs open it holds around 95 MB of memory, and on a PC with integrated graphics about 60 MB of that belongs to the graphics driver.

## Build from source

trame needs Rust 1.89 or newer.

```sh
git clone https://github.com/LetermeFlorent/trame.git
cd trame
cargo build --release
```

The program lands in `target/release/trame` (`trame.exe` on Windows). `cargo test` runs the test suite, and `scripts/install-hooks` sets up the pre-commit hook that runs `cargo fmt` and `cargo clippy`.

To debug what a program sends to the terminal, build trame with `cargo build --features dump` and set the environment variable `TRAME_DUMP` to a folder. Every session then writes the raw bytes it receives from its shell to `session-<number>-output.bin` in that folder, which is created if it is missing, and each file stops at 64 MB. These files hold everything the shell printed, so delete them once read. Release builds leave the recorder out, and the shells trame starts never see the variable.

The packages are made with the same tools the release workflow uses:

```sh
cargo deb --no-build
cargo generate-rpm
iscc /DAppVersion=<version> packaging\windows\trame.iss
sh packaging/macos/bundle.sh <binary> <version> <folder>
```

The first two need [cargo-deb](https://github.com/kornelski/cargo-deb) and [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm). The third needs [Inno Setup 6](https://jrsoftware.org/isinfo.php) and writes the installer to `target/installer`. The last one runs on a Mac and writes `trame.app` into the folder.

### Releasing

Raise `version` in `Cargo.toml`, commit, and push a tag with the same number, such as `v0.4.0`. The release workflow then tests and builds trame on Windows, Ubuntu and macOS. It installs the `.deb` and opens trame, installs the `.rpm` on Fedora, wraps the macOS build into `trame.app` and opens it. Then it writes `SHA512SUMS`, signs it with the repository secret `TRAME_SIGNING_KEY`, checks the signature against `packaging/release-key.der` and publishes everything as a GitHub release. Without the secret, or with a key that does not match, nothing is published. Last, on Ubuntu and macOS, a copy numbered 0.0.1 checks that it fetches this release on its own: the package lands in the update folder and installs, and the Mac copy replaces its `trame.app`. Installed copies pick the release up within the hour or at their next launch.
