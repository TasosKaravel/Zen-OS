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
    boot::serial::println!("Zen OS v0.1.0 - Booting...");

    // Initialize core kernel components
    kernel::init(boot_info);
    boot::serial::println!("[OK] Kernel core initialized");

    // Initialize memory management
    kernel::memory::init(boot_info);
    boot::serial::println!("[OK] Memory management initialized");

    // Initialize interrupt handling
    kernel::interrupts::init();
    boot::serial::println!("[OK] Interrupt handling initialized");

    // Initialize per-CPU structures
    kernel::percpu::init();
    boot::serial::println!("[OK] Per-CPU structures initialized");

    // Initialize scheduler
    scheduler::init();
    boot::serial::println!("[OK] Scheduler initialized");

    // Initialize IPC subsystem
    ipc::init();
    boot::serial::println!("[OK] IPC subsystem initialized");

    // Initialize capability system
    capability::init();
    boot::serial::println!("[OK] Capability system initialized");

    // Initialize TagFS
    tagfs::init();
    boot::serial::println!("[OK] TagFS initialized");

    // Initialize storage subsystem
    storage::init();
    boot::serial::println!("[OK] Storage subsystem initialized");

    // Initialize GPU/compositor
    gpu::init();
    boot::serial::println!("[OK] GPU subsystem initialized");

    // Initialize AI inference engine
    ai::init();
    boot::serial::println!("[OK] AI inference engine initialized");

    // Initialize userspace environment
    userspace::init();
    boot::serial::println!("[OK] Userspace environment initialized");

    // Initialize compatibility layer
    compat::init();
    boot::serial::println!("[OK] Compatibility layer initialized");

    boot::serial::println!("\n=== Zen OS Boot Complete ===\n");

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
    boot::serial::println!("\n!!! KERNEL PANIC !!!");
    boot::serial::println!("{}", info);
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
