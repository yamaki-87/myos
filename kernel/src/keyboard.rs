use crate::{clear_screen, interrupts, print, println, spinlock::SpinLock, utils::wrapper};

pub fn handle_scancode(scancode: u8) {
    if scancode & 0x80 != 0 {
        return;
    }

    match scancode {
        0x0E => {
            let removed = wrapper::without_interrupts_fn(|| {
                let mut input = INPUT_BUF.lock();
                input.pop()
            });
            if removed.is_some() {
                print!("\x08 \x08");
            }
        }
        0x1C => {
            handle_enter();
        }
        _ => {
            if let Some(ch) = decode_scancode(scancode) {
                wrapper::without_interrupts_fn(|| {
                    let mut input = INPUT_BUF.lock();
                    let _ = input.push(ch as u8);
                });

                print!("{}", ch);
            }
        }
    }
}

fn handle_enter() {
    print!("\n");
    let line = wrapper::without_interrupts_fn(|| {
        let mut input = INPUT_BUF.lock();

        let len = input.len;
        let mut out = [0u8; 128];
        out[..len].copy_from_slice(input.as_slice());
        input.clear();

        (out, len)
    });

    process_line(&line.0[..line.1]);

    print!("> ");
}

fn process_line(line: &[u8]) {
    match line {
        b"" => {}
        b"help" => {
            println!("commands: help clear ticks");
        }
        b"clear" => {
            clear_screen!();
        }
        b"ticks" => {
            let ticks = interrupts::TICKS.load(core::sync::atomic::Ordering::Relaxed);
            println!("ticks = {}", ticks);
        }
        _ => {
            if let Ok(s) = core::str::from_utf8(line) {
                println!("unknown command: {}", s);
            } else {
                println!("unknown command");
            }
        }
    }
}
fn decode_scancode(sc: u8) -> Option<char> {
    match sc {
        0x1E => Some('a'),
        0x30 => Some('b'),
        0x2E => Some('c'),
        0x20 => Some('d'),
        0x12 => Some('e'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x17 => Some('i'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x32 => Some('m'),
        0x31 => Some('n'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x10 => Some('q'),
        0x13 => Some('r'),
        0x1F => Some('s'),
        0x14 => Some('t'),
        0x16 => Some('u'),
        0x2F => Some('v'),
        0x11 => Some('w'),
        0x2D => Some('x'),
        0x15 => Some('y'),
        0x2C => Some('z'),
        0x39 => Some(' '),
        0x1C => Some('\n'),
        _ => None,
    }
}

pub struct LineBuffer {
    buf: [u8; 128],
    len: usize,
}

impl LineBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; 128],
            len: 0,
        }
    }

    pub fn push(&mut self, b: u8) -> Result<(), ()> {
        if self.len == self.buf.len() {
            return Err(());
        }
        self.buf[self.len] = b;
        self.len += 1;
        Ok(())
    }
    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.buf[self.len])
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
    pub fn len(&self) -> usize {
        self.len
    }
}

pub static INPUT_BUF: SpinLock<LineBuffer> = SpinLock::new(LineBuffer::new());
