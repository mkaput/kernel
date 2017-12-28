//! Keyboard driver for PCs and ATs

use spin::Mutex;
use x86_64::structures::idt::ExceptionStackFrame;

use kio::idt::register_interrupt;
use kio::pic;
use kio::port::UnsafePort;
use dev::{self, Driver};
use dev::kbd::{Kbd, KbdDriverApi, KeyCode};

const IRQ: u8 = 33;

const DATA_PORT: UnsafePort<u8> = unsafe { UnsafePort::new(0x60) };

static ATKBD: Mutex<AtkbdDriver> = Mutex::new(AtkbdDriver::uninitialized());

pub fn init() {
    ATKBD.lock().start();
    dev::mgr::install(box Kbd::new(&ATKBD));
}

pub struct AtkbdDriver {
    kbd: Option<KbdDriverApi>,
}

impl AtkbdDriver {
    const fn uninitialized() -> AtkbdDriver {
        AtkbdDriver { kbd: None }
    }

    fn start(&mut self) {
        unsafe {
            register_interrupt(IRQ, handle_irq);
            pic::enable(IRQ);
        }
    }

    fn process_scancode(&mut self, scancode: u8) {
        let key = parse_scancode(scancode);
        if let Some(ref mut kbd) = self.kbd {
            kbd.process_key(key);
        }
    }
}

impl Driver<KbdDriverApi> for AtkbdDriver {
    fn init(&mut self, api: KbdDriverApi) {
        self.kbd = Some(api);
    }
}

fn process_scancode(scancode: u8) {
    let mut atkbd = ATKBD.lock();
    atkbd.process_scancode(scancode);
}

fn parse_scancode(scancode: u8) -> KeyCode { KeyCode(scancode) }

extern "x86-interrupt" fn handle_irq(_stack_frame: &mut ExceptionStackFrame) {
    let scancode = unsafe { DATA_PORT.read() };
    process_scancode(scancode);
    unsafe {
        pic::eoi(IRQ);
    }
}
