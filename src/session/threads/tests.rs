use std::fs::File;
use std::io::{PipeReader, PipeWriter, Write, pipe};

use super::{BACKLOG, read_batch};
use crate::session::pty;

#[cfg(windows)]
fn file(reader: PipeReader) -> File {
    File::from(std::os::windows::io::OwnedHandle::from(reader))
}

#[cfg(unix)]
fn file(reader: PipeReader) -> File {
    File::from(std::os::fd::OwnedFd::from(reader))
}

fn written(bytes: &[u8]) -> (File, PipeWriter) {
    let (reader, mut writer) = pipe().expect("a pipe");
    writer.write_all(bytes).expect("room in the pipe");
    (file(reader), writer)
}

fn batch(source: &File, buffer: &mut [u8], writer: &mut PipeWriter, late: &[u8]) -> Vec<u8> {
    let mut late = Some(late);
    let count = read_batch(source, buffer, || {
        if let Some(bytes) = late.take() {
            writer.write_all(bytes).expect("room in the pipe");
        }
        pty::waiting_bytes(source)
    });
    buffer[..count.expect("bytes to read")].to_vec()
}

fn long_then(tail: &[u8]) -> Vec<u8> {
    [&[b'a'; BACKLOG][..], tail].concat()
}

#[test]
fn a_long_read_takes_what_arrives_while_it_reads() {
    let (source, mut writer) = written(&long_then(b""));
    let mut buffer = vec![0; 2 * BACKLOG];
    let read = batch(&source, &mut buffer, &mut writer, b"def");
    assert_eq!(read, long_then(b"def"));
}

#[test]
fn a_short_read_is_fed_without_asking_what_waits() {
    let (source, mut writer) = written(b"abc");
    let mut buffer = vec![0; 2 * BACKLOG];
    let read = batch(&source, &mut buffer, &mut writer, b"def");
    assert_eq!((read, pty::waiting_bytes(&source)), (b"abc".to_vec(), 0));
}

#[test]
fn a_batch_stops_at_the_end_of_its_buffer() {
    let (source, mut writer) = written(&long_then(b""));
    let mut buffer = vec![0; BACKLOG + 2];
    let first = batch(&source, &mut buffer, &mut writer, b"bcd");
    let second = batch(&source, &mut buffer, &mut writer, b"");
    assert_eq!((first, second), (long_then(b"bc"), b"d".to_vec()));
}

#[test]
fn a_closed_pipe_ends_after_its_last_bytes() {
    let (source, writer) = written(b"abc");
    drop(writer);
    let mut buffer = [0; 16];
    assert_eq!(read_batch(&source, &mut buffer, || 0), Some(3));
    assert_eq!(read_batch(&source, &mut buffer, || 0), None);
}
