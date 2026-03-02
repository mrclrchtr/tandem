#![doc = "Core domain library for tandem."]
#![deny(
    clippy::dbg_macro,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::print_stderr,
    clippy::print_stdout
)]

pub mod error;
pub mod ports;
pub mod ticket;
