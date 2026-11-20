use std::sync::Arc;

use super::StoredImage;
use crate::files::pictures::{dimensions, image_limits};
use crate::log_error;
use crate::term::dispatch::decode_base64;
use crate::term::screen::Screen;

const PREVIEW_ROWS: usize = 8;
const MAX_PREVIEW_COLS: usize = 255;

struct InlineArgs {
    inline: bool,
    width: Option<usize>,
    height: Option<usize>,
    preserve_aspect: bool,
}

impl Screen {
    pub(crate) fn inline_image(&mut self, params: &[&[u8]]) {
        let Some((args, payload)) = parse_params(params) else {
            return;
        };
        if !args.inline {
            return;
        }
        let Some(bytes) = decode_base64(payload).filter(|bytes| !bytes.is_empty()) else {
            log_error!("invalid inline image payload");
            return;
        };
        let Ok((width, height)) = dimensions(&bytes, image_limits()) else {
            log_error!("unreadable inline image");
            return;
        };
        let (rows, cols) = preview_cells(self.rows(), self.cols(), width, height, &args);
        let stored = StoredImage {
            bytes: Arc::from(bytes),
            width,
            height,
            rows: rows as u16,
            cols: cols as u16,
        };
        let id = self.insert_image(stored);
        self.place_image(id, rows, cols);
    }
}

fn parse_params<'a>(params: &'a [&'a [u8]]) -> Option<(InlineArgs, &'a [u8])> {
    let first = params.first()?.strip_prefix(b"File=")?;
    let last = if params.len() == 1 {
        first
    } else {
        *params.last()?
    };
    let colon = last.iter().position(|&byte| byte == b':');
    let (tail, payload) = match colon {
        Some(index) => (&last[..index], &last[index + 1..]),
        None => (last, &[][..]),
    };
    let mut args = InlineArgs {
        inline: false,
        width: None,
        height: None,
        preserve_aspect: true,
    };
    if params.len() == 1 {
        apply_args(&mut args, tail);
    } else {
        apply_args(&mut args, first);
        for &part in &params[1..params.len() - 1] {
            apply_args(&mut args, part);
        }
        apply_args(&mut args, tail);
    }
    Some((args, payload))
}

fn apply_args(args: &mut InlineArgs, slice: &[u8]) {
    for part in slice.split(|&byte| byte == b';') {
        let Some(eq) = part.iter().position(|&byte| byte == b'=') else {
            continue;
        };
        let (key, value) = (&part[..eq], &part[eq + 1..]);
        match key {
            b"inline" => args.inline = value == b"1",
            b"width" => args.width = parse_cells(value),
            b"height" => args.height = parse_cells(value),
            b"preserveAspectRatio" => args.preserve_aspect = value != b"0",
            _ => {}
        }
    }
}

fn parse_cells(value: &[u8]) -> Option<usize> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(value)
        .ok()?
        .parse::<usize>()
        .ok()
        .filter(|&n| n > 0)
}

fn preview_cells(
    grid_rows: usize,
    grid_cols: usize,
    width: u32,
    height: u32,
    args: &InlineArgs,
) -> (usize, usize) {
    let max_rows = grid_rows.saturating_sub(1).max(1);
    let rows = PREVIEW_ROWS
        .min(max_rows)
        .min(args.height.unwrap_or(usize::MAX))
        .max(1);
    let aspect_cols = ((rows as f64 * 2.0 * f64::from(width)) / f64::from(height)).round() as usize;
    let cols = match (args.preserve_aspect, args.width) {
        (false, Some(given)) => given,
        _ => aspect_cols.min(args.width.unwrap_or(usize::MAX)),
    };
    let cols = cols.min(grid_cols).clamp(1, MAX_PREVIEW_COLS);
    (rows, cols)
}
