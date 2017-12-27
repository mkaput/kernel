//! Device manager subsystem

use alloc::{BTreeMap, String};
use alloc::boxed::Box;

use hashmap_core::HashMap;
use lazy_static;
use spin::RwLock;

use dev::Device;

pub type DeviceName = String;

lazy_static! {
    static ref INSTANCE: RwLock<DeviceManager> = RwLock::new(DeviceManager::new());
}

pub fn init() {
    lazy_static::initialize(&INSTANCE);
}

pub fn install(device: Box<Device>) -> DeviceName {
    INSTANCE.write().install(device)
}

struct DeviceManager {
    classes: HashMap<&'static str, DeviceClassEntry>,
}

impl DeviceManager {
    fn new() -> DeviceManager {
        DeviceManager {
            classes: HashMap::new(),
        }
    }

    fn install(&mut self, device: Box<Device>) -> DeviceName {
        self.classes.entry(device.type_name())
            .or_insert_with(|| DeviceClassEntry::new())
            .install(device)
    }
}

struct DeviceClassEntry {
    devices: BTreeMap<usize, RwLock<Box<Device>>>,
    last_id: usize,
}

impl DeviceClassEntry {
    fn new() -> DeviceClassEntry {
        DeviceClassEntry {
            devices: BTreeMap::new(),
            last_id: 0,
        }
    }

    fn install(&mut self, device: Box<Device>) -> DeviceName {
        let id = self.last_id;
        self.last_id += 1;

        let dev_name = format!("{}{}", device.type_name(), id);

        let r = self.devices.insert(id, RwLock::new(device));
        assert!(r.is_none());

        println!("dev::mgr: connected device {}", dev_name);

        dev_name
    }
}
