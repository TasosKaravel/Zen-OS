//! Lazy allocation pool for on-demand structures

/// Maximum number of slots in the lazy pool
pub const MAX_POOL_SLOTS: usize = 256;

/// Slot state
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Free,
    Allocated,
}

/// Generic lazy pool slot
#[derive(Clone, Copy)]
pub struct PoolSlot<T> {
    state: SlotState,
    data: Option<T>,
}

impl<T> PoolSlot<T> {
    /// Create a new free slot
    pub const fn new() -> Self {
        Self {
            state: SlotState::Free,
            data: None,
        }
    }

    /// Allocate this slot with data
    pub fn allocate(&mut self, data: T) -> Result<(), PoolError> {
        if self.state == SlotState::Free {
            self.data = Some(data);
            self.state = SlotState::Allocated;
            Ok(())
        } else {
            Err(PoolError::SlotInUse)
        }
    }

    /// Free this slot
    pub fn free(&mut self) -> Option<T> {
        self.state = SlotState::Free;
        self.data.take()
    }

    /// Get reference to data if allocated
    pub fn get(&self) -> Option<&T> {
        self.data.as_ref()
    }

    /// Get mutable reference to data if allocated
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }
}

/// Lazy allocation pool
pub struct LazyPool<T, const N: usize> {
    slots: [PoolSlot<T>; N],
}

impl<T, const N: usize> LazyPool<T, N>
where
    T: Copy,
{
    /// Create a new empty pool
    pub const fn new() -> Self {
        Self {
            slots: [const { PoolSlot::new() }; N],
        }
    }

    /// Allocate a slot with data
    pub fn allocate(&mut self, data: T) -> Result<usize, PoolError> {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.state == SlotState::Free {
                slot.allocate(data)?;
                return Ok(i);
            }
        }
        Err(PoolError::PoolFull)
    }

    /// Free a slot by index
    pub fn free(&mut self, index: usize) -> Result<T, PoolError> {
        if index >= N {
            return Err(PoolError::InvalidIndex);
        }
        self.slots[index].free().ok_or(PoolError::SlotNotAllocated)
    }

    /// Get reference to slot data
    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index)?.get()
    }

    /// Get mutable reference to slot data
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index)?.get_mut()
    }
}

/// Pool allocation errors
#[derive(Debug)]
pub enum PoolError {
    PoolFull,
    SlotInUse,
    SlotNotAllocated,
    InvalidIndex,
}
