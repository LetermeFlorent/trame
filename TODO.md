# Work in progress

Stopped on 2026-10-23 in the middle of task I (custom colours and icons).

## Done in the last commit

The custom colour model is in place: `Swatch::Custom(u32)` stored as `#RRGGBB`, HSV conversion in
`src/theme/hsv.rs`, a shared list `Preferences::custom_colors` (8 at most) in
`src/preferences/colors.rs`, and `Workspace::replace_color`. The editor popups moved to
`src/ui/frame/editor/popups/`, the host list takes plain labels (`ListSpec`), and buttons-only
forms share `button_form`. The full test suite was not run before this commit, only clippy.

## Left to do, in order

1. Colour popup: under the preset row, a "My colours" list (dot, hex code, pencil, cross) built
   with `ListSpec { pickable: true, forgets: true }`, plus an Add button (`FormAction::Add`).
2. Picker in `popups/picker.rs`: saturation and value square (hue fill, white columns, black rows),
   hue bar in thin strips, `#RRGGBB` field, preview, Cancel and Apply. `Hit::Picker(part)` with a
   drag, the hex field typed through `with_field`, Apply saves the custom colour, swaps it in the
   marks when edited, applies it to the target and closes.
3. Custom icons: 1 or 2 letters, or a PNG resized to 64x64 in `%APPDATA%\trame\icons\`, loaded
   lazily into the colour atlas.
4. Memory (task R): start restored tabs only when first shown, shrink the scrollback, alt screen
   and OSC buffers, let the atlas and quad buffers shrink, drop the SSH secret after use.
5. Display settings (task K): panel side left or right, Compact or Spacious style.
6. Installer (task L): per-user by default or for all users.
7. Release 0.3.6, published only, never installed on the main PC.
