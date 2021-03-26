#[macro_use]
mod macros;

pub mod client;
pub use client::Client;

pub mod error;
pub use error::Error;

pub mod users;
pub use users::*;

pub type Result<T> = std::result::Result<T, Error>;
