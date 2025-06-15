pub mod core {
    pub mod bus;
}

pub mod impls {
    pub mod local;
}

pub mod error;
pub mod global;
pub use core::bus::EventBus;
pub use impls::local::LocalEventBus;
pub use error::EventBusError;