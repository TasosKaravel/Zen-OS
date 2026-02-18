//! Memory management subsystem

use bootloader::BootInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};
use spin::Mutex;
use lazy_static::lazy_static;

/// Global frame allocator
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

/// Global page table mapper
pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);

/// Initialize memory management
pub fn init(boot_info: &'static BootInfo) {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let level_4_table = unsafe { active_level_4_table(phys_mem_offset) };
    let mapper = unsafe { OffsetPageTable::new(level_4_table, phys_mem_offset) };
    
    *MAPPER.lock() = Some(mapper);
    
    let frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    *FRAME_ALLOCATOR.lock() = Some(frame_allocator);
}

/// Get the active level 4 page table
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Frame allocator that uses bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a new frame allocator from the bootloader memory map
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Allocate a physical frame
pub fn allocate_frame() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.lock().as_mut()?.allocate_frame()
}

/// Map a virtual page to a physical frame
pub fn map_page(page: Page, frame: PhysFrame) -> Result<(), MapError> {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let mut mapper = MAPPER.lock();
    let mapper = mapper.as_mut().ok_or(MapError::MapperNotInitialized)?;
    
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.as_mut().ok_or(MapError::AllocatorNotInitialized)?;

    unsafe {
        mapper
            .map_to(page, frame, Flags::PRESENT | Flags::WRITABLE, frame_allocator)
            .map_err(|_| MapError::MapFailed)?
            .flush();
    }

    Ok(())
}

/// Memory mapping errors
#[derive(Debug)]
pub enum MapError {
    MapperNotInitialized,
    AllocatorNotInitialized,
    MapFailed,
}
