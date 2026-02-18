//! Tag-based file system (TagFS)

use arrayvec::ArrayVec;
use core::hash::{Hash, Hasher};

/// Object metadata (12 bytes packed)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ObjectMeta {
    /// Object ID
    pub id: u64,
    /// Size in bytes
    pub size: u32,
}

impl ObjectMeta {
    pub const fn new(id: u64, size: u32) -> Self {
        Self { id, size }
    }
}

/// Tag structure
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag {
    data: [u8; 32],
    len: u8,
}

impl Tag {
    pub fn new(s: &str) -> Self {
        let mut data = [0u8; 32];
        let len = s.len().min(32);
        data[..len].copy_from_slice(&s.as_bytes()[..len]);
        Self { data, len: len as u8 }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len as usize]).unwrap_or("")
    }
}

/// Cuckoo hash table for tag index
const HASH_TABLE_SIZE: usize = 4096;

pub struct TagIndex {
    table1: [Option<(Tag, u64)>; HASH_TABLE_SIZE],
    table2: [Option<(Tag, u64)>; HASH_TABLE_SIZE],
}

impl TagIndex {
    pub const fn new() -> Self {
        Self {
            table1: [None; HASH_TABLE_SIZE],
            table2: [None; HASH_TABLE_SIZE],
        }
    }

    fn hash1(&self, tag: &Tag) -> usize {
        let mut h = 0u64;
        for &b in &tag.data[..tag.len as usize] {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        (h as usize) % HASH_TABLE_SIZE
    }

    fn hash2(&self, tag: &Tag) -> usize {
        let mut h = 5381u64;
        for &b in &tag.data[..tag.len as usize] {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        (h as usize) % HASH_TABLE_SIZE
    }

    pub fn insert(&mut self, tag: Tag, object_id: u64) -> Result<(), TagFsError> {
        let idx1 = self.hash1(&tag);
        if self.table1[idx1].is_none() {
            self.table1[idx1] = Some((tag, object_id));
            return Ok(());
        }

        let idx2 = self.hash2(&tag);
        if self.table2[idx2].is_none() {
            self.table2[idx2] = Some((tag, object_id));
            return Ok(());
        }

        // Cuckoo eviction would go here
        Err(TagFsError::HashTableFull)
    }

    pub fn lookup(&self, tag: &Tag) -> Option<u64> {
        let idx1 = self.hash1(tag);
        if let Some((t, oid)) = &self.table1[idx1] {
            if t == tag {
                return Some(*oid);
            }
        }

        let idx2 = self.hash2(tag);
        if let Some((t, oid)) = &self.table2[idx2] {
            if t == tag {
                return Some(*oid);
            }
        }

        None
    }
}

/// Global TagFS state
static mut TAG_INDEX: TagIndex = TagIndex::new();
static mut NEXT_OBJECT_ID: u64 = 1;

/// Initialize TagFS
pub fn init() {
    // TagFS initialized
}

/// Create a new object with tags
pub fn tagfs_create(tags: &[Tag], data: &[u8]) -> Result<u64, TagFsError> {
    unsafe {
        let object_id = NEXT_OBJECT_ID;
        NEXT_OBJECT_ID += 1;

        for tag in tags {
            TAG_INDEX.insert(*tag, object_id)?;
        }

        // TODO: Store actual data
        Ok(object_id)
    }
}

/// Query objects by tag
pub fn tagfs_query(tag: &Tag) -> Option<u64> {
    unsafe { TAG_INDEX.lookup(tag) }
}

/// Add tag to object
pub fn tagfs_add_tag(object_id: u64, tag: Tag) -> Result<(), TagFsError> {
    unsafe { TAG_INDEX.insert(tag, object_id) }
}

/// TagFS errors
#[derive(Debug)]
pub enum TagFsError {
    HashTableFull,
    ObjectNotFound,
    InvalidTag,
    StorageFull,
}
