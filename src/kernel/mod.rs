//! Core kernel subsystem - Microkernel implementation

pub mod allocator;
pub mod edge_registry;
pub mod interrupts;
pub mod lazy_pool;
pub mod memory;
pub mod percpu;

use bootloader::BootInfo;

/// Initialize kernel core
pub fn init(boot_info: &'static BootInfo) {
    // Detect firmware type
    let firmware = crate::boot::detect_firmware();
    crate::serial_println!("Firmware: {:?}", firmware);

    // Verify secure boot if enabled
    if let Err(e) = crate::boot::verify_secure_boot() {
        crate::serial_println!("Secure boot verification: {:?}", e);
    }

    // Initialize per-CPU first (needed by other subsystems)
    percpu::init();

    // Then memory (needs per-CPU for statistics)
    memory::init(boot_info);

    // Then interrupts
    interrupts::init();

    // Initialize heap allocator
    allocator::init_heap();
}
