//! Hybrid stride-based scheduler with per-CPU run queues

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use heapless::Vec;

/// Maximum number of tasks per CPU
pub const MAX_TASKS_PER_CPU: usize = 256;

/// Task descriptor (packed for cache efficiency)
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct TaskDesc {
    /// Task ID
    pub id: u32,
    /// Task priority (stride value)
    pub stride: u32,
    /// Pass value for stride scheduling
    pub pass: u64,
    /// Task state
    pub state: TaskState,
    /// Stack pointer
    pub stack_ptr: u64,
    /// Instruction pointer
    pub instruction_ptr: u64,
}

impl TaskDesc {
    /// Create a new task descriptor
    pub const fn new(id: u32, stride: u32) -> Self {
        Self {
            id,
            stride,
            pass: 0,
            state: TaskState::Ready,
            stack_ptr: 0,
            instruction_ptr: 0,
        }
    }
}

/// Task state
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskState {
    Ready = 0,
    Running = 1,
    Blocked = 2,
    Terminated = 3,
}

/// Per-CPU run queue (cache-line aligned, lock-free)
#[repr(C, align(64))]
pub struct RunQueue {
    tasks: Vec<TaskDesc, MAX_TASKS_PER_CPU>,
    current_index: AtomicU32,
}

impl RunQueue {
    /// Create a new empty run queue
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_index: AtomicU32::new(0),
        }
    }

    /// Add a task to the run queue
    pub fn enqueue(&mut self, task: TaskDesc) -> Result<(), SchedulerError> {
        self.tasks.push(task).map_err(|_| SchedulerError::QueueFull)
    }

    /// Get next task to run (stride scheduling)
    pub fn next_task(&mut self) -> Option<&mut TaskDesc> {
        if self.tasks.is_empty() {
            return None;
        }

        // Find task with minimum pass value
        let mut min_idx = 0;
        let mut min_pass = u64::MAX;

        for (i, task) in self.tasks.iter().enumerate() {
            if task.state == TaskState::Ready && task.pass < min_pass {
                min_pass = task.pass;
                min_idx = i;
            }
        }

        let task = &mut self.tasks[min_idx];
        task.pass += task.stride as u64;
        task.state = TaskState::Running;
        Some(task)
    }
}

/// Global scheduler state
static mut RUN_QUEUES: [RunQueue; crate::kernel::percpu::MAX_CPUS] = {
    const INIT: RunQueue = RunQueue::new();
    [INIT; crate::kernel::percpu::MAX_CPUS]
};

static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Initialize scheduler
pub fn init() {
    // Create idle task for CPU 0
    let idle_task = TaskDesc::new(0, 100);
    unsafe {
        let _ = RUN_QUEUES[0].enqueue(idle_task);
    }
}

/// Start the scheduler
pub fn start() -> ! {
    loop {
        schedule();
        x86_64::instructions::hlt();
    }
}

/// Perform a scheduling decision
pub fn schedule() {
    let cpu_id = crate::kernel::percpu::current_cpu_id() as usize;
    
    unsafe {
        if let Some(task) = RUN_QUEUES[cpu_id].next_task() {
            // Context switch to task
            switch_to_task(task);
        }
    }
}

/// Handle timer tick
pub fn tick() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
    
    // Trigger rescheduling
    schedule();
}

/// Context switch to a task (assembly stub)
fn switch_to_task(task: &TaskDesc) {
    // TODO: Implement register-only context switch in assembly
    // For now, this is a placeholder
}

/// Scheduler errors
#[derive(Debug)]
pub enum SchedulerError {
    QueueFull,
    NoTasksAvailable,
    InvalidTaskId,
}
