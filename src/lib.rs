use std::sync::{mpsc, Mutex, Arc};
use std::{thread, io};
use termios::{Termios, tcsetattr};
use termios::os::linux::{ICANON, ECHO, TCSANOW};

use termion::input::{TermRead};
use termion::event::Key::Char;
use termion::event::Key as TermionKey;

enum Mode {
    Line,
    Symbol,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Input {
    Line(String),
    Symbol(Key),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,

    __IsNotComplete,
}

pub struct Arl {
    mode: Arc<Mutex<Mode>>,
    stdin_fd: i32,
    termios_default: Termios,
    termios_raw: Termios,
}

impl Default for Arl {
    fn default() -> Self {
        let stdin_fd = 0;
        let termios_default = Termios::from_fd(stdin_fd).unwrap();

        let mut termios_raw = termios_default.clone();
        termios_raw.c_lflag &= !(ICANON | ECHO);

        let mut arl = Arl {
            mode: Arc::new(Mutex::new(Mode::Symbol)),
            stdin_fd,
            termios_default,
            termios_raw,
        };

        arl.symbol_mode();
        arl
    }
}

impl Arl {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn line_mode(&mut self) {
        let mut mode = self.mode.lock().unwrap();
        *mode = Mode::Line;

        tcsetattr(self.stdin_fd, TCSANOW, &self.termios_default).unwrap();
    }

    pub fn symbol_mode(&mut self) {
        let mut mode = self.mode.lock().unwrap();
        *mode = Mode::Symbol;

        tcsetattr(self.stdin_fd, TCSANOW, &mut self.termios_raw).unwrap();
    }

    pub fn start(&mut self) -> impl Iterator<Item=Input>
    {
        let (snd, rcv) = mpsc::channel();

        let mode_inner = Arc::clone(&self.mode);
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut line_buf = vec![];
            for k in stdin.keys() {
                let mode = mode_inner.lock().unwrap();
                match *mode {
                    Mode::Symbol => {
                        match k {
                            Ok(k) => {
                                snd.send(Input::Symbol(Self::convert_key(k)));
                            }
                            _ => ()
                        }
                    }
                    Mode::Line => {
                        match k {
                            Ok(Char('\n')) => {
                                let line = line_buf.iter().collect();
                                line_buf.clear();
                                snd.send(Input::Line(line));
                            }
                            Ok(Char(c)) => {
                                line_buf.push(c);
                            }
                            _ => ()
                        }
                    }
                }
            }
        });

        rcv.into_iter()
    }

    fn convert_key(key: termion::event::Key) -> Key {
        match key {
            TermionKey::Backspace => Key::Backspace,
            TermionKey::Left => Key::Left,
            TermionKey::Right => Key::Right,
            TermionKey::Up => Key::Up,
            TermionKey::Down => Key::Down,
            TermionKey::Home => Key::Home,
            TermionKey::End => Key::End,
            TermionKey::PageUp => Key::PageUp,
            TermionKey::PageDown => Key::PageDown,
            TermionKey::Delete => Key::Delete,
            TermionKey::Insert => Key::Insert,
            TermionKey::F(n) => Key::F(n),
            TermionKey::Char(c) => Key::Char(c),
            TermionKey::Alt(c) => Key::Alt(c),
            TermionKey::Ctrl(c) => Key::Ctrl(c),
            TermionKey::Null => Key::Null,
            TermionKey::Esc => Key::Esc,

            TermionKey::__IsNotComplete => Key::__IsNotComplete
        }
    }
}

impl Drop for Arl {
    fn drop(&mut self) {
        // reset stdin
        tcsetattr(self.stdin_fd, TCSANOW, &self.termios_default).unwrap();
    }
}
