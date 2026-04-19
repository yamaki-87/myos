#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
mod allocator;
mod draw_logic;
mod font;
mod gdt;
mod interrupts;
mod keyboard;
mod memory;
mod scancode_queue;
mod serial;
mod spinlock;
mod utils;

extern crate alloc;

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec::Vec};
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use x86_64::{VirtAddr, structures::paging::OffsetPageTable};

use crate::draw_logic::Color;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn init(
    boot_info: &'static mut BootInfo,
) -> (memory::BootInfoFrameAllocator, OffsetPageTable<'static>) {
    serial::serial_write_str("init start");
    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .as_ref()
            .cloned()
            .expect("failed to get physical_memory_offset"),
    );
    serial::serial_write_str("init phys mem done\n");
    let mapper = unsafe { memory::init(phys_mem_offset) };
    serial::serial_write_str("init mapper done\n");
    let frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    serial::serial_write_str("init frame allocator done\n");

    draw_logic::init_writer(
        boot_info
            .framebuffer
            .as_mut()
            .expect("framebuffer not available"),
    );
    gdt::init_gdt();
    interrupts::init_pics();
    x86_64::instructions::interrupts::enable();
    interrupts::init_idt();

    (frame_allocator, mapper)
}

fn screen_init() {
    {
        let mut guard = draw_logic::WRITER.lock();
        if let Some(writer) = guard.as_mut() {
            writer.clear(Color::BLACK);
            writer.set_color(Color::WHITE, Color::BLACK);
        }
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial::serial_init();
    serial::serial_write_str("entered kernel_main\n");
    let (mut frame_allocator, mut mapper) = init(boot_info);
    screen_init();
    serial::serial_write_str("init done\n");

    if let Err(e) = allocator::init_heap(&mut mapper, &mut frame_allocator) {
        println!("heap allocate errror:{:?}", e);
    }

    println!("KERNEL START");
    println!("BOOT OK");

    // x86_64::instructions::interrupts::int3();
    // unsafe {
    //     core::ptr::write_volatile(0 as *mut u64, 42);
    // }
    let x = Box::new(41);
    println!("heap value = {}", x);

    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    println!("vec = {:?}", v);

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
