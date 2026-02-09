use libc::{c_int, c_ulong};

pub const KVM_API_VERSION: c_int = 12;

// KVM ioctls
pub const KVMIO: c_ulong = 0xAE;

#[inline]
const fn _IO(ty: c_ulong, nr: c_ulong) -> c_ulong {
    (0 << 30) | (ty << 8) | nr
}

#[inline]
const fn _IOR<T>(ty: c_ulong, nr: c_ulong) -> c_ulong {
    (2 << 30) | ((std::mem::size_of::<T>() as c_ulong) << 16) | (ty << 8) | nr
}

#[inline]
const fn _IOW<T>(ty: c_ulong, nr: c_ulong) -> c_ulong {
    (1 << 30) | ((std::mem::size_of::<T>() as c_ulong) << 16) | (ty << 8) | nr
}

pub const KVM_GET_API_VERSION: c_ulong = _IO(KVMIO, 0x00);
pub const KVM_CREATE_VM: c_ulong = _IO(KVMIO, 0x01);
pub const KVM_GET_VCPU_MMAP_SIZE: c_ulong = _IO(KVMIO, 0x04);
pub const KVM_CREATE_VCPU: c_ulong = _IO(KVMIO, 0x41);
pub const KVM_SET_USER_MEMORY_REGION: c_ulong = _IOW::<KvmUserspaceMemoryRegion>(KVMIO, 0x46);
pub const KVM_RUN: c_ulong = _IO(KVMIO, 0x80);
pub const KVM_GET_REGS: c_ulong = _IOR::<KvmRegs>(KVMIO, 0x81);
pub const KVM_SET_REGS: c_ulong = _IOW::<KvmRegs>(KVMIO, 0x82);
pub const KVM_GET_SREGS: c_ulong = _IOR::<KvmSregs>(KVMIO, 0x83);
pub const KVM_SET_SREGS: c_ulong = _IOW::<KvmSregs>(KVMIO, 0x84);

// Exit reasons
pub const KVM_EXIT_UNKNOWN: u32 = 0;
pub const KVM_EXIT_EXCEPTION: u32 = 1;
pub const KVM_EXIT_IO: u32 = 2;
pub const KVM_EXIT_HYPERCALL: u32 = 3;
pub const KVM_EXIT_DEBUG: u32 = 4;
pub const KVM_EXIT_HLT: u32 = 5;
pub const KVM_EXIT_MMIO: u32 = 6;
pub const KVM_EXIT_IRQ_WINDOW_OPEN: u32 = 7;
pub const KVM_EXIT_SHUTDOWN: u32 = 8;
pub const KVM_EXIT_FAIL_ENTRY: u32 = 9;
pub const KVM_EXIT_INTR: u32 = 10;
pub const KVM_EXIT_SET_TPR: u32 = 11;
pub const KVM_EXIT_TPR_ACCESS: u32 = 12;
pub const KVM_EXIT_S390_SIEIC: u32 = 13;
pub const KVM_EXIT_S390_RESET: u32 = 14;
pub const KVM_EXIT_DCR: u32 = 15;
pub const KVM_EXIT_NMI: u32 = 16;
pub const KVM_EXIT_INTERNAL_ERROR: u32 = 17;

pub const KVM_EXIT_IO_IN: u8 = 0;
pub const KVM_EXIT_IO_OUT: u8 = 1;

#[repr(C)]
#[derive(Default)]
pub struct KvmUserspaceMemoryRegion {
    pub slot: u32,
    pub flags: u32,
    pub guest_phys_addr: u64,
    pub memory_size: u64,
    pub userspace_addr: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct KvmRegs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct KvmSegment {
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub type_: u8,
    pub present: u8,
    pub dpl: u8,
    pub db: u8,
    pub s: u8,
    pub l: u8,
    pub g: u8,
    pub avl: u8,
    pub unusable: u8,
    pub padding: u8,
}

#[repr(C)]
#[derive(Default)]
pub struct KvmDtable {
    pub base: u64,
    pub limit: u16,
    pub padding: [u16; 3],
}

#[repr(C)]
#[derive(Default)]
pub struct KvmSregs {
    pub cs: KvmSegment,
    pub ds: KvmSegment,
    pub es: KvmSegment,
    pub fs: KvmSegment,
    pub gs: KvmSegment,
    pub ss: KvmSegment,
    pub tr: KvmSegment,
    pub ldt: KvmSegment,
    pub gdt: KvmDtable,
    pub idt: KvmDtable,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub apic_base: u64,
    pub interrupt_bitmap: [u64; 4],
}

#[repr(C)]
pub struct KvmRun {
    pub request_interrupt_window: u8,
    pub immediate_exit: u8,
    pub padding1: [u8; 6],
    pub exit_reason: u32,
    pub ready_for_interrupt_injection: u8,
    pub if_flag: u8,
    pub flags: u16,
    pub cr8: u64,
    pub apic_base: u64,
    pub exit_data: KvmRunExit,
}

#[repr(C)]
pub union KvmRunExit {
    pub hw: KvmRunHw,
    pub fail_entry: KvmRunFailEntry,
    pub io: KvmRunIo,
    pub internal: KvmRunInternal,
    pub padding: [u8; 256],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KvmRunHw {
    pub hardware_exit_reason: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KvmRunFailEntry {
    pub hardware_entry_failure_reason: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KvmRunIo {
    pub direction: u8,
    pub size: u8,
    pub port: u16,
    pub count: u32,
    pub data_offset: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KvmRunInternal {
    pub suberror: u32,
    pub ndata: u32,
    pub data: [u64; 16],
}
