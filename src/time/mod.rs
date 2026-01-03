pub mod sleep;
pub mod timeout;
pub mod wrapper;

pub use sleep::sleep;
pub use timeout::timeout;

pub enum TimeError {
    TimeOut,
}
