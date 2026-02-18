//! Core kernel subsystem - Microkernel implementation

pub mod memory;
pub mod interrupts;
pub mod percpu;
pub mod edge_registry;
pub mod lazy_pool;
pub mod allocator;

use bootloader::BootInfo;

/// Initialize kernel core
pub fn init(boot_info: &'static BootInfo) {
    // Detect firmware type
    let firmware = crate::boot::detect_firmware();
    crate::boot::serial::println!("Firmware: {:?}", firmware);
    
    // Verify secure boot if enabled
    if let Err(e) = crate::boot::verify_secure_boot() {
        crate::boot::serial::println!("Secure boot verification: {:?}", e);
    }
    
    // Initialize heap allocator
    allocator::init_heap();
}
