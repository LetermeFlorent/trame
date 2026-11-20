use std::path::{Path, PathBuf};
use std::rc::Rc;

use trame::desk::FolderTarget;
use trame::files::{PickRequest, import_icon, pick_folder, pick_image};
use trame::log_error;
use trame::workspace::InstanceId;

use super::WindowCtx;
use super::timer::refresh;
use crate::gnome::events::unbounded_listen;
use crate::gnome::state::PickerJob;

enum PickerResult {
    Folder(FolderTarget, PathBuf),
    Icon(InstanceId, u64),
}

pub(super) fn launch(ctx: &Rc<WindowCtx>, job: PickerJob) {
    let weak = Rc::downgrade(ctx);
    let tx = unbounded_listen(move |res| {
        if let Some(c) = weak.upgrade() {
            match res {
                PickerResult::Folder(t, path) => c.state.borrow_mut().folder_picked(t, path),
                PickerResult::Icon(id, img) => c.state.borrow_mut().icon_picked(id, img),
            }
            refresh(&c);
        }
        false
    });
    match job {
        PickerJob::Folder(target, start) => launch_folder(ctx, tx, target, start),
        PickerJob::Image(id, dir) => launch_image(ctx, tx, id, dir),
    }
}

fn launch_folder(
    ctx: &WindowCtx,
    tx: async_channel::Sender<PickerResult>,
    target: FolderTarget,
    start: Option<PathBuf>,
) {
    let title = ctx.state.borrow().desk.strings.pick_folder.to_owned();
    pick_folder(
        PickRequest {
            owner: None,
            title,
            start,
        },
        move |path| {
            let _ = tx.send_blocking(PickerResult::Folder(target, path));
        },
    );
}

fn launch_image(
    ctx: &WindowCtx,
    tx: async_channel::Sender<PickerResult>,
    id: InstanceId,
    dir: PathBuf,
) {
    let title = ctx.state.borrow().desk.strings.pick_image.to_owned();
    pick_image(
        PickRequest {
            owner: None,
            title,
            start: None,
        },
        move |path| {
            finish_icon_import(&tx, id, &path, &dir);
        },
    );
}

fn finish_icon_import(
    tx: &async_channel::Sender<PickerResult>,
    id: InstanceId,
    path: &Path,
    dir: &Path,
) {
    match import_icon(path, dir) {
        Ok(img) => {
            let _ = tx.send_blocking(PickerResult::Icon(id, img));
        }
        Err(e) => log_error!("the image could not become an icon: {e}"),
    }
}
