use core::fmt::{self, Write};

use bootloader_api::{
    BootInfo,
    info::{FrameBufferInfo, PixelFormat},
};

use crate::{font, spinlock::SpinLock, utils::wrapper};

pub static WRITER: SpinLock<Option<FrameBufferWriter<'static>>> = SpinLock::new(None);

pub fn init_writer(boot_info: &'static mut BootInfo) {
    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("framebuffer not available");

    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    let writer = FrameBufferWriter::new(buffer, info);

    *WRITER.lock() = Some(writer);
}

pub fn _print(args: fmt::Arguments) {
    wrapper::without_interrupts_fn(|| {
        let mut guard = WRITER.lock();
        if let Some(writer) = guard.as_mut() {
            writer.write_fmt(args).ok();
        }
    });
}

pub fn _clear_screen() {
    wrapper::without_interrupts_fn(|| {
        let mut guard = WRITER.lock();
        if let Some(writer) = guard.as_mut() {
            let bg = writer.bg;
            writer.clear(bg);
        }
    })
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::draw_logic::_print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::draw_logic::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print!("{}\n", format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! clear_screen {
    () => {
        $crate::draw_logic::_clear_screen();
    };
}

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}
impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
    };
}

pub struct FrameBufferWriter<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,
    fg: Color,
    bg: Color,
}
impl<'a> FrameBufferWriter<'a> {
    const CHAR_WIDTH: usize = 8;
    const CHAR_HEIGHT: usize = 8;
    const CHAR_SPACING: usize = 1;
    const LINE_SPACING: usize = 2;
    const LEFT_PADDING: usize = 8;
    const TOP_PADDING: usize = 8;

    pub fn new(buffer: &'a mut [u8], info: FrameBufferInfo) -> Self {
        Self {
            buffer,
            info,
            cursor_x: Self::LEFT_PADDING,
            cursor_y: Self::TOP_PADDING,
            fg: Color::WHITE,
            bg: Color::BLACK,
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.fill_rect(0, 0, self.info.width, self.info.height, color);
        self.cursor_x = Self::LEFT_PADDING;
        self.cursor_y = Self::TOP_PADDING;
        self.bg = color;
    }
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel_index = y * self.info.stride + x;
        let byte_index = pixel_index * self.info.bytes_per_pixel;

        if byte_index + self.info.bytes_per_pixel > self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[byte_index] = color.r;
                self.buffer[byte_index + 1] = color.g;
                self.buffer[byte_index + 2] = color.b;
            }
            PixelFormat::Bgr => {
                self.buffer[byte_index] = color.b;
                self.buffer[byte_index + 1] = color.g;
                self.buffer[byte_index + 2] = color.r;
            }
            PixelFormat::U8 => {
                let gray = ((color.r as u16 + color.g as u16 + color.b as u16) / 3) as u8;
                self.buffer[byte_index] = gray;
            }
            _ => {}
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        let x_end = x.saturating_add(width).min(self.info.width);
        let y_end = y.saturating_add(height).min(self.info.height);

        for py in y..y_end {
            for px in x..x_end {
                self.draw_pixel(px, py, color);
            }
        }
    }

    fn newline(&mut self) {
        self.cursor_x = Self::LEFT_PADDING;
        self.cursor_y += Self::CHAR_HEIGHT + Self::LINE_SPACING;

        let max_y = self
            .info
            .height
            .saturating_sub(Self::CHAR_HEIGHT + Self::TOP_PADDING);
        if self.cursor_y > max_y {
            self.cursor_y = Self::TOP_PADDING;
        }
    }

    fn draw_char_at(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        let glyph = font::glyph_for(ch);

        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..Self::CHAR_WIDTH {
                // This font data is LSB-left:
                // bit0 = leftmost pixel, bit7 = rightmost pixel.
                //let mask = 1 << (7 - col);
                let mask = 1 << col;
                let color = if (bits & mask) != 0 { fg } else { bg };
                self.draw_pixel(x + col, y + row, color);
            }
        }
    }
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.newline();
            }
            _ => {
                let next_x = self.cursor_x + Self::CHAR_WIDTH;
                if next_x >= self.info.width {
                    self.newline();
                }

                self.draw_char_at(self.cursor_x, self.cursor_y, ch, self.fg, self.bg);
                self.cursor_x += Self::CHAR_WIDTH + Self::CHAR_SPACING;
            }
        }
    }
}

impl fmt::Write for FrameBufferWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            self.write_char(ch);
        }
        Ok(())
    }
}
