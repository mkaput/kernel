use x86_64::structures::idt::ExceptionStackFrame;

use kio::idt::register_interrupt;
use kio::pic;
use kio::port::UnsafePort;

const IRQ: u8 = 33;

const CMD_PORT: UnsafePort<u8> = unsafe { UnsafePort::new(0x64) };
const DATA_PORT: UnsafePort<u8> = unsafe { UnsafePort::new(0x60) };

pub unsafe fn init() {
    register_interrupt(IRQ, handle_irq);
    pic::unmask(IRQ);
}

extern "x86-interrupt" fn handle_irq(_stack_frame: &mut ExceptionStackFrame) {
    let scancode = unsafe { DATA_PORT.read() };
    println!("got key {}", scancode);
    unsafe { pic::eoi(IRQ); }
}
