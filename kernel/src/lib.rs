#![feature(asm)]
#![feature(inclusive_range_syntax)]
#![feature(lang_items)]
#![no_std]

#[allow(unused_extern_crates)]
extern crate rlibc;

#[no_mangle]
pub extern "C" fn krnl_main() {
    // ATTENTION: we have a very small stack and no guard page

    loop {
        for color_byte in 0x00u8...0xffu8 {
            hello_world(color_byte);
            for i in 0..75000 {
                unsafe {
                    asm!("NOP");
                }
            }
        }
    }
}

fn hello_world(color_byte: u8) {
    let hello = b" Hello World! ";
    let mut hello_colored = [color_byte; 28];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }
    let buffer_ptr = (0xb8000 + 1986) as *mut _;
    unsafe { *buffer_ptr = hello_colored };
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}
