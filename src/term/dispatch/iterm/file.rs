use super::args::{FileArgs, parse};
use super::cells::{Geometry, cells};
use crate::term::dispatch::decode::decode_base64;
use crate::term::graphics::{Anchor, Crop, ImageStore, MAX_SIDE, decode_png};
use crate::term::screen::Screen;

impl Screen {
    pub(super) fn iterm_file(&mut self, body: &[u8]) {
        let Some(colon) = body.iter().position(|&byte| byte == b':') else {
            return;
        };
        let args = parse(&body[..colon]);
        if args.inline {
            self.show_file(&args, &body[colon + 1..]);
        }
    }

    pub(super) fn show_file(&mut self, args: &FileArgs, encoded: &[u8]) {
        let Some((width, height, pixels)) = decode_base64(encoded)
            .and_then(|raw| decode_png(&raw))
            .filter(|&(width, height, _)| width <= MAX_SIDE && height <= MAX_SIDE)
        else {
            return;
        };
        let geometry = Geometry {
            image: (width, height),
            cell: self.cell_pixels,
            screen: (self.cols(), self.rows()),
        };
        let (cols, rows) = cells(args, geometry);
        let id = self.graphics.assign(0);
        self.graphics.insert(id, width, height, pixels);
        let anchor = Anchor {
            col: u16::try_from(self.cursor.col).unwrap_or(u16::MAX),
            image: id,
            placement: 0,
            cols,
            rows,
            crop: Crop {
                x: 0,
                y: 0,
                w: width,
                h: height,
            },
            offset: (0, 0),
            z: 0,
        };
        self.anchor_at_cursor(anchor, true);
        self.graphics
            .evict_unreferenced(ImageStore::QUOTA, [&self.primary, &self.alternate]);
        if !args.stay {
            for _ in 0..rows {
                self.index();
            }
            self.carriage_return();
        }
    }
}
