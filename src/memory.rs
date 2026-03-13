// Bootloader-Agnostic HAL
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryKind {
    Usable,
    Reserved,
    Kernel,
    Other,
}

#[derive(Clone, Copy)]
pub struct MemoryRegion {
    pub base: u64,
    pub pages: usize,
    pub kind: MemoryKind,
}

pub struct BootMemoryMap {
    pub regions: [MemoryRegion; 64],
    pub count: usize,
}

// Physical Allocator
pub struct PhysicalAllocator {
    pub next_free_frame: u64,
    pub limit: u64,
}

impl PhysicalAllocator {
    pub fn init(map: &BootMemoryMap) -> Self {
        // Find the first large usable memory region to give to our physical allocator
        for i in 0..map.count {
            let r = &map.regions[i];
            if r.kind == MemoryKind::Usable && r.pages > 512 {
                return Self {
                    next_free_frame: r.base,
                    limit: r.base + (r.pages as u64 * 4096),
                };
            }
        }
        Self {
            next_free_frame: 0,
            limit: 0,
        }
    }

    /// Allocates a fast 4KB physical frame
    pub fn alloc_frame(&mut self) -> Option<u64> {
        if self.next_free_frame + 4096 <= self.limit {
            let frame = self.next_free_frame;
            self.next_free_frame += 4096;
            Some(frame)
        } else {
            None
        }
    }
}

// Virtual Bump Allocator
pub struct VirtualBumpAllocator {
    pub current_vaddr: u64,
}

impl VirtualBumpAllocator {
    pub const fn new(start_addr: u64) -> Self {
        Self {
            current_vaddr: start_addr,
        }
    }

    /// Instantly reserves a massive virtual window (0 physical RAM cost until page fault)
    pub fn reserve_window(&mut self, size_bytes: u64) -> u64 {
        let addr = self.current_vaddr;
        self.current_vaddr += size_bytes;
        addr
    }
}
