pub struct ScancodeQueue {
    buf: [u8; 128],
    head: usize,
    tail: usize,
    len: usize,
}

impl ScancodeQueue {
    pub const fn new() -> Self {
        Self {
            buf: [0; 128],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: u8) -> Result<(), ()> {
        if self.len == self.buf.len() {
            return Err(());
        }

        self.buf[self.tail] = value;
        self.tail = (self.tail + 1) % self.buf.len();
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        let value = self.buf[self.head];
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        Some(value)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }
}
