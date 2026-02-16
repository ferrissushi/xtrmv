use crate::key::Key;
use libc::winsize;
use libc::STDOUT_FILENO;
use libc::TIOCGWINSZ;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::io::Stdin;
use std::io::Stdout;
use std::io::Write;

pub fn clear_screen(stdout: &mut Stdout) -> Result<()> {
    stdout.write_all(b"\x1b[2J")?;
    stdout.write_all(b"\x1b[H")?;
    stdout.flush()
}

pub fn get_window_size() -> Result<(u16, u16)> {
    let mut ws = winsize {
        ws_col: 0,
        ws_row: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        if libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) == -1 || ws.ws_col == 0 {
            return Err(Error::other("get_window_size: ioctl failed"));
        }
    }
    Ok((ws.ws_row, ws.ws_col))
}

pub const TAB_STOP: usize = 8;

pub fn read_non_blocking<R: Read>(r: &mut R, buf: &mut [u8]) -> usize {
    match r.read(buf) {
        Ok(n) => n,
        Err(e) if e.kind() == ErrorKind::WouldBlock => 0,
        Err(_) => 0,
    }
}

pub fn byte_slice(s: &str, offset: usize, max_len: usize) -> &[u8] {
    if s.len() > max_len + offset {
        &s.as_bytes()[offset..(max_len + offset)]
    } else if s.len() > offset {
        &s.as_bytes()[offset..]
    } else {
        &s.as_bytes()[0..0]
    }
}

pub fn read_escape_sequence(stdin: &mut Stdin) -> Key {
    let mut seq = [0; 2];
    let n = read_non_blocking(stdin, &mut seq);
    if n == 2 && seq[0] == b'[' {
        if seq[1] >= b'0' && seq[1] <= b'9' {
            let mut last = [0; 1];
            if read_non_blocking(stdin, &mut last) == 1 && last[0] == b'~' {
                match seq[1] {
                    b'1' | b'7' => Key::Home,
                    b'3' => Key::Delete,
                    b'4' | b'8' => Key::End,
                    b'5' => Key::PageUp,
                    b'6' => Key::PageDown,
                    _ => Key::Character(b'\x1b'),
                }
            } else {
                Key::Character(b'\x1b')
            }
        } else {
            match seq[1] {
                b'A' => Key::ArrowUp,
                b'B' => Key::ArrowDown,
                b'C' => Key::ArrowRight,
                b'D' => Key::ArrowLeft,
                b'H' => Key::Home,
                b'F' => Key::End,
                _ => Key::Character(b'\x1b'),
            }
        }
    } else if n == 2 && seq[0] == b'O' {
        match seq[1] {
            b'H' => Key::Home,
            b'F' => Key::End,
            _ => Key::Character(b'\x1b'),
        }
    } else {
        Key::Character(b'\x1b')
    }
}

pub fn editor_read_key(stdin: &mut Stdin) -> Key {
    let mut buf = [0; 1];
    loop {
        if read_non_blocking(stdin, &mut buf) == 1 {
            return match buf[0] {
                b'\x1b' => read_escape_sequence(stdin),
                ch => Key::Character(ch),
            };
        }
    }
}
