use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const MAX_CPUS: usize = 256;
pub const BUFFER_SIZE: usize = 4096;

pub static PANIC_FLAG: AtomicBool = AtomicBool::new(false);

pub struct CpuBuffer {
    pub head: AtomicUsize, // written by cpu
    pub tail: AtomicUsize, // read by consumer
    pub overflow_count: AtomicUsize,
    pub data: UnsafeCell<[u8; BUFFER_SIZE]>,
}

// safely share across threads.
// manage data via atomic indices
unsafe impl Sync for CpuBuffer {}

impl CpuBuffer {
    pub const fn new() -> Self {
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            overflow_count: AtomicUsize::new(0),
            data: UnsafeCell::new([0; BUFFER_SIZE]),
        }
    }

    pub fn push_byte(&self, byte: u8) {
        if PANIC_FLAG.load(Ordering::Relaxed) {
            return;
            // OS panicking, freeze the buffer
        }

        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let next_head = (head + 1) % BUFFER_SIZE;

        // overflow: when buffer is full,
        // force tail fwd to overwrite oldest
        if next_head == tail {
            self.tail.store((tail + 1) % BUFFER_SIZE, Ordering::Release);
            self.overflow_count.fetch_add(1, Ordering::Relaxed);
        }

        unsafe {
            (*self.data.get())[head] = byte;
        }

        // Release ordering
        // guarantees byte is written before head updates
        self.head.store(next_head, Ordering::Release);
    }

    pub fn pop_byte(&self) -> Option<u8> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if head == tail {
            return None; // Buffer is empty
        }
        let byte = unsafe { (*self.data.get())[tail] };
        self.tail.store((tail + 1) % BUFFER_SIZE, Ordering::Release);
        Some(byte)
    }
}

pub struct KernelLogger;

impl core::fmt::Write for KernelLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let cpu_id = crate::cpu::apic_id() % crate::logger::MAX_CPUS;
        let buffer = &LOG_BUFFERS[cpu_id];
        for byte in s.bytes() {
            buffer.push_byte(byte);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut $crate::logger::KernelLogger, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// The manual consumer: drains the buffer to the serial port
pub fn flush() {
    while let Some(byte) = LOG_BUFFERS[0].pop_byte() {
        crate::serial::write_byte(byte);
    }
}

// The static boot-time reserved array for all CPUs
pub static LOG_BUFFERS: [CpuBuffer; MAX_CPUS] = {
    const INIT: CpuBuffer = CpuBuffer::new();
    [INIT; MAX_CPUS]
};
