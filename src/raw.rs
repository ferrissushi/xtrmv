use libc::STDIN_FILENO;
use termios::{
    tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP, IXON, OPOST,
    TCSAFLUSH, VMIN, VTIME,
};

pub struct RawMode {
    pub origin_terminal: Termios,
}
use std::io::Result;
impl RawMode {
    pub fn enable_raw_mode() -> Result<Self> {
        let mut terminal = Termios::from_fd(STDIN_FILENO)?;
        let mode = Self {
            origin_terminal: terminal,
        };

        terminal.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        terminal.c_oflag &= !OPOST;
        terminal.c_cflag |= CS8;
        terminal.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        terminal.c_cc[VMIN] = 0;
        terminal.c_cc[VTIME] = 1;

        tcsetattr(STDIN_FILENO, TCSAFLUSH, &terminal)?;
        Ok(mode)
    }

    /// Creates a disabled RawMode for testing
    #[doc(hidden)]
    pub fn new_disabled() -> Self {
        // Create a minimal terminal for testing
        // We use from_fd and if that fails, we create a minimal Termios
        let terminal = match Termios::from_fd(STDIN_FILENO) {
            Ok(t) => t,
            Err(_) => {
                // Create a minimal Termios - most fields can be 0
                unsafe { std::mem::zeroed() }
            }
        };
        Self {
            origin_terminal: terminal,
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // Ignore errors during drop - especially in test environments
        let _ = tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.origin_terminal);
    }
}
