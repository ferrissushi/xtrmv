use crate::{
    filetype::{get_filetype, load_filetype},
    raw::RawMode,
    row::Row,
    utils::{byte_slice, clear_screen, editor_read_key, get_window_size},
};
use std::io::{BufRead, BufReader};
use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::{stdin, stdout, Result, Stdin, Stdout, Write},
    time::{Duration, Instant},
};

#[macro_export]
macro_rules! ctrl_key {
    ($k:expr) => {
        $k & 0x1f
    };
}
pub const CTRL_C: u8 = ctrl_key!(b'c');
pub const CTRL_W: u8 = ctrl_key!(b'w');
pub const CTRL_H: u8 = ctrl_key!(b'h');
pub const BACKSPACE: u8 = 127;

// ASCII constants
pub const ASCII_SPACE: u8 = 32;
pub const ASCII_DELETE: u8 = 127;
pub const ASCII_DIGIT_OFFSET: u8 = 48;
pub const PRINTABLE_RANGE: std::ops::Range<u8> = ASCII_SPACE..ASCII_DELETE;

// Vim key bindings
pub const KEY_H: u8 = b'h';
pub const KEY_J: u8 = b'j';
pub const KEY_K: u8 = b'k';
pub const KEY_L: u8 = b'l';
pub const KEY_V: u8 = b'v';
pub const KEY_I: u8 = b'i';
pub const KEY_A: u8 = b'a';
pub const KEY_O: u8 = b'o';
pub const KEY_O_UPPER: u8 = b'O';
pub const KEY_W: u8 = b'w';
pub const KEY_B: u8 = b'b';
pub const KEY_E: u8 = b'e';
pub const KEY_X: u8 = b'x';
pub const KEY_D: u8 = b'd';
pub const KEY_G: u8 = b'g';
pub const KEY_H_UPPER: u8 = b'H';
pub const KEY_L_UPPER: u8 = b'L';
pub const KEY_M_UPPER: u8 = b'M';
pub const KEY_A_UPPER: u8 = b'A';
pub const KEY_ZERO: u8 = b'0';

