<p align="center">
  <img src="assets/logo-256.png" width="112" alt="trame logo">
</p>

<h1 align="center">trame</h1>

<p align="center">
  A light terminal for Windows and Linux that keeps your shells in instances, tabs and splits.
</p>

<p align="center">
  <a href="https://github.com/LetermeFlorent/trame/releases/latest">Download</a>
  ·
  <a href="#keyboard-shortcuts">Shortcuts</a>
  ·
  <a href="#build-from-source">Build from source</a>
</p>

## What it does

An instance is a named group of shells, usually one per project. Instances sit in a side bar on the left, and each one has its own tabs and its own splits, the way an editor like Zed lays out panes. Switching from "trame" to "dev server" brings back that instance's tabs exactly as you left them.

The rest stays out of your way:

- a tab can be dragged into another pane, onto the edge of a pane to split it, or onto an instance in the side bar;
- the side bar can be resized with the mouse or folded into a thin rail of icons;
- a double click on a tab or an instance renames it, and a right click opens a menu to rename it, change its colour, change its folder or, for an instance, its icon. The folder is picked in the system window; the shells already open in that tab or instance start again there, and new tabs of an instance open in its folder;
- the same menu opens an SSH connection in a new tab. The window lists the saved connections and the hosts of `~/.ssh/config`: a click on one connects, and its pencil fills the form below to change it. The form takes an address, a user, a port and a password, all optional but the address; Connect writes the host to `~/.ssh/config` (or changes its block there), keeps the password in the Windows Credential Manager and opens the tab. trame types the system `ssh` command (OpenSSH, part of Windows 10 and 11) and answers the password prompt with the saved password. Each connection used goes first in the list, and a long list scrolls with the wheel or its bar;
- a double click on the empty part of a tab bar opens a new tab in that pane;
- the light or dark theme follows the system and changes when the system does;
- the default shell of the system opens in every new tab: PowerShell 7 when it is installed, Windows PowerShell otherwise, and `$SHELL` on Linux;
- the Settings button at the bottom of the side bar (a gear with a tooltip when the bar is folded) opens the settings window in the middle of the screen, where you can choose what comes back at launch, set a startup command, choose how many terminals a new instance opens with (1 to 8, 1 by default), change the size of the terminal text and of the interface (80% to 150%), pick the language (Auto follows the language of Windows, or force French or English), choose the folder new instances open in. Its Connections tab lists the same SSH connections with a pencil to change one and a cross to forget it, which also removes its block from `~/.ssh/config`, and Add opens an empty form with Save and Cancel. Escape, the cross or a click outside closes the window;
- at launch, trame reopens the last layout with each shell in the folder it was in, and each tab shows again the text it held when trame closed. Only the text comes back: a program that was running in the tab is not started again, except an SSH connection, which asks for its password again. Three switches in the settings change that, all on by default: "Remember instances" off starts a single new instance, "Remember terminals" off brings the instances back with their name, colour, icon and folder but with fresh terminals, as many as a new instance gets, and "Restore tab text" off brings the tabs back empty;
- the startup command, empty by default, is typed and run in every shell trame opens, new tabs and tabs brought back at launch alike. A tab opened for an SSH connection only runs `ssh`. Click the command to change it, Enter or a click elsewhere keeps it, Escape leaves it as it was, and the cross removes it.

