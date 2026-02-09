mod kvm;
mod kvm_bindings;
mod utils;

use kvm::{GuestMemory, Kvm, VcpuRun};
use kvm_bindings::*;
use utils::exit_reason_name;

fn main() -> Result<(), String> {
    println!("=== Creating a minimal KVM virtual machine ===");

    // Step 1: Open KVM
    let kvm = Kvm::new()?;
    println!("✓ Opened /dev/kvm (fd: {})", kvm.fd());
    println!("✓ KVM API version: {}", KVM_API_VERSION);

    // Step 2: Create VM
    let vm = kvm.create_vm()?;
    println!("✓ Created VM (fd: {})", vm.fd());

    // Step 3: Allocate guest memory
    let mem_size = 0x1000; // 4KB
    let mut guest_memory = GuestMemory::new(mem_size)?;
    println!(
        "✓ Allocated guest memory: {:p} (size: {} bytes)",
        guest_memory.as_ptr(),
        mem_size
    );

    // Step 4: Put machine code in memory
    let code = guest_memory.as_slice_mut();
    code[0] = 0xba; // mov $0x8a00, %dx
    code[1] = 0x00;
    code[2] = 0x8a;
    code[3] = 0xb0; // mov $'H', %al
    code[4] = 0x48;
    code[5] = 0xee; // out %al, (%dx)
    code[6] = 0xb0; // mov $'i', %al
    code[7] = 0x69;
    code[8] = 0xee; // out %al, (%dx)
    code[9] = 0xf4; // hlt

    println!("✓ Loaded guest code: HLT instruction at guest address 0x1000");

    // Step 5: Set up memory region
    let mem_region = KvmUserspaceMemoryRegion {
        slot: 0,
        flags: 0,
        guest_phys_addr: 0x1000,
        memory_size: mem_size as u64,
        userspace_addr: guest_memory.as_ptr() as u64,
    };

    vm.set_user_memory_region(&mem_region)?;
    println!(
        "✓ Mapped guest physical 0x1000 -> host virtual {:p}",
        guest_memory.as_ptr()
    );

    // Step 6: Create VCPU
    let vcpu = vm.create_vcpu(0)?;
    println!("✓ Created VCPU (fd: {})", vcpu.fd());

    // Step 7: Get VCPU mmap size
    let vcpu_mmap_size = kvm.get_vcpu_mmap_size()?;
    println!("✓ VCPU mmap size: {} bytes", vcpu_mmap_size);

    // Step 8: Map VCPU
    let run_ptr = vcpu.map_run(vcpu_mmap_size)?;
    let run = VcpuRun::new(run_ptr, vcpu_mmap_size);
    println!("✓ Mapped VCPU communication area");

    // Step 9: Initialize CPU state
    let mut sregs = vcpu.get_sregs()?;
    sregs.cs.base = 0;
    sregs.cs.limit = 0xffff;
    sregs.cs.selector = 0;
    vcpu.set_sregs(&sregs)?;

    let mut regs = vcpu.get_regs()?;
    regs.rip = 0x1000;
    regs.rax = 0;
    regs.rbx = 0;
    regs.rcx = 0;
    regs.rdx = 0;
    regs.rsi = 0;
    regs.rdi = 0;
    regs.rsp = 0x2000;
    regs.rbp = 0;
    regs.rflags = 0x2;
    vcpu.set_regs(&regs)?;
    println!(
        "✓ Set CPU registers (RIP=0x{:x}, RSP=0x{:x})",
        regs.rip, regs.rsp
    );

    // Step 10: Run the VM
    println!("\n=== Running virtual machine ===");

    let mut exit_count = 0;
    loop {
        vcpu.run()?;

        exit_count += 1;
        let kvm_run = run.as_ref();
        println!(
            "VM Exit #{}: {} (reason={})",
            exit_count,
            exit_reason_name(kvm_run.exit_reason),
            kvm_run.exit_reason
        );

        match kvm_run.exit_reason {
            KVM_EXIT_HLT => {
                println!("✓ VM halted normally");
                break;
            }
            KVM_EXIT_IO => {
                let io = unsafe { &kvm_run.exit_data.io };
                println!(
                    "I/O operation: port=0x{:x}, direction={}, size={}, count={}",
                    io.port,
                    if io.direction == KVM_EXIT_IO_OUT {
                        "OUT"
                    } else {
                        "IN"
                    },
                    io.size,
                    io.count
                );
            }
            KVM_EXIT_SHUTDOWN => {
                println!("VM requested shutdown");
                break;
            }
            KVM_EXIT_INTERNAL_ERROR => {
                let internal = unsafe { &kvm_run.exit_data.internal };
                println!("Internal KVM error: suberror={}", internal.suberror);
                break;
            }
            KVM_EXIT_FAIL_ENTRY => {
                let fail_entry = unsafe { &kvm_run.exit_data.fail_entry };
                println!(
                    "VM entry failed: hardware_entry_failure_reason=0x{:x}",
                    fail_entry.hardware_entry_failure_reason
                );
                break;
            }
            _ => {
                println!("Unhandled exit reason: {}", kvm_run.exit_reason);
                if exit_count > 10 {
                    println!("Too many exits, stopping");
                    break;
                }
            }
        }
    }

    println!("\n=== Cleanup ===");
    println!("✓ VM execution complete!");

    Ok(())
}
