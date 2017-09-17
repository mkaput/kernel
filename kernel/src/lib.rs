#![feature(lang_items)]
#![no_std]

#[allow(unused_extern_crates)]
extern crate rlibc;

#[no_mangle]
pub extern "C" fn krnl_main() {
    // ATTENTION: we have a very small stack and no guard page

    let hello = b"Hello World!";
    let color_byte = 0x0f;

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i * 2] = *char_byte;
    }

    // write `Hello World!` to the center of the VGA text buffer
    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt() -> ! {
    loop {}
}
