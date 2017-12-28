//! Device abstractions

pub mod mgr;

pub mod kbd;
pub mod output_serial;
pub mod text_video;

/// Devices are required to be externally immutable.
pub trait Device: Send + Sync {
    const CLASS_NAME: &'static str;
}

pub trait Driver<D>: Send + Sync {
    fn init(&mut self, api: D);
}