use crate::{
    key::Key,
    mode::{stringify_mode, Mode},
};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Editor {
    pub _mode: RawMode,
    pub line_number_width: usize,
    pub type_mode: Mode,
    /// Cursor column position (character index)
    pub cursor_col: usize,
    /// Cursor row position (0-indexed)
    pub cursor_row: usize,
    /// Visual cursor column position (accounting for tabs)
    pub visual_cursor_col: usize,
    pub initial_colorcolumn: usize,
    /// Selection start column
    pub select_start_col: usize,
    /// Selection start row
    pub select_start_row: usize,
    pub start_key: u8,
    /// Row offset for scrolling
    pub row_offset: usize,
    /// Column offset for scrolling
    pub col_offset: usize,
    pub active_rows: usize,
    pub active_cols: usize,
    pub rows: Vec<Row>,
    pub modified: bool,
    pub stdin: Stdin,
    pub stdout: Stdout,
    pub filename: Option<String>,
    pub filetype: String,
    pub notification: String,
    pub notification_timeout: Instant,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let mode = RawMode::enable_raw_mode()?;
        let (rows, cols) = get_window_size()?;
        let stdin = stdin();
        let stdout = stdout();
        Ok(Self {
            initial_colorcolumn: 100,
            _mode: mode,
            line_number_width: 4,
            type_mode: Mode::Normal,
            filetype: "none".to_string(),
            cursor_col: 0,
            cursor_row: 0,
            visual_cursor_col: 0,
            start_key: 0,
            row_offset: 0,
            col_offset: 0,
            select_start_col: 0,
            select_start_row: 0,
            active_rows: (rows - 2) as usize,
            active_cols: cols as usize,
            rows: Vec::new(),
            modified: false,
            stdin,
            stdout,
            filename: None,
            notification: String::new(),
            notification_timeout: Instant::now(),
        })
    }

    /// Creates an editor for testing purposes without enabling raw mode
    #[doc(hidden)]
    pub fn new_for_test(rows: Vec<Row>) -> Self {
        let stdin = stdin();
        let stdout = stdout();
        let mode = RawMode::new_disabled();
        Self {
            initial_colorcolumn: 100,
            _mode: mode,
            line_number_width: 4,
            type_mode: Mode::Normal,
            filetype: "none".to_string(),
            cursor_col: 0,
            cursor_row: 0,
            visual_cursor_col: 0,
            start_key: 0,
            row_offset: 0,
            col_offset: 0,
            select_start_col: 0,
            select_start_row: 0,
            active_rows: 24,
            active_cols: 80,
            rows,
            modified: false,
            stdin,
            stdout,
            filename: None,
            notification: String::new(),
            notification_timeout: Instant::now(),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.stdout.write_all(buf)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }

    pub fn scroll(&mut self) {
        self.visual_cursor_col = self.cursor_col;
        if self.cursor_row < self.numrows() {
            let row_len = self.rows[self.cursor_row].characters.len();
            let safe_inline_pos = self.cursor_col.min(row_len);
            self.visual_cursor_col = self.rows[self.cursor_row].cx_to_rx(safe_inline_pos);
        }

        if self.cursor_row < self.row_offset {
            self.row_offset = self.cursor_row;
        }

        if self.cursor_row >= self.row_offset + self.active_rows {
            self.row_offset = self.cursor_row.saturating_sub(self.active_rows) + 1;
        }
        if self.visual_cursor_col < self.col_offset {
            self.col_offset = self.visual_cursor_col;
        }
        if self.visual_cursor_col >= self.col_offset + self.active_cols {
            self.col_offset = self.visual_cursor_col.saturating_sub(self.active_cols) + 1;
        }
    }

    pub fn numrows(&self) -> usize {
        self.rows.len()
    }

    pub fn draw_rows(&mut self) -> Result<()> {
        let numrows = self.numrows();
        let mut count = 0;

        for y in 0..(self.active_rows) {
            let filerow = y + self.row_offset;
            if filerow >= numrows {
                if self.filename.is_none()
                    && self.rows.is_empty()
                    && (y >= self.active_rows / 3 && count != 5)
                {
                    let mut msg = "XTRMV".to_string();
                    if count == 1 {
                        msg = "Have fun with our CLI text editor.".to_string();
                    } else if count == 2 {
                        msg = "You can explore various Mode by using the keymaps".to_string();
                    } else if count == 4 {
                        msg = "You can read the documentation by pressing :help or quit with :q!"
                            .to_string();
                    } else if count == 3 {
                        msg = "-----------------".to_string();
                    }
                    count += 1;
                    msg.truncate(self.active_cols);
                    let padding = (self.active_cols - msg.len()) / 2;
                    if padding > 0 {
                        self.write(b"-")?;
                        for _ in 1..padding {
                            self.write(b" ")?;
                        }
                    }
                    self.write(msg.as_bytes())?;
                } else {
                    self.write(b"-")?;
                }
            } else {
                let line_number = format!(
                    "{:>width$} ",
                    filerow + 1,
                    width = self.line_number_width - 1
                );

                self.write(b"\x1b[2m\x1b[90m")?;
                self.write(line_number.as_bytes())?;
                self.write(b"\x1b[22m\x1b[39m")?;

                let content_width = self.active_cols.saturating_sub(self.line_number_width);
                self.stdout.write_all(byte_slice(
                    &self.rows[filerow].render,
                    self.col_offset,
                    content_width,
                ))?;
            }
            self.write(b"\x1b[K")?;
            self.write(b"\r\n")?;
        }
        Ok(())
    }

    pub fn draw_status_bar(&mut self) -> Result<()> {
        self.write(b"\x1b[7m")?; // revert background color
        let status;
        {
            let mode = stringify_mode(&self.type_mode);
            let name = self.filename.as_ref().map_or("[No name]", |s| s.as_str());
            let modified = if self.modified { " (modified)" } else { "" };
            let mut content = format!(
                "-- {} ------ {:.20} - {} lines{}",
                mode,
                name,
                self.numrows(),
                modified
            );
            // :.20 print only 20 letter
            content.truncate(self.active_cols);
            status = content;
        }
        let visual_c_pos_inline = self.visual_cursor_col + 1;
        let visual_c_pos_block = self.cursor_row + 1;
        let right_status_content = format!("{}:{}", visual_c_pos_inline, visual_c_pos_block);

        self.write(status.as_bytes())?;
        let mut len = status.len();
        while len < self.active_cols {
            // This part is trying to put the right status content when the active cols - len of
            // the remain status space is equals to the right status
            if self.active_cols - len == right_status_content.len() {
                self.write(right_status_content.as_bytes())?;
                break;
            } else {
                self.write(b" ")?;
                len += 1;
            }
        }
        self.write(b"\x1b[m")?; // RESET
        self.write(b"\r\n")
    }

    pub fn draw_message_bar(&mut self) -> Result<()> {
        self.write(b"\x1b[K")?;
        if !self.notification.is_empty()
            && self.notification_timeout.elapsed() < Duration::from_secs(5)
        {
            let mut msg = Cow::from(self.notification.as_str());
            if msg.len() > self.active_cols {
                msg.to_mut().truncate(self.active_cols);
            }
            self.stdout.write_all(msg.as_bytes())?;
        }
        Ok(())
    }

    pub fn move_cursor_x_times(
        &mut self,
        mut row_times: usize,
        mut col_times: usize,
        direction: Direction,
    ) -> Result<()> {
        match direction {
            Direction::Up => {
                while row_times > 0 {
                    self.move_cursor_up();
                    row_times -= 1;
                }
            }
            Direction::Down => {
                while row_times > 0 {
                    self.move_cursor_down();
                    row_times -= 1;
                }
            }
            Direction::Left => {
                while col_times > 0 {
                    self.move_cursor_left();
                    col_times -= 1;
                }
            }
            Direction::Right => {
                while col_times > 0 {
                    self.move_cursor_right();
                    col_times -= 1;
                }
            }
        }
        Ok(())
    }

    pub fn try_refresh_screen(&mut self) -> Result<()> {
        self.scroll();

        self.write(b"\x1b[?25l")?; // Hide the cursor.
        self.write(b"\x1b[H")?; // Replace cursor at 1,1.

        self.draw_rows()?;
        self.draw_status_bar()?;
        self.draw_message_bar()?;

        let move_cursor = format!(
            "\x1b[{};{}H",
            (self.cursor_row.saturating_sub(self.row_offset) + 1),
            (self.visual_cursor_col.saturating_sub(self.col_offset)) + 1 + self.line_number_width
        )
        .into_bytes();
        // cursor_row - self.row_offset is the cursor position relative to what's visible.
        // it is dynamic
        self.write(&move_cursor)?;

        self.write(b"\x1b[?25h")?; // Show the real terminal cursor
        self.flush()
    }

    pub fn refresh_screen(&mut self) {
        if let Err(e) = self.try_refresh_screen() {
            self.set_status_message(format!("Screen refresh error: {}", e));
        }
    }

    pub fn set_status_message<S: Into<String>>(&mut self, msg: S) {
        self.notification = msg.into();
        self.notification_timeout = Instant::now();
    }

    pub fn rowlen(&self, index: usize) -> usize {
        let row = self.rows.get(index);
        row.map_or(0, |r| r.characters.len())
    }

    pub fn prompt<F, C>(&mut self, format_prompt: F, mut callback: C) -> Option<String>
    where
        F: Fn(&str) -> String,
        C: FnMut(&mut Self, &str, Key),
    {
        let mut buf = String::new();
        loop {
            self.set_status_message(format_prompt(&buf));
            self.refresh_screen();

            let k = editor_read_key(&mut self.stdin);
            match k {
                Key::Delete | Key::Character(CTRL_H) | Key::Character(BACKSPACE) => {
                    buf.pop();
                }
                Key::Character(b'\x1b') => {
                    self.set_status_message("");
                    callback(self, &buf, k);
                    return None;
                }
                Key::Character(b'\r') => {
                    if !buf.is_empty() {
                        self.set_status_message("");
                        callback(self, &buf, k);
                        return Some(buf);
                    }
                }
                Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                    callback(self, &buf, k);
                }
                Key::Character(c) if PRINTABLE_RANGE.contains(&c) => {
                    buf.push(c as char);
                    callback(self, &buf, k);
                }
                Key::Character(CTRL_C) => {
                    self.type_mode = Mode::Normal;
                }
                _ => (),
            }
        }
    }

    pub fn move_cursor_with_vim_key(&mut self, k: Key) -> bool {
        match k {
            Key::Character(KEY_K) => self.move_cursor_up(),
            Key::Character(KEY_J) => self.move_cursor_down(),
            Key::Character(KEY_H) => self.move_cursor_left(),
            Key::Character(KEY_L) => self.move_cursor_right(),
            _ => (),
        }
        true
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.rowlen(self.cursor_row);
        }
    }

    pub fn move_cursor_right(&mut self) {
        let row = self.rows.get(self.cursor_row);
        let rowlen = row.map_or(0, |r| r.characters.len());
        if self.cursor_col < rowlen {
            self.cursor_col += 1;
        } else if row.is_some() && self.cursor_col == rowlen {
            if self.cursor_row + 1 < self.numrows() {
                self.cursor_col = 0;
                self.cursor_row += 1;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.numrows() != 0 && self.cursor_row < self.numrows() - 1 {
            self.cursor_row += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
        }
    }

    pub fn move_cursor_with_arrow_key(&mut self, k: Key) {
        match k {
            Key::ArrowUp => {
                self.move_cursor_up();
            }
            Key::ArrowDown => {
                self.move_cursor_down();
            }
            Key::ArrowLeft => {
                self.move_cursor_left();
            }
            Key::ArrowRight => {
                self.move_cursor_right();
            }
            _ => (),
        }

        let rowlen = self.rowlen(self.cursor_row);
        if self.cursor_col > rowlen {
            self.cursor_col = rowlen;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_row >= self.rows.len() {
            self.rows.push(Row::new(""));
        }
        let row = &mut self.rows[self.cursor_row];
        let safe_pos = self.cursor_col.min(row.characters.len());
        row.insert_char(safe_pos, c);
        row.update_render_with_syntax(&self.filetype);
        self.cursor_col = safe_pos + 1;
        self.modified = true;
    }

    pub fn insert_new_line(&mut self) {
        if self.cursor_row >= self.rows.len() {
            self.rows.push(Row::new(""));
            self.cursor_row = self.rows.len() - 1;
            self.cursor_col = 0;
            return;
        }
        if self.cursor_col == 0 {
            self.rows.insert(self.cursor_row, Row::new(""));
            if let Some(row) = self.rows.get_mut(self.cursor_row) {
                row.update_render_with_syntax(&self.filetype);
            }
        } else {
            let safe_pos = self
                .cursor_col
                .min(self.rows[self.cursor_row].characters.len());
            let new_line = self.rows[self.cursor_row].truncate(safe_pos);
            self.rows[self.cursor_row].update_render_with_syntax(&self.filetype);
            self.rows.insert(self.cursor_row + 1, Row::new(new_line));
            if let Some(row) = self.rows.get_mut(self.cursor_row + 1) {
                row.update_render_with_syntax(&self.filetype);
            }
        }
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn delete_char_by_space(&mut self) {
        if let Some(r) = self.rows.get(self.cursor_row) {
            let chars: Vec<char> = r.characters.chars().collect();
            loop {
                if self.cursor_col == 0 {
                    break;
                }
                self.delete_char();
                if chars[self.cursor_col] == ' ' {
                    break;
                }
            }
        }
    }
    pub fn delete_char(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.cursor_row >= self.rows.len() {
            self.cursor_row = self.rows.len().saturating_sub(1);
        }
        if self.cursor_row == 0 && self.cursor_col == 0 {
            return;
        }
        if self.cursor_col > 0 {
            let safe_pos = self.cursor_col.saturating_sub(1);
            self.cursor_col = safe_pos;
            if let Some(row) = self.rows.get_mut(self.cursor_row) {
                row.delete_char(safe_pos);
                row.update_render_with_syntax(&self.filetype);
            }
        } else if self.cursor_row > 0 {
            let right = self.rows.remove(self.cursor_row);
            self.cursor_row -= 1;
            if let Some(left) = self.rows.get_mut(self.cursor_row) {
                self.cursor_col = left.characters.len();
                left.append_str(right.characters.as_str());
                left.update_render_with_syntax(&self.filetype);
            }
        }
        self.modified = true;
    }

    pub fn try_starting_process(&mut self) -> bool {
        match self.type_mode {
            Mode::Select => self.process_keypress(Mode::Select),
            Mode::Normal => self.process_keypress(Mode::Normal),
            Mode::Insert => self.process_keypress(Mode::Insert),
            Mode::LineSelect => self.process_keypress(Mode::LineSelect),
        }
    }

    pub fn process_keypress(&mut self, current_mode: Mode) -> bool {
        let c = editor_read_key(&mut self.stdin);
        match current_mode {
            Mode::Insert => self.insert_process(c),
            Mode::Normal => self.normal_process(c),
            Mode::Select => self.select_process(c),
            // Here I should add another function to process it.
            Mode::LineSelect => self.line_select_process(c),
        }
    }

    pub fn line_select_process(&mut self, c: Key) -> bool {
        // TODO: Fill this.
        true
    }

    pub fn cmd_process(&mut self) -> bool {
        let command = self.prompt(|v| format!(":{}", v), |_, _, _| ());
        if let Some(cmd) = command {
            if cmd == "q!" {
                return false;
            } else if cmd == "q" {
                if self.modified {
                    let msg = "WARNING!!! File has unsaved changes. \
                         Press :q! to quit without saving."
                        .to_string();

                    self.set_status_message(msg);
                    return true;
                } else {
                    return false;
                }
            } else if cmd == "w" {
                self.save();
            } else if cmd == "wq" || cmd == "x" {
                self.save();
                return false;
            } else if cmd == "help" {
                return false;
            } else if cmd == "set filetype" {
                self.set_status_message(self.filetype.clone());
                return true;
            } else if cmd == "set colorcolumn" {
            }
        }
        true
    }

    pub fn select_process(&mut self, c: Key) -> bool {
        match c {
            Key::Character(CTRL_C) => {
                self.type_mode = Mode::Normal;
                self.cursor_col = self.select_start_col;
                self.cursor_row = self.select_start_row;
                self.select_start_col = 0;
                self.select_start_row = 0;
                return true;
            }
            Key::Character(b'h')
            | Key::Character(b'j')
            | Key::Character(b'k')
            | Key::Character(b'l') => {
                self.move_cursor_with_vim_key(c);
                return true;
            }
            Key::Character(b'x') | Key::Character(b'd') => {
                if self.cursor_row < self.select_start_row
                    || (self.cursor_row == self.select_start_row
                        && self.cursor_col < self.select_start_col)
                {
                    let temp_x = self.cursor_col;
                    let temp_y = self.cursor_row;
                    self.cursor_col = self.select_start_col;
                    self.cursor_row = self.select_start_row;
                    self.select_start_col = temp_x;
                    self.select_start_row = temp_y;
                }
                while (self.cursor_col, self.cursor_row)
                    != (self.select_start_col, self.select_start_row)
                {
                    self.delete_char();
                }
                true
            }
            Key::Character(b'w') | Key::Character(b'b') | Key::Character(b'e') => {
                self.move_by_space(c)
            }
            _ => true,
        };
        true
    }

    pub fn normal_process(&mut self, c: Key) -> bool {
        match c {
            Key::Character(b'k')
            | Key::Character(b'j')
            | Key::Character(b'l')
            | Key::Character(b'h') => self.move_cursor_with_vim_key(c),
            Key::Character(b'x') => {
                self.cursor_col += 1;
                self.delete_char();
                true
            }
            Key::Character(b'v') => {
                self.select_start_col = self.cursor_col;
                self.select_start_row = self.cursor_row;
                self.type_mode = Mode::Select;
                true
            }
            Key::Character(b'i') => {
                self.type_mode = Mode::Insert;
                true
            }
            Key::Character(b'a') => {
                self.type_mode = Mode::Insert;
                self.move_cursor_x_times(0, 1, Direction::Right).unwrap();
                true
            }
            Key::Character(b'o') => {
                self.type_mode = Mode::Insert;
                self.cursor_col = self.rowlen(self.cursor_row);
                self.insert_new_line();
                self.cursor_row = self
                    .cursor_row
                    .saturating_add(1)
                    .min(self.rows.len().saturating_sub(1));
                true
            }
            Key::Character(b'O') => {
                self.type_mode = Mode::Insert;
                self.cursor_col = 0;
                self.insert_new_line();
                self.cursor_row = self.cursor_row.saturating_sub(1);
                true
            }
            Key::Character(b'H') => {
                self.cursor_row = 0;
                self.cursor_col = 0;
                true
            }
            Key::Character(b'L') => {
                self.cursor_col = 0;
                self.cursor_row = self.rows.len().saturating_sub(1);
                true
            }
            Key::Character(b'0') => {
                self.cursor_col = 0;
                true
            }
            Key::Character(b'$') => {
                self.cursor_col = self.rowlen(self.cursor_row);
                true
            }
            Key::Character(b'M') => {
                let middle_pos = (self.rows.len() - 1) / 2;
                self.cursor_row = middle_pos;
                true
            }
            Key::Character(b'G') => {
                self.cursor_row = self.rows.len() - 1;
                self.cursor_col = 0;
                true
            }
            Key::Character(b'g') => {
                self.start_key = b'g';
                self.double_key_press(self.start_key)
            }
            Key::Character(b':') => self.cmd_process(),
            Key::Character(b'w') | Key::Character(b'b') | Key::Character(b'e') => {
                self.move_by_space(c)
            }
            Key::Character(b'A') => {
                self.cursor_col = self.rowlen(self.cursor_row);
                self.type_mode = Mode::Insert;
                true
            }
            Key::Character(bkey) => {
                if bkey.is_ascii_digit() {
                    let (repetion_count, last_pressed_key) = self.multiple_key_press(bkey);
                    if last_pressed_key == b'j' {
                        self.move_cursor_x_times(repetion_count, 0, Direction::Down)
                            .unwrap();
                    } else if last_pressed_key == b'k' {
                        self.move_cursor_x_times(repetion_count, 0, Direction::Up)
                            .unwrap();
                    } else if last_pressed_key == b'l' {
                        self.move_cursor_x_times(0, repetion_count, Direction::Right)
                            .unwrap();
                    } else if last_pressed_key == b'h' {
                        self.move_cursor_x_times(0, repetion_count, Direction::Left)
                            .unwrap();
                    }
                }
                true
            }
            _ => true,
        }
    }

    pub fn multiple_key_press(&mut self, first_key: u8) -> (usize, u8) {
        let mut keys_string = String::from((first_key - ASCII_DIGIT_OFFSET).to_string().as_str());
        let mut number: usize = 0;
        let mut key = editor_read_key(&mut self.stdin);
        let mut last_key_press: u8 = 0;
        while let Key::Character(x) = key {
            if !x.is_ascii_digit() {
                number = match keys_string.parse::<usize>() {
                    Ok(n) => n,
                    Err(_e) => {
                        return (0, 0);
                    }
                };
                last_key_press = x;
                break;
            } else {
                let current_byte_to_number = x - ASCII_DIGIT_OFFSET;
                keys_string.push_str(current_byte_to_number.to_string().as_str());
            }
            key = editor_read_key(&mut self.stdin);
        }
        (number, last_key_press)
    }

    pub fn double_key_press(&mut self, first_byte_key: u8) -> bool {
        let second_key = editor_read_key(&mut self.stdin);
        if first_byte_key == b'g' && second_key == Key::Character(b'g') {
            self.cursor_row = 0;
            self.cursor_col = 0;
        }
        true
    }

    pub fn move_by_space(&mut self, k: Key) -> bool {
        if let Some(r) = self.rows.get(self.cursor_row) {
            let chars: Vec<char> = r.characters.chars().collect();
            loop {
                match k {
                    Key::Character(b'w') => {
                        if self.cursor_col >= chars.len() {
                            if self.cursor_row + 1 < self.rows.len() {
                                self.cursor_row += 1;
                                self.cursor_col = 0;
                                return true;
                            } else {
                                break;
                            }
                        }

                        if chars[self.cursor_col] == ' '
                            && (self.cursor_col + 1 < chars.len()
                                && chars[self.cursor_col + 1] != ' ')
                        {
                            self.cursor_col += 1;
                            break;
                        } else if self.cursor_col >= chars.len() - 1 {
                            self.cursor_col = chars.len();
                            break;
                        } else {
                            self.cursor_col += 1;
                        }
                    }

                    Key::Character(b'b') => {
                        if self.cursor_col == 0 {
                            if self.cursor_row == 0 {
                                break;
                            } else {
                                self.cursor_row -= 1;
                                self.cursor_col = self.rowlen(self.cursor_row);
                                return true;
                            }
                        }

                        if self.cursor_col >= chars.len() {
                            self.cursor_col = chars.len() - 1;
                        }

                        if chars[self.cursor_col] != ' '
                            && self.cursor_col > 0
                            && chars[self.cursor_col - 1] == ' '
                        {
                            self.cursor_col -= 1;
                            break;
                        } else {
                            self.cursor_col -= 1;
                        }
                    }
                    _ => return true,
                }
            }
        }
        true
    }

    pub fn insert_tab(&mut self) {
        for _i in 0..2 {
            self.insert_char(' ');
            self.write(b" ").unwrap();
        }
    }

    pub fn insert_process(&mut self, c: Key) -> bool {
        match c {
            Key::Character(CTRL_W) => {
                self.delete_char_by_space();
                return true;
            }
            Key::Character(CTRL_C) => {
                self.type_mode = Mode::Normal;
            }
            Key::Character(b'\t') => self.insert_tab(),
            Key::Character(b'\r') => self.insert_new_line(),
            Key::Character(CTRL_H) | Key::Character(BACKSPACE) => self.delete_char(),
            Key::Delete => {
                self.move_cursor_with_arrow_key(Key::ArrowRight);
                self.delete_char();
            }
            Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
                self.move_cursor_with_arrow_key(c);
            }
            Key::Character(b'(') => {
                self.insert_char('(');
                self.insert_char(')');
                self.cursor_col -= 1;
            }
            Key::Character(k) if PRINTABLE_RANGE.contains(&k) => self.insert_char(k as char),
            _ => (),
        };
        true
    }

    pub fn open(&mut self, filename: &str) -> Result<()> {
        self.filename = Some(filename.to_owned());
        let f = File::open(filename)?;
        let file = BufReader::new(&f);
        let results: Result<Vec<Row>> = file.lines().map(|r| r.map(Row::new)).collect();
        self.rows = results?;
        let map = load_filetype("src/filetype.json");
        let filetype = get_filetype(filename, &map);
        self.filetype = filetype.clone();
        for row in &mut self.rows {
            row.update_render_with_syntax(&self.filetype);
        }
        self.modified = false;
        Ok(())
    }

    pub fn save_to_file(&mut self) -> Result<usize> {
        let filename = match self.filename {
            Some(ref f) => f,
            None => return Ok(0),
        };
        let mut data: Vec<u8> = Vec::new();
        for row in &self.rows {
            writeln!(data, "{}", &row.characters)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;
        file.set_len(data.len() as u64)?;
        file.write_all(&data)?;
        self.modified = false;
        let map = load_filetype("src/filetype.json");
        let filetype = get_filetype(filename, &map);
        self.filetype = filetype.clone();
        Ok(data.len())
    }

    pub fn save(&mut self) {
        if self.filename.is_none() {
            self.filename = self.prompt(|v| format!("Save as: {}", v), |_, _, _| ());
            if self.filename.is_none() {
                self.set_status_message("Save aborted");
                return;
            }
        }
        match self.save_to_file() {
            Ok(size) => self.set_status_message(format!("{} bytes written to disk", size)),
            Err(e) => self.set_status_message(format!("Can't save! I/O error: {}", e)),
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = clear_screen(&mut self.stdout);
    }
}
