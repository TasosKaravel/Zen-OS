//! Global edge-case registry (12-byte packed structs)

/// Maximum number of edge cases that can be registered
pub const MAX_EDGE_CASES: usize = 1024;

/// Edge case entry (12 bytes packed)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct EdgeCase {
    /// Error code
    pub code: u32,
    /// Source location (file ID)
    pub file_id: u16,
    /// Source location (line number)
    pub line: u16,
    /// Timestamp (ticks since boot)
    pub timestamp: u32,
}

impl EdgeCase {
    /// Create a new edge case entry
    pub const fn new(code: u32, file_id: u16, line: u16, timestamp: u32) -> Self {
        Self {
            code,
            file_id,
            line,
            timestamp,
        }
    }
}

/// Global edge case registry
pub struct EdgeRegistry {
    entries: [EdgeCase; MAX_EDGE_CASES],
    count: usize,
}

impl EdgeRegistry {
    /// Create a new empty registry
    pub const fn new() -> Self {
        Self {
            entries: [EdgeCase::new(0, 0, 0, 0); MAX_EDGE_CASES],
            count: 0,
        }
    }

    /// Register a new edge case
    pub fn register(&mut self, code: u32, file_id: u16, line: u16, timestamp: u32) {
        if self.count < MAX_EDGE_CASES {
            self.entries[self.count] = EdgeCase::new(code, file_id, line, timestamp);
            self.count += 1;
        }
    }

    /// Get all registered edge cases
    pub fn entries(&self) -> &[EdgeCase] {
        &self.entries[..self.count]
    }
}

/// Global edge case registry instance
static mut EDGE_REGISTRY: EdgeRegistry = EdgeRegistry::new();

/// Register an edge case globally
pub fn register_edge_case(code: u32, file_id: u16, line: u16, timestamp: u32) {
    unsafe {
        EDGE_REGISTRY.register(code, file_id, line, timestamp);
    }
}

/// Get all registered edge cases
pub fn get_edge_cases() -> &'static [EdgeCase] {
    unsafe { EDGE_REGISTRY.entries() }
}
