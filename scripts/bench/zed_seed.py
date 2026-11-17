import shutil
import sqlite3
import sys
from pathlib import Path

source, bench, work = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
stable = bench / "zed-seed" / "db" / "0-stable"
shutil.rmtree(bench / "zed-seed", ignore_errors=True)
for name in ("0-global", "0-stable"):
    target = bench / "zed-seed" / "db" / name
    target.mkdir(parents=True)
    for part in ("db.sqlite", "db.sqlite-wal", "db.sqlite-shm"):
        if (source / name / part).exists():
            shutil.copy2(source / name / part, target / part)
db = sqlite3.connect(stable / "db.sqlite")
db.execute("PRAGMA foreign_keys=ON")
display, identity = db.execute("select display, identity_paths_order from workspaces limit 1").fetchone()
db.execute("delete from workspaces")
db.execute(
    "insert into workspaces (workspace_id, paths, paths_order, identity_paths, identity_paths_order, window_state,"
    " window_x, window_y, window_width, window_height, display, left_dock_visible, right_dock_visible,"
    " bottom_dock_visible, bottom_dock_active_panel, left_dock_zoom, right_dock_zoom, bottom_dock_zoom,"
    " fullscreen, centered_layout) values (1, ?, '0', ?, '0', 'Windowed', 200, 150, 1300, 800, ?, 0, 0, 1,"
    " 'TerminalPanel', 0, 0, 0, 0, 0)",
    (work, work, display),
)
db.commit()
db.execute("PRAGMA journal_mode=DELETE")
db.close()
for name in ("0-global", "0-stable"):
    for part in ("db.sqlite-wal", "db.sqlite-shm"):
        (bench / "zed-seed" / "db" / name / part).unlink(missing_ok=True)