Drawing goes through the graphics card with [wgpu](https://wgpu.rs), on Vulkan first, with DirectX 12 on Windows and OpenGL on Linux when Vulkan is missing. Nothing is drawn while the window is hidden or minimized. The terminal itself, from the escape sequence parser to the screen grid and the scrollback, is written from scratch in Rust. On Windows 11 the program weighs 6.4 MB and uses around 65 MB of memory once open.

## Install

### Windows

Download `trame-<version>-setup.exe` from the [latest release](https://github.com/LetermeFlorent/trame/releases/latest) and run it. It installs trame for your account only, in `%LOCALAPPDATA%\Programs\trame`, so it asks for no administrator rights. It adds trame to the Start menu, and to the desktop if you tick the box.

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

## Updates

On Windows, trame looks for a newer release on GitHub every time it starts. The window opens right away on an update screen while your shells start behind it, and the check gives up after one second, so a slow or missing connection never holds you back. When a newer version exists, a bar under the message fills with the download percentage, then trame swaps its own executable and restarts. Your layout and the text of your tabs come back as they were. Until the screen fades out, only the window itself can be moved, resized or closed. A release that comes out while trame is open shows an Update button at the bottom of the side bar: clicking it restarts on the new version and reopens every instance and tab with its text, exactly as you left them, even when the launch settings say to forget them.

To turn this off, set the environment variable `TRAME_NO_UPDATE` to any value. Debug builds never update themselves.

On Linux, updates come from the package: download the new `.deb` or `.rpm` and install it over the old one.

## Keyboard shortcuts

These are the Windows and Linux bindings.

| Action | Keys |
| --- | --- |
| New tab | `Ctrl+Shift+T` |
| Close tab | `Ctrl+Shift+W` |
| New instance | `Ctrl+Shift+N` |
| Split right | `Ctrl+Shift+D` |
| Split down | `Ctrl+Shift+Alt+D` |
| Show or hide the side bar | `Ctrl+Shift+B` |
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` or `Shift+Insert` |
| Next and previous tab | `Ctrl+Tab` and `Ctrl+Shift+Tab`, or `Ctrl+PageDown` and `Ctrl+PageUp` |
| Next and previous instance | `Ctrl+Shift+PageDown` and `Ctrl+Shift+PageUp` |
| Go to tab 1 to 8 | `Alt+1` to `Alt+8` |
| Go to the last tab | `Alt+9` |
| Move to the pane on that side | `Ctrl+Alt+Arrow` |
| Bigger, smaller, reset text | `Ctrl+=`, `Ctrl+-`, `Ctrl+0` |
| Scroll one page | `Shift+PageUp` and `Shift+PageDown` |
| Scroll to the top or the bottom | `Ctrl+Shift+Home` and `Ctrl+Shift+End` |

`Escape` cancels a tab drag before the drop.

## Where trame keeps your layout

The layout of instances, tabs and splits, along with the window size, is saved once a second while it changes:

- Windows: `%APPDATA%\trame\session.json`
- Linux: `$XDG_STATE_HOME/trame/session.json`, which is `~/.local/state/trame/session.json` by default

The settings live in the same file. Deleting it makes trame start over with a single instance.

The text of each tab goes into the `text` folder next to it, `%APPDATA%\trame\text` or `~/.local/state/trame/text`, once a minute when something changed and once more when trame closes. Each tab keeps its last 4 MB. Turning off "Restore tab text", or one of the two switches above it, deletes that folder.

## Build from source

trame needs Rust 1.87 or newer.

```sh
git clone https://github.com/LetermeFlorent/trame.git
cd trame
cargo build --release
```

The program lands in `target/release/trame` (`trame.exe` on Windows). `cargo test` runs the test suite, and `scripts/install-hooks` sets up the pre-commit hook that runs `cargo fmt` and `cargo clippy`.

The packages are made with the same tools the release workflow uses:

```sh
cargo deb --no-build
cargo generate-rpm
iscc /DAppVersion=0.2.2 packaging\windows\trame.iss
```

The first two need [cargo-deb](https://github.com/kornelski/cargo-deb) and [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm). The last one needs [Inno Setup 6](https://jrsoftware.org/isinfo.php) and writes the installer to `target/installer`.

### Releasing

Raise `version` in `Cargo.toml`, commit, and push a tag with the same number, such as `v0.2.1`. The release workflow then tests and builds trame on Windows and Ubuntu, installs the `.deb` and opens trame, installs the `.rpm` on Fedora, and publishes all of it as a GitHub release. Every installed copy on Windows picks it up at its next launch.

## License

MIT, see [LICENSE](LICENSE).
