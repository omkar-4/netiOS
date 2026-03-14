use core::arch::asm;

#[repr(align(16))]
pub struct InterruptStack(pub [u8; 65536]);

pub static mut INTERRUPT_STACK: InterruptStack = InterruptStack([0; 65536]);

#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved1: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    reserved2: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    reserved3: u64,
    reserved4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            reserved1: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved3: 0,
            reserved4: 0,
            iomap_base: core::mem::size_of::<TaskStateSegment>() as u16,
        }
    }
}

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

pub fn init() {
    unsafe {
        let stack_top = core::ptr::addr_of!(INTERRUPT_STACK) as u64 + 65536;
        TSS.rsp0 = stack_top;
        TSS.ist1 = stack_top;
    }
}
