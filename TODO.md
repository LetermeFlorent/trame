# Work in progress

Task I (custom colours and icons) is finished. Next up is task R.

## Done in the last commit

Instances take a custom icon: one or two letters typed under the presets, or a PNG chosen with the
Image button. The picture is fitted into 64x64, saved as `<id>.png` in the `icons` folder next to
the session (`%APPDATA%\trame\icons\` on Windows), loaded on first paint by `IconStore` and drawn
from the colour atlas. The file window moved from `src/files/folder` to `src/files/picker` and now
picks folders or images. The full test suite passed.

## Left to do, in order

1. Memory (task R): start restored tabs only when first shown, shrink the scrollback, alt screen
   and OSC buffers, let the atlas and quad buffers shrink, drop the SSH secret after use.
2. Display settings (task K): panel side left or right, Compact or Spacious style.
3. Installer (task L): per-user by default or for all users.
4. Release 0.3.6, published only, never installed on the main PC.
