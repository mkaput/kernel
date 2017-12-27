//! Universal keyboard device abstraction

extern crate alloc;

use alloc::VecDeque;
use alloc::arc::Arc;

use spin::Mutex;

use dev::Driver;
use dev::Device;

const BUFFER_SIZE: usize = 256;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyCode {
    pub char: u8,
    pub state: KeyState,
}

/// Keyboard device
pub struct Kbd {
    inner: Arc<Mutex<KbdInner>>,
}

impl Kbd {
    pub fn new(driver: &Mutex<Driver<KbdDriverApi>>) -> Kbd {
        let kbd = Kbd {
            inner: Arc::new(Mutex::new(KbdInner::new())),
        };

        {
            let api = KbdDriverApi::new(kbd.inner.clone());
            driver.lock().init(api);
        }

        kbd
    }

    /// Asynchronously checks if there is key input from keyboard and pulls it,
    /// or returns `None` if there are not any.
    pub fn poll(&mut self) -> Option<KeyCode> {
        let mut lock = self.inner.lock();
        lock.poll()
    }

    /// Synchronously waits for key input from keyboard.
    pub fn wait(&mut self) -> KeyCode {
        // TODO: Optimize this
        loop {
            if let Some(key) = self.poll() {
                return key;
            }
        }
    }
}

impl Device for Kbd {
    fn type_name(&self) -> &'static str {
        "kbd"
    }
}

struct KbdInner {
    buffer: VecDeque<KeyCode>,
}

impl KbdInner {
    fn new() -> KbdInner {
        KbdInner {
            buffer: VecDeque::with_capacity(BUFFER_SIZE),
        }
    }

    fn poll(&mut self) -> Option<KeyCode> {
        self.buffer.pop_front()
    }

    fn push(&mut self, key: KeyCode) {
        self.buffer.push_back(key);
    }
}

/// API for keyboard drivers
pub struct KbdDriverApi {
    kbd: Arc<Mutex<KbdInner>>,
}

impl KbdDriverApi {
    fn new(kbd: Arc<Mutex<KbdInner>>) -> KbdDriverApi {
        KbdDriverApi { kbd }
    }
}

impl KbdDriverApi {
    pub fn process_key(&mut self, key: KeyCode) {
        println!("{:?}", key);
        let mut lock = self.kbd.lock();
        lock.push(key);
    }
}
