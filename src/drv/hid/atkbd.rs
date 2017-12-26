use x86_64::structures::idt::ExceptionStackFrame;

use kio::idt::register_interrupt;
use kio::pic;

pub unsafe fn init() {
    register_interrupt(33, handle_irq);
}

extern "x86-interrupt" fn handle_irq(stack_frame: &mut ExceptionStackFrame) {
    println!("got key");
}
