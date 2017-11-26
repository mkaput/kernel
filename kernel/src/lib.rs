#![feature(asm)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(lang_items)]
#![feature(unique)]
#![feature(universal_impl_trait)]
#![no_std]

#[macro_use]
extern crate bitflags;
// FIXME: this crate is multiboot2 1.6 compliant while we use 2.0
extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

pub mod dev;
pub mod drv;
#[macro_use]
pub mod kio;
pub mod mem;

use dev::text_video::{TextColor, TextStyle};

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

/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(mb_info_addr: usize) {
    // ATTENTION: we have a very small stack and no guard page

    // Set up early console ASAP, so we will be able to use `kprintln!`
    unsafe {
        kio::early_init();
    }

    enable_nxe_bit();

    let boot_info = unsafe { multiboot2::load(mb_info_addr) };

    let bootloader_name = boot_info
        .boot_loader_name_tag()
        .map(|t| t.name())
        .unwrap_or("unknown");
    kprintln!("bootloader: {}", bootloader_name);

    let cmdline = boot_info
        .command_line_tag()
        .map(|t| t.command_line())
        .unwrap_or("none");
    kprintln!("cmdline: {}", cmdline);

    unsafe {
        mem::init(boot_info);
    }

    // ATTENTION: now everything is fine

    enable_write_protect_bit();

    unreachable!();
}

/// TODO: The heck is this?
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Kernel panic handler
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    let red = TextStyle {
        foreground: TextColor::Red,
        background: TextColor::Black,
    };
    kio::with_output_style(red, || {
        kprintln!("PANIC in {}:{}", file, line);
        kprintln!("  {}", fmt);
    });
    loop {}
}
