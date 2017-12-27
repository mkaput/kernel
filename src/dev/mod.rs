//! Device abstractions

pub mod mgr;

pub mod kbd;
pub mod output_serial;
pub mod text_video;

pub trait Device: Send + Sync {
    fn type_name(&self) -> &'static str;
}

pub trait Driver<D>: Send + Sync {
    fn init(&mut self, api: D);
}
