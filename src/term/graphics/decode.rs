use miniz_oxide::inflate::{TINFLStatus, decompress_to_vec_zlib_with_limit};

use super::{Command, Error, Format, decode_png};

pub const MAX_SIDE: u32 = 10_000;
pub const MAX_BYTES: usize = 128 << 20;

const RGB_BYTES: usize = 3;
const RGBA_BYTES: usize = 4;

pub fn decode(cmd: &Command, raw: Vec<u8>) -> Result<(u32, u32, Vec<u8>), Error> {
    let raw = if cmd.compressed { inflate(&raw)? } else { raw };
    let (width, height, pixels) = match cmd.format {
        Format::Png => decode_png(&raw).ok_or(Error::Inval)?,
        Format::Rgb => (cmd.width, cmd.height, expand(&sized(cmd, raw, RGB_BYTES)?)),
        Format::Rgba => (cmd.width, cmd.height, sized(cmd, raw, RGBA_BYTES)?),
        Format::Unsupported => return Err(Error::Inval),
    };
    check(width, height)?;
    if pixels.len() > MAX_BYTES {
        return Err(Error::TooBig);
    }
    Ok((width, height, pixels))
}

fn inflate(raw: &[u8]) -> Result<Vec<u8>, Error> {
    decompress_to_vec_zlib_with_limit(raw, MAX_BYTES).map_err(|error| match error.status {
        TINFLStatus::HasMoreOutput => Error::TooBig,
        _ => Error::Inval,
    })
}

fn check(width: u32, height: u32) -> Result<(), Error> {
    if width == 0 || height == 0 {
        return Err(Error::Inval);
    }
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(Error::TooBig);
    }
    Ok(())
}

fn sized(cmd: &Command, raw: Vec<u8>, depth: usize) -> Result<Vec<u8>, Error> {
    check(cmd.width, cmd.height)?;
    let expected = cmd.width as usize * cmd.height as usize * depth;
    if raw.len() == expected {
        Ok(raw)
    } else {
        Err(Error::Inval)
    }
}

fn expand(rgb: &[u8]) -> Vec<u8> {
    rgb.as_chunks::<RGB_BYTES>()
        .0
        .iter()
        .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], u8::MAX])
        .collect()
}
