#[cfg(feature = "desktop")]
mod app;
pub mod domain;
pub mod network;
pub mod platform;
pub mod storage;
#[cfg(feature = "desktop")]
pub use app::run;
