//! Universal keyboard device abstraction

extern crate alloc;

mod keys;

use alloc::VecDeque;
use alloc::arc::Arc;

use spin::{Mutex, RwLock};
use x86_64;

use dev::Driver;
use dev::Device;

pub use self::keys::*;

/// Keyboard device
pub struct Kbd {
    inner: Arc<KbdInner>,
}

impl Kbd {
    pub fn new(driver: &Mutex<Driver<KbdDriverApi>>) -> Kbd {
        let kbd = Kbd {
            inner: Arc::new(KbdInner::new()),
        };

        {
            let api = KbdDriverApi::new(&kbd.inner);
            driver.lock().init(api);
        }

        kbd
    }

    /// Synchronously waits for key input from keyboard.
    pub fn wait(&self) -> KeyCode {
        self.inner.wait()
    }
}

impl Device for Kbd {
    const CLASS_NAME: &'static str = "kbd";
}

struct KbdInner {
    buffer: RwLock<VecDeque<KeyCode>>,
}

impl KbdInner {
    fn new() -> KbdInner {
        KbdInner {
            buffer: RwLock::new(VecDeque::new()),
        }
    }

    fn wait(&self) -> KeyCode {
        while self.buffer.read().is_empty() {
            unsafe {
                x86_64::instructions::halt();
            }
        }
        self.buffer.write().pop_front().unwrap()
    }

    fn push(&self, key: KeyCode) {
        self.buffer.write().push_back(key);
    }
}

unsafe impl Sync for KbdInner {}

/// API for keyboard drivers
pub struct KbdDriverApi {
    kbd: Arc<KbdInner>,
}

impl KbdDriverApi {
    fn new(kbd: &Arc<KbdInner>) -> KbdDriverApi {
        KbdDriverApi { kbd: kbd.clone() }
    }
}

impl KbdDriverApi {
    pub fn process_key(&mut self, key: KeyCode) {
        self.kbd.push(key);
    }
}
