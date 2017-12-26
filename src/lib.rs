#![feature(abi_x86_interrupt)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(asm)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(const_unsafe_cell_new)]
#![feature(global_allocator)]
#![feature(lang_items)]
#![feature(unique)]
#![feature(universal_impl_trait)]
#![feature(nll)]
#![no_std]

extern crate alloc;
extern crate bit_field;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate linked_list_allocator;
// FIXME: this crate is multiboot2 1.6 compliant while we use 2.0
extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

#[macro_use]
pub mod kio;
pub mod dev;
pub mod drv;
pub mod mem;

use linked_list_allocator::LockedHeap;

use dev::text_video::{TextColor, TextStyle};

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(mb_info_addr: usize) {
    // ATTENTION: we have small stack, no guard page and no interrupts configured

    // Set up early console ASAP, so we will be able to use `println!`
    unsafe {
        kio::early_init();
    }

    let boot_info = unsafe { multiboot2::load(mb_info_addr) };

    let bootloader_name = boot_info.boot_loader_name_tag().map(|t| t.name());
    println!("bootloader: {}", bootloader_name.unwrap_or("unknown"));

    let cmdline = boot_info.command_line_tag().map(|t| t.command_line());
    println!("cmdline: {}", cmdline.unwrap_or("none"));

    enable_nxe_bit();
    enable_write_protect_bit();

    unsafe {
        mem::init(boot_info);
        kio::idt::init();
    }

    // ATTENTION: now everything is fine

//    x86_64::instructions::interrupts::int3();

    unreachable!();
}

/// TODO: The heck is this?
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Kernel panic handler
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    let header = TextStyle {
        foreground: TextColor::White,
        background: TextColor::Red,
    };

    let details = TextStyle {
        foreground: TextColor::LightRed,
        background: TextColor::Black,
    };

    println!();

    kio::with_output_style(header, || {
        println!("=== KERNEL PANIC ===");
    });

    kio::with_output_style(details, || {
        println!("{}:{}:", file, line);
        println!("  {}", fmt);
    });

    println!();

    loop {}
}

fn enable_nxe_bit() {
    use x86_64::registers::msr::{rdmsr, wrmsr, IA32_EFER};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{Cr0, cr0, cr0_write};

    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}
