# PMM Verification & Testing Checklist

## 1. Bootstrapping and Memory Safety
- [ ] Verify `Total Frames` is calculated by dividing the *highest physical address* by 4096, not just the sum of usable memory (prevents out-of-bounds indexing).
- [ ] Confirm the entire bitmap is initialized to `0xFF` before any usable regions are unlocked.
- [ ] Confirm the exact physical memory span where the bitmap `&mut [u8]` is stored is explicitly marked as `1` (used) to prevent self-allocation.

## 2. Hardware-Accelerated Math
- [ ] Verify `alloc_frame` casts the bitmap to `&[u64]` to check 64 frames at a time.
- [ ] Verify `.trailing_ones()` (or equivalent bitwise intrinsic) is used on the `u64` chunk to find the specific free bit without a 0..64 loop.
- [ ] Verify `Address % 4096 == 0` is strictly asserted inside `free_frame`.

## 3. Hint Caching Logic
- [ ] Confirm `next_free_frame_hint` is updated to `allocated_index + 1` after an allocation.
- [ ] Confirm `next_free_frame_hint` is updated to `freed_index` during deallocation *only if* `freed_index < next_free_frame_hint`.

## 4. Immediate Stress Testing (Post-Init)
- [ ] **Null Trap Test:** Call `alloc_frame`. Assert the returned address is strictly greater than `0x0`.
- [ ] **Alignment Test:** Call `alloc_frame`. Assert `addr % 4096 == 0`.
- [ ] **Double Free Test:** Call `free_frame(addr)`. Call `free_frame(addr)` again. Verify the OS panics or safely rejects the double-free.
- [ ] **Reallocation Test:** Call `alloc_frame`, then `free_frame(addr)`, then `alloc_frame`. Assert the exact same physical address is returned due to hint caching.
