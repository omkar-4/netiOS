use core::cell::UnsafeCell;
use core::ptr::write_bytes;
use core::slice;

const FRAME_SIZE: u64 = 4096;

#[derive(Clone, Copy, PartialEq)]
pub enum RegionKind {
    Usable,
    Reclaimable,
    Reserved,
}

#[derive(Clone, Copy)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub kind: RegionKind,
}

struct BitmapAllocatorInner {
    bitmap: *mut u8,
    bitmap_len: usize,
    total_frames: usize,
    free_frames_count: usize,
    next_free_frame_hint: usize,
}

struct SyncBitmapAllocator(UnsafeCell<BitmapAllocatorInner>);
unsafe impl Sync for SyncBitmapAllocator {}

static ALLOCATOR: SyncBitmapAllocator = SyncBitmapAllocator(UnsafeCell::new(BitmapAllocatorInner {
    bitmap: core::ptr::null_mut(),
    bitmap_len: 0,
    total_frames: 0,
    free_frames_count: 0,
    next_free_frame_hint: 0,
}));

fn bitmap_slice(inner: &BitmapAllocatorInner) -> &[u8] {
    unsafe { slice::from_raw_parts(inner.bitmap, inner.bitmap_len) }
}

fn bitmap_slice_mut(inner: &mut BitmapAllocatorInner) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut(inner.bitmap, inner.bitmap_len) }
}

fn set_bit(inner: &mut BitmapAllocatorInner, index: usize) {
    let byte_index = index / 8;
    let bit_offset = index % 8;
    let bm = bitmap_slice_mut(inner);
    bm[byte_index] |= 1 << bit_offset;
}

fn clear_bit(inner: &mut BitmapAllocatorInner, index: usize) {
    let byte_index = index / 8;
    let bit_offset = index % 8;
    let bm = bitmap_slice_mut(inner);
    bm[byte_index] &= !(1 << bit_offset);
}

fn test_bit(inner: &BitmapAllocatorInner, index: usize) -> bool {
    let byte_index = index / 8;
    let bit_offset = index % 8;
    let bm = bitmap_slice(inner);
    (bm[byte_index] >> bit_offset) & 1 == 1
}

pub fn init(regions: &[MemoryRegion], hhdm_offset: u64) {
    let inner = unsafe { &mut *ALLOCATOR.0.get() };

    let mut highest_address: u64 = 0;
    for region in regions {
        let region_end = region.base + region.length;
        if region_end > highest_address {
            highest_address = region_end;
        }
    }

    let total_frames = (highest_address / FRAME_SIZE) as usize;
    let bitmap_bytes = (total_frames + 7) / 8;

    let mut bitmap_phys: u64 = 0;
    let mut bitmap_placed = false;

    for region in regions {
        if region.kind != RegionKind::Usable {
            continue;
        }
        if region.length >= bitmap_bytes as u64 {
            bitmap_phys = region.base;
            bitmap_placed = true;
            break;
        }
    }

    if !bitmap_placed {
        panic!();
    }

    let bitmap_virt = (bitmap_phys + hhdm_offset) as *mut u8;

    unsafe {
        write_bytes(bitmap_virt, 0xFF, bitmap_bytes);
    }

    inner.bitmap = bitmap_virt;
    inner.bitmap_len = bitmap_bytes;
    inner.total_frames = total_frames;
    inner.free_frames_count = 0;
    inner.next_free_frame_hint = 1;

    for region in regions {
        let is_free = region.kind == RegionKind::Usable
            || region.kind == RegionKind::Reclaimable;

        if !is_free {
            continue;
        }

        let start_frame = (region.base / FRAME_SIZE) as usize;
        let end_frame = ((region.base + region.length) / FRAME_SIZE) as usize;

        for frame_idx in start_frame..end_frame {
            if frame_idx == 0 {
                continue;
            }
            if frame_idx >= total_frames {
                break;
            }
            clear_bit(inner, frame_idx);
            inner.free_frames_count += 1;
        }
    }

    let bitmap_start_frame = (bitmap_phys / FRAME_SIZE) as usize;
    let bitmap_end_frame = ((bitmap_phys + bitmap_bytes as u64 + FRAME_SIZE - 1) / FRAME_SIZE) as usize;
    for frame_idx in bitmap_start_frame..bitmap_end_frame {
        if !test_bit(inner, frame_idx) {
            set_bit(inner, frame_idx);
            if inner.free_frames_count > 0 {
                inner.free_frames_count -= 1;
            }
        }
    }

    set_bit(inner, 0);
}

