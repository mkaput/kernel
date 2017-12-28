//! Device manager subsystem

use alloc::{BTreeMap, String};
use alloc::boxed::Box;
use alloc::arc::Arc;
use core::mem;

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

/// Registers new device and manager, and takes ownership of it.
pub fn install<D: Device>(device: Box<D>) -> DeviceName {
    do_install(CommonDevice::new(device))
}

fn do_install(device: CommonDevice) -> DeviceName {
    INSTANCE.write().install(device)
}

/// Returns already cloned shared reference to device.
pub fn get_device(name: &str) -> Option<Arc<CommonDevice>> {
    INSTANCE.read()
        .get(name)
        .map(Arc::clone)
}

pub fn parse_device_name(name: &str) -> Option<(&str, usize)> {
    for (i, ch) in name.char_indices() {
        if ch.is_numeric() {
            let class = unsafe { name.slice_unchecked(0, i) };
            let id = unsafe { name.slice_unchecked(i, name.len()) }
                .parse::<usize>()
                .ok()?;
            return Some((class, id));
        }
    }

    None
}

/// Proxy for common fields of device structures.
pub struct CommonDevice {
    class: &'static str,
    id: usize,
    dev: Box<u8>
}

impl CommonDevice {
    fn new<D: Device>(dev: Box<D>) -> CommonDevice {
        let class = D::CLASS_NAME;
        let id = usize::max_value();
        // FIXME: Handle Drop
        let dev: Box<u8> = unsafe { mem::transmute(dev) };
        CommonDevice { class, id, dev }
    }

    pub fn downcast<D: Device>(&self) -> &D {
        match self.try_downcast() {
            Some(dev) => dev,
            None => panic!("wrong device class: got {}, expected {}", self.class, D::CLASS_NAME),
        }
    }

    pub fn try_downcast<D: Device>(&self) -> Option<&D> {
        if self.class != D::CLASS_NAME { return None; }
        let dev: &D = unsafe { mem::transmute(&self.dev) };
        Some(&dev)
    }

    pub fn class(&self) -> &'static str { self.class }

    pub fn id(&self) -> usize { self.id }

    pub fn name(&self) -> DeviceName {
        format!("{}{}", self.class, self.id)
    }
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

    fn install(&mut self, device: CommonDevice) -> DeviceName {
        self.classes
            .entry(device.class())
            .or_insert_with(|| DeviceClassEntry::new())
            .install(device)
    }

    fn get(&self, name: &str) -> Option<&Arc<CommonDevice>> {
        let (class, id) = parse_device_name(&name).expect("expected valid device name");

        if let Some(devs) = self.classes.get(class) {
            devs.get(id)
        } else {
            None
        }
    }
}

struct DeviceClassEntry {
    devices: BTreeMap<usize, Arc<CommonDevice>>,
    last_id: usize,
}

impl DeviceClassEntry {
    fn new() -> DeviceClassEntry {
        DeviceClassEntry {
            devices: BTreeMap::new(),
            last_id: 0,
        }
    }

    fn install(&mut self, device: CommonDevice) -> DeviceName {
        let id = self.last_id;
        self.last_id += 1;

        let mut dev = device;
        dev.id = id;

        let dev_name = dev.name();

        let r = self.devices.insert(id, Arc::new(dev));
        assert!(r.is_none());

        println!("dev::mgr: connected device {}", dev_name);

        dev_name
    }

    fn get(&self, id: usize) -> Option<&Arc<CommonDevice>> {
        self.devices.get(&id)
    }
}
