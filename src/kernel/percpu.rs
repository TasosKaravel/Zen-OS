//! Per-CPU data structures (1 KB scratch buffers)

use core::sync::atomic::{AtomicU32, Ordering};

/// Maximum number of CPUs supported
pub const MAX_CPUS: usize = 256;

/// Size of per-CPU scratch buffer
pub const SCRATCH_BUFFER_SIZE: usize = 1024;

/// Per-CPU data structure
#[repr(C, align(64))] // Cache-line aligned
pub struct PerCpuData {
    /// CPU ID
    pub cpu_id: u32,
    /// Scratch buffer for temporary allocations
    pub scratch_buffer: [u8; SCRATCH_BUFFER_SIZE],
    /// Current task ID
    pub current_task: AtomicU32,
    /// Idle time counter
    pub idle_ticks: AtomicU32,
}

impl PerCpuData {
    /// Create a new per-CPU data structure
    pub const fn new(cpu_id: u32) -> Self {
        Self {
            cpu_id,
            scratch_buffer: [0; SCRATCH_BUFFER_SIZE],
            current_task: AtomicU32::new(0),
            idle_ticks: AtomicU32::new(0),
        }
    }
}

/// Global per-CPU data array
static mut PER_CPU_DATA: [PerCpuData; MAX_CPUS] = {
    const INIT: PerCpuData = PerCpuData::new(0);
    [INIT; MAX_CPUS]
};

/// Initialize per-CPU structures
pub fn init() {
    // Initialize CPU 0 (BSP)
    unsafe {
        PER_CPU_DATA[0] = PerCpuData::new(0);
    }
    
    // TODO: Initialize additional CPUs (APs) when SMP is implemented
}

/// Get current CPU ID
pub fn current_cpu_id() -> u32 {
    // TODO: Read from APIC or similar
    0
}

/// Get per-CPU data for current CPU
pub fn current() -> &'static PerCpuData {
    let cpu_id = current_cpu_id() as usize;
    unsafe { &PER_CPU_DATA[cpu_id] }
}

/// Get mutable per-CPU data for current CPU
pub fn current_mut() -> &'static mut PerCpuData {
    let cpu_id = current_cpu_id() as usize;
    unsafe { &mut PER_CPU_DATA[cpu_id] }
}
