//! Boot subsystem - UEFI/BIOS bootloader and early initialization

pub mod serial;
pub mod bootloader;

pub use self::bootloader::*;
