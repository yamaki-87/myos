#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
mod draw_logic;
mod font;
mod gdt;
mod interrupts;
mod keyboard;
mod scancode_queue;
mod serial;
mod spinlock;
mod utils;
use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};

use crate::draw_logic::{Color, FrameBufferWriter};

entry_point!(kernel_main);

fn init(boot_info: &'static mut BootInfo) {
    draw_logic::init_writer(boot_info);
    gdt::init_gdt();
    interrupts::init_pics();
    x86_64::instructions::interrupts::enable();
    interrupts::init_idt();
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial::serial_init();
    serial::serial_write_str("entered kernel_main\n");

    serial::serial_write_str("framebuffer exists\n");
    init(boot_info);
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
    print!(".");

    // x86_64::instructions::interrupts::int3();
    // unsafe {
    //     core::ptr::write_volatile(0 as *mut u64, 42);
    // }

    loop {
        loop {
            let scancode = {
                let mut queue = interrupts::SCANCODE_QUEUE.lock();
                queue.pop()
            };

            match scancode {
                Some(sc) => keyboard::handle_scancode(sc),
                None => break,
            }
        }
        // let ticks = interrupts::TICKS.load(core::sync::atomic::Ordering::Relaxed);
        // println!("ticks = {}", ticks);
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
