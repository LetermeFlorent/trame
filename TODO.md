# Work in progress

Task R (memory) is finished. Next up is task K.

## Done in the last commit

Restored tabs no longer start their shell at launch: each one waits, labelled with its folder, until
it is first shown in a pane. Seven restored tabs now start one shell instead of seven. Clearing the
scrollback, leaving the alternate screen and handling a long OSC string give their memory back. The
glyph atlas goes back to its starting size when it is cleared, and the quad buffers shrink after a
run of small frames. The SSH secret is wiped once both prompts got it or its minute runs out, and
the bytes sent to the shell are zeroed after writing. The full test suite passed.

## Left to do, in order

1. Display settings (task K): panel side left or right, Compact or Spacious style.
2. Installer (task L): per-user by default or for all users.
3. Release 0.3.6, published only, never installed on the main PC.
