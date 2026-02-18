//! Zero-copy IPC with lock-free ring buffers

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use heapless::Vec;

/// Maximum message size
pub const MAX_MESSAGE_SIZE: usize = 4096;

/// Maximum number of IPC channels
pub const MAX_IPC_CHANNELS: usize = 1024;

/// Ring buffer size (must be power of 2)
pub const RING_BUFFER_SIZE: usize = 16;

/// IPC message header
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MessageHeader {
    /// Message ID
    pub id: u64,
    /// Sender process ID
    pub sender: u32,
    /// Receiver process ID
    pub receiver: u32,
    /// Message length
    pub length: u32,
    /// Message type
    pub msg_type: u32,
}

/// Lock-free ring buffer for IPC
#[repr(C, align(64))]
pub struct RingBuffer {
    /// Write index
    write_idx: AtomicUsize,
    /// Read index
    read_idx: AtomicUsize,
    /// Message buffer
    messages: [Option<MessageHeader>; RING_BUFFER_SIZE],
    /// Data buffer
    data: [[u8; MAX_MESSAGE_SIZE]; RING_BUFFER_SIZE],
}

impl RingBuffer {
    /// Create a new ring buffer
    pub const fn new() -> Self {
        Self {
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
            messages: [None; RING_BUFFER_SIZE],
            data: [[0; MAX_MESSAGE_SIZE]; RING_BUFFER_SIZE],
        }
    }

    /// Send a message (zero-copy)
    pub fn send(&mut self, header: MessageHeader, data: &[u8]) -> Result<(), IpcError> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(IpcError::MessageTooLarge);
        }

        let write_idx = self.write_idx.load(Ordering::Acquire);
        let read_idx = self.read_idx.load(Ordering::Acquire);

        // Check if buffer is full
        if (write_idx + 1) % RING_BUFFER_SIZE == read_idx {
            return Err(IpcError::BufferFull);
        }

        // Write message
        self.messages[write_idx] = Some(header);
        self.data[write_idx][..data.len()].copy_from_slice(data);

        // Update write index
        self.write_idx.store((write_idx + 1) % RING_BUFFER_SIZE, Ordering::Release);

        Ok(())
    }

    /// Receive a message (zero-copy)
    pub fn recv(&mut self) -> Result<(MessageHeader, &[u8]), IpcError> {
        let read_idx = self.read_idx.load(Ordering::Acquire);
        let write_idx = self.write_idx.load(Ordering::Acquire);

        // Check if buffer is empty
        if read_idx == write_idx {
            return Err(IpcError::BufferEmpty);
        }

        // Read message
        let header = self.messages[read_idx].ok_or(IpcError::InvalidMessage)?;
        let data = &self.data[read_idx][..header.length as usize];

        // Update read index
        self.read_idx.store((read_idx + 1) % RING_BUFFER_SIZE, Ordering::Release);

        Ok((header, data))
    }
}

/// Global IPC channel table
static mut IPC_CHANNELS: [Option<RingBuffer>; MAX_IPC_CHANNELS] = [const { None }; MAX_IPC_CHANNELS];
static NEXT_CHANNEL_ID: AtomicU64 = AtomicU64::new(0);

/// Initialize IPC subsystem
pub fn init() {
    // IPC channels are created on demand
}

/// Create a new IPC channel
pub fn create_channel() -> Result<u64, IpcError> {
    let channel_id = NEXT_CHANNEL_ID.fetch_add(1, Ordering::Relaxed);
    
    if channel_id >= MAX_IPC_CHANNELS as u64 {
        return Err(IpcError::TooManyChannels);
    }

    unsafe {
        IPC_CHANNELS[channel_id as usize] = Some(RingBuffer::new());
    }

    Ok(channel_id)
}

/// Send message via IPC
pub fn msg_send(channel_id: u64, header: MessageHeader, data: &[u8]) -> Result<(), IpcError> {
    if channel_id >= MAX_IPC_CHANNELS as u64 {
        return Err(IpcError::InvalidChannel);
    }

    unsafe {
        let channel = IPC_CHANNELS[channel_id as usize]
            .as_mut()
            .ok_or(IpcError::InvalidChannel)?;
        
        // Check capability token
        crate::capability::check_ipc_permission(header.sender, channel_id)?;
        
        channel.send(header, data)
    }
}

/// Receive message via IPC
pub fn msg_recv(channel_id: u64) -> Result<(MessageHeader, &'static [u8]), IpcError> {
    if channel_id >= MAX_IPC_CHANNELS as u64 {
        return Err(IpcError::InvalidChannel);
    }

    unsafe {
        let channel = IPC_CHANNELS[channel_id as usize]
            .as_mut()
            .ok_or(IpcError::InvalidChannel)?;
        
        channel.recv()
    }
}

/// Poll for messages
pub fn msg_poll(channel_id: u64) -> Result<bool, IpcError> {
    if channel_id >= MAX_IPC_CHANNELS as u64 {
        return Err(IpcError::InvalidChannel);
    }

    unsafe {
        let channel = IPC_CHANNELS[channel_id as usize]
            .as_ref()
            .ok_or(IpcError::InvalidChannel)?;
        
        let read_idx = channel.read_idx.load(Ordering::Acquire);
        let write_idx = channel.write_idx.load(Ordering::Acquire);
        
        Ok(read_idx != write_idx)
    }
}

/// IPC errors
#[derive(Debug)]
pub enum IpcError {
    BufferFull,
    BufferEmpty,
    MessageTooLarge,
    InvalidMessage,
    InvalidChannel,
    TooManyChannels,
    PermissionDenied,
}

impl From<crate::capability::CapabilityError> for IpcError {
    fn from(_: crate::capability::CapabilityError) -> Self {
        IpcError::PermissionDenied
    }
}
