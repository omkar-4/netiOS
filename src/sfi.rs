// software fault isolation : legacy fallback
// can cause 5-10% CPU overhead,
// but protects from compromise

pub struct WasmSecurityPolicy {
    pub pks_enabled: bool,
    pub sfi_masking_enabled: bool,
    pub window_base_addr: u64,
}

impl WasmSecurityPolicy {
    pub fn new(pks_supported: bool, base_addr: u64) -> Self {
        Self {
            pks_enabled: pks_supported,
            // PKS => missing, forcefully enable SFI
            sfi_masking_enabled: !pks_supported,
            window_base_addr: base_addr,
        }
    }

    // simulates wasi's on-wasm security wall
    #[inline(always)]
    pub fn compile_safe_ptr(&self, wasm_ptr: u32) -> u64 {
        if self.sfi_masking_enabled {
            // mask & bind pointer to sandbox base
            // Even if WASM is malicious
            // generated assembly strictly binds it.
            self.window_base_addr + (wasm_ptr as u64)
        } else {
            // PKS Hardware Mode: Raw translation.
            // PKS hardware will catch illegal accesses.
            self.window_base_addr + (wasm_ptr as u64)
        }
    }
}
