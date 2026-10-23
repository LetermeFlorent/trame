# Work in progress

Task K (display settings) is finished. Next up is task L.

## Done in the last commit

The Display section of the settings has two new rows. Panel side puts the instances panel on the
left or the right: the rail, its tooltips, the border and the drag that resizes it follow. Spacing
switches between Compact, the sizes used so far, and Spacious, which makes the panel rows, the tab
bars and the terminal margins a quarter larger. Both are saved with the other preferences as
`panel_side` and `density`, and older files fall back to Left and Compact. The full test suite
passed.

## Left to do, in order

1. Installer (task L): per-user by default or for all users.
2. Release 0.3.6, published only, never installed on the main PC.
