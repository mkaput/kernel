#![feature(alloc)]
#![feature(allocator_api)]
#![feature(asm)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(global_allocator)]
#![feature(lang_items)]
#![feature(unique)]
#![feature(universal_impl_trait)]
#![no_std]

#[macro_use]
extern crate alloc;
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
use mem::{HEAP_START, HEAP_SIZE};
use mem::alloc::KernelAlloc;

#[global_allocator]
static HEAP_ALLOCATOR: KernelAlloc = KernelAlloc::new(HEAP_START, HEAP_START + HEAP_SIZE);


/// Real kernel entry point
#[no_mangle]
pub extern "C" fn krnl_main(mb_info_addr: usize) {
    // ATTENTION: we have small stack and no guard page

    // Set up early console ASAP, so we will be able to use `kprintln!`
    unsafe {
        kio::early_init();
    }

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

    enable_nxe_bit();
    enable_write_protect_bit();

    unsafe {
        mem::init(boot_info);
    }

    // ATTENTION: now everything is fine

    {
        use alloc::boxed::Box;
        let mut heap_test = Box::new(42);
        *heap_test -= 15;
        let heap_test2 = Box::new("hello");
        kprintln!("{:?} {:?}", heap_test, heap_test2);

        let mut vec_test = vec![1, 2, 3, 4, 5, 6, 7];
        vec_test[3] = 42;
        for i in &vec_test {
            kprint!("{} ", i);
        }
        kprintln!();

        for _ in 0..10000 {
            format!("Some String");
        }
    }

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