pub fn alloc_frame() -> u64 {
    let inner = unsafe { &mut *ALLOCATOR.0.get() };

    if inner.free_frames_count == 0 {
        panic!();
    }

    let bm = bitmap_slice(inner);
    let bm_ptr = bm.as_ptr() as *const u64;
    let chunks = inner.bitmap_len / 8;
    let start_chunk = inner.next_free_frame_hint / 64;

    for chunk_idx in start_chunk..chunks {
        let chunk = unsafe { core::ptr::read_unaligned(bm_ptr.add(chunk_idx)) };
        if chunk == 0xFFFF_FFFF_FFFF_FFFF {
            continue;
        }

        let bit_in_chunk = chunk.trailing_ones() as usize;
        let global_bit = chunk_idx * 64 + bit_in_chunk;

        if global_bit == 0 || global_bit >= inner.total_frames {
            continue;
        }

        set_bit(inner, global_bit);
        inner.free_frames_count -= 1;
        inner.next_free_frame_hint = global_bit + 1;

        return (global_bit as u64) * FRAME_SIZE;
    }

    for chunk_idx in 0..start_chunk {
        let chunk = unsafe { core::ptr::read_unaligned(bm_ptr.add(chunk_idx)) };
        if chunk == 0xFFFF_FFFF_FFFF_FFFF {
            continue;
        }

        let bit_in_chunk = chunk.trailing_ones() as usize;
        let global_bit = chunk_idx * 64 + bit_in_chunk;

        if global_bit == 0 || global_bit >= inner.total_frames {
            continue;
        }

        set_bit(inner, global_bit);
        inner.free_frames_count -= 1;
        inner.next_free_frame_hint = global_bit + 1;

        return (global_bit as u64) * FRAME_SIZE;
    }

    let remainder_start = chunks * 8;
    for byte_idx in remainder_start..inner.bitmap_len {
        let byte = bm[byte_idx];
        if byte == 0xFF {
            continue;
        }
        for bit in 0..8u8 {
            if (byte >> bit) & 1 == 0 {
                let global_bit = byte_idx * 8 + bit as usize;
                if global_bit == 0 || global_bit >= inner.total_frames {
                    continue;
                }
                set_bit(inner, global_bit);
                inner.free_frames_count -= 1;
                inner.next_free_frame_hint = global_bit + 1;
                return (global_bit as u64) * FRAME_SIZE;
            }
        }
    }

    panic!();
}

pub fn free_frame(addr: u64) {
    assert!(addr % FRAME_SIZE == 0);
    assert!(addr > 0);

    let inner = unsafe { &mut *ALLOCATOR.0.get() };
    let frame_index = (addr / FRAME_SIZE) as usize;

    assert!(frame_index < inner.total_frames);
    assert!(test_bit(inner, frame_index));

    clear_bit(inner, frame_index);
    inner.free_frames_count += 1;

    if frame_index < inner.next_free_frame_hint {
        inner.next_free_frame_hint = frame_index;
    }
}

pub fn free_frames_count() -> usize {
    let inner = unsafe { &*ALLOCATOR.0.get() };
    inner.free_frames_count
}

pub fn total_frames() -> usize {
    let inner = unsafe { &*ALLOCATOR.0.get() };
    inner.total_frames
}
