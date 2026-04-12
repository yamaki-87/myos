#![no_std]
#![no_main]

mod draw_logic;
mod serial;
mod spinlock;
use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};

use crate::draw_logic::{Color, FrameBufferWriter};
use core::fmt::Write;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial::serial_init();
    serial::serial_write_str("entered kernel_main\n");

    if let Some(fb) = boot_info.framebuffer.as_mut() {
        serial::serial_write_str("framebuffer exists\n");
        draw_logic::init_writer(boot_info);

        {
            let mut guard = draw_logic::WRITER.lock();
            if let Some(writer) = guard.as_mut() {
                writer.clear(Color::BLACK);
                writer.set_color(Color::WHITE, Color::BLACK);
            }
        }

        println!("HELLO");
        println!("KERNEL START");
        println!("BOOT OK");
    } else {
        serial::serial_write_str("framebuffer none\n");
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
