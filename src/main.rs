//! Zen OS - Revolutionary Commercial Operating System
//! 
//! A microkernel-based operating system written in Rust, featuring:
//! - Tag-based file system (TagFS)
//! - Zero-copy IPC with capability-based security
//! - GPU-accelerated compositor
//! - On-device AI inference
//! - Formal verification of critical components

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![deny(unsafe_code)]
#![allow(unsafe_code)] // Only for assembly glue and hardware interaction

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod boot;
mod kernel;
mod scheduler;
mod ipc;
mod capability;
mod tagfs;
mod storage;
mod gpu;
mod ai;
mod userspace;
mod compat;

entry_point!(kernel_main);

/// Kernel entry point called by the bootloader
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize serial output for early debugging
    boot::serial::init();
    crate::serial_println!("Zen OS v0.1.0 - Booting...");

    // Initialize core kernel components
    kernel::init(boot_info);
    crate::serial_println!("[OK] Kernel core initialized");

    // Initialize memory management
    kernel::memory::init(boot_info);
    crate::serial_println!("[OK] Memory management initialized");

    // Initialize interrupt handling
    kernel::interrupts::init();
    crate::serial_println!("[OK] Interrupt handling initialized");

    // Initialize per-CPU structures
    kernel::percpu::init();
    crate::serial_println!("[OK] Per-CPU structures initialized");

    // Initialize scheduler
    scheduler::init();
    crate::serial_println!("[OK] Scheduler initialized");

    // Initialize IPC subsystem
    ipc::init();
    crate::serial_println!("[OK] IPC subsystem initialized");

    // Initialize capability system
    capability::init();
    crate::serial_println!("[OK] Capability system initialized");

    // Initialize TagFS
    tagfs::init();
    crate::serial_println!("[OK] TagFS initialized");

    // Initialize storage subsystem
    storage::init();
    crate::serial_println!("[OK] Storage subsystem initialized");

    // Initialize GPU/compositor
    gpu::init();
    crate::serial_println!("[OK] GPU subsystem initialized");

    // Initialize AI inference engine
    ai::init();
    crate::serial_println!("[OK] AI inference engine initialized");

    // Initialize userspace environment
    userspace::init();
    crate::serial_println!("[OK] Userspace environment initialized");

    // Initialize compatibility layer
    compat::init();
    crate::serial_println!("[OK] Compatibility layer initialized");

    crate::serial_println!("\n=== Zen OS Boot Complete ===\n");

    // Start the scheduler and enter idle loop
    scheduler::start();

    // Should never reach here
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler for the kernel
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::serial_println!("\n!!! KERNEL PANIC !!!");
    crate::serial_println!("{}", info);
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
