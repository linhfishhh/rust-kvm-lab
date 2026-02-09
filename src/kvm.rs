use crate::kvm_bindings::*;
use libc::{
    c_int, close, ioctl, mmap, munmap, open, MAP_ANONYMOUS, MAP_FAILED, MAP_SHARED, O_CLOEXEC,
    O_RDWR, PROT_READ, PROT_WRITE,
};
use std::os::unix::io::RawFd;
use std::ptr;

pub struct Kvm {
    fd: RawFd,
}

impl Kvm {
    pub fn new() -> Result<Self, String> {
        let fd = unsafe {
            open(
                b"/dev/kvm\0".as_ptr() as *const libc::c_char,
                O_RDWR | O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err("Could not open /dev/kvm".to_string());
        }

        let api_version = unsafe { ioctl(fd, KVM_GET_API_VERSION, 0) };
        if api_version != KVM_API_VERSION {
            unsafe { close(fd) };
            return Err(format!(
                "KVM API version mismatch: got {}, expected {}",
                api_version, KVM_API_VERSION
            ));
        }

        Ok(Kvm { fd })
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn create_vm(&self) -> Result<Vm, String> {
        let vm_fd = unsafe { ioctl(self.fd, KVM_CREATE_VM, 0) };
        if vm_fd < 0 {
            return Err("Could not create VM".to_string());
        }
        Ok(Vm { fd: vm_fd })
    }

    pub fn get_vcpu_mmap_size(&self) -> Result<usize, String> {
        let size = unsafe { ioctl(self.fd, KVM_GET_VCPU_MMAP_SIZE, 0) };
        if size < 0 {
            return Err("Could not get VCPU mmap size".to_string());
        }
        Ok(size as usize)
    }
}

impl Drop for Kvm {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

pub struct Vm {
    fd: RawFd,
}

impl Vm {
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn set_user_memory_region(&self, region: &KvmUserspaceMemoryRegion) -> Result<(), String> {
        let ret = unsafe { ioctl(self.fd, KVM_SET_USER_MEMORY_REGION, region) };
        if ret < 0 {
            return Err("Could not set memory region".to_string());
        }
        Ok(())
    }

    pub fn create_vcpu(&self, id: c_int) -> Result<Vcpu, String> {
        let vcpu_fd = unsafe { ioctl(self.fd, KVM_CREATE_VCPU, id) };
        if vcpu_fd < 0 {
            return Err("Could not create VCPU".to_string());
        }
        Ok(Vcpu { fd: vcpu_fd })
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

pub struct Vcpu {
    fd: RawFd,
}

impl Vcpu {
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn get_regs(&self) -> Result<KvmRegs, String> {
        let mut regs = KvmRegs::default();
        let ret = unsafe { ioctl(self.fd, KVM_GET_REGS, &mut regs) };
        if ret < 0 {
            return Err("Could not get registers".to_string());
        }
        Ok(regs)
    }

    pub fn set_regs(&self, regs: &KvmRegs) -> Result<(), String> {
        let ret = unsafe { ioctl(self.fd, KVM_SET_REGS, regs) };
        if ret < 0 {
            return Err("Could not set registers".to_string());
        }
        Ok(())
    }

    pub fn get_sregs(&self) -> Result<KvmSregs, String> {
        let mut sregs = KvmSregs::default();
        let ret = unsafe { ioctl(self.fd, KVM_GET_SREGS, &mut sregs) };
        if ret < 0 {
            return Err("Could not get special registers".to_string());
        }
        Ok(sregs)
    }

    pub fn set_sregs(&self, sregs: &KvmSregs) -> Result<(), String> {
        let ret = unsafe { ioctl(self.fd, KVM_SET_SREGS, sregs) };
        if ret < 0 {
            return Err("Could not set special registers".to_string());
        }
        Ok(())
    }

    pub fn run(&self) -> Result<(), String> {
        let ret = unsafe { ioctl(self.fd, KVM_RUN, 0) };
        if ret < 0 {
            return Err("KVM_RUN failed".to_string());
        }
        Ok(())
    }

    pub fn map_run(&self, size: usize) -> Result<*mut KvmRun, String> {
        let run = unsafe {
            mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                self.fd,
                0,
            )
        };
        if run == MAP_FAILED {
            return Err("Could not mmap VCPU".to_string());
        }
        Ok(run as *mut KvmRun)
    }
}

impl Drop for Vcpu {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

pub struct GuestMemory {
    ptr: *mut u8,
    size: usize,
}

impl GuestMemory {
    pub fn new(size: usize) -> Result<Self, String> {
        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == MAP_FAILED {
            return Err("Could not allocate guest memory".to_string());
        }
        Ok(GuestMemory {
            ptr: ptr as *mut u8,
            size,
        })
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }
}

impl Drop for GuestMemory {
    fn drop(&mut self) {
        unsafe { munmap(self.ptr as *mut _, self.size) };
    }
}

pub struct VcpuRun {
    ptr: *mut KvmRun,
    size: usize,
}

impl VcpuRun {
    pub fn new(ptr: *mut KvmRun, size: usize) -> Self {
        VcpuRun { ptr, size }
    }

    pub fn as_ref(&self) -> &KvmRun {
        unsafe { &*self.ptr }
    }
}

impl Drop for VcpuRun {
    fn drop(&mut self) {
        unsafe { munmap(self.ptr as *mut _, self.size) };
    }
}
