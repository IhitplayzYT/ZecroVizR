/*
 * Copyright (C) 2026 Ihit Rajesh Acharya
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY.
 */

pub mod kvm {
    #![allow(non_camel_case_types, non_snake_case, unused_imports,non_upper_case_globals)]
    // ****************************************  IMPORTS  ****************************************
    use crate::kvm::arch::{self, arch::asm_test_code};
    use crate::kvm::vcpu::vcpu::{Vcpu_wrapper, vcpu_setup};

    use kvm_bindings::{KVM_MAX_CPUID_ENTRIES, KVM_MEM_LOG_DIRTY_PAGES, kvm_pit_config, kvm_regs, kvm_userspace_memory_region};
    use kvm_ioctls::{Cap, DeviceFd, Kvm, VcpuFd, VmFd};
    use libc::KERNEL_VERSION;

    use crate::kvm::kvm_err::*;
    use crate::utils::utils::DBG_STR;
    use crate::*;
    use core::panic;
    use std::alloc::alloc;
    use std::io::Write;
    use std::slice;
    use std::sync::Arc;
    use std::{fmt::write, process::exit, ptr::null_mut};
    // ****************************************  IMPORTS  ****************************************

    // ****************************************  GLOBAL CONSTANTS  ****************************************
    const mem_size: u64 = 512 * 1024 * 1024; // 512 MB of guest memory 
    const guest_addr: u64 = 0x0000; // THe starting point of the guest memory space

    pub const KERNEL_LOAD_ADDR: u64 = 0x00100000;  // 1MB,place wherebzImage protected-mode kernel is loaded.

    pub const BOOT_PARAMS_ADDR: u64 = 0x00007000; // boot_params struct (zero-page) location.

    pub const CMDLINE_ADDR: u64     = 0x00008000; // Kernel command line string 

    /// Initial stack for vCPU0 during boot, Needs to be below the ROM hole (0xA0000) and not overlap anything above
    /// 0x8FF0 = top of the page just below 36KB — gives the stack room to grow down.
    pub const BOOT_STACK_ADDR: u64  = 0x00008FF0;

    /// Where initrd is loaded
    pub const INITRD_ADDR: u64      = 0x08000000; // Placed at 128Mb
    // ****************************************  GLOBAL CONSTANTS  ****************************************


    // ****************************************  DATA STRUCTURES  ****************************************

    // An empty struct that acts as a common interface for multi threaded and smp systems
    pub struct DeviceBus;




    pub struct Vmm{
        kvm: Kvm,
        vm_fd: VmFd,
        device:Arc<DeviceFd>
    }


    // ****************************************  DATA STRUCTURES  ****************************************


    // ****************************************  MAIN API  ****************************************

    pub fn INIT_KVM(setup:vcpu_setup,DEBUG_FLAG: bool) -> r_KVM<bool> {
        let state = eval_vcpu_config(setup);


        let Vkvm = open_dev_kvm()?; // Validated KVM and /dev/kvm created 

        let Vm_fd: VmFd = Vkvm.create_vm().map_err(|op| {
            e_KVM::UnableToOpen(DBG_STR(&format!(
                "Unable to Open:{:?}\nReason: [ {:?} ]\n",
                "/dev/kvm", op
            )))
        })?; // Initilaised the VM






        let guest_memory = init_guest_mem()?;
        let user_space = init_userspace(guest_memory)?;

        unsafe { Vm_fd.set_user_memory_region(user_space).unwrap() };

        //  OPTIONAL TEST PART 
        // TODO:
        // FIXME: Check if it is worth adding the in kvm page fault tester

        if DEBUG_FLAG {
            test_code(guest_memory as *mut u8);
        }
        //  OPTIONAL TEST PART

        let vcpu_id = 0_usize;
        let v_cpu0_fd = create_vcpu(Vm_fd, if vcpu_id < Vkvm.get_max_vcpu_id() { vcpu_id as u64 }else {
        panic!("KVM INIT FAILED!![NO VCPUS POSSIBLE]")
        })?; // Making the first Vcpu


        let kvm_runtime_size = Vkvm.get_vcpu_mmap_size().map_err(|op| {
            e_KVM::MemoryInsufficient(DBG_STR(&format!(
                "Insufficient Memory for kvm_run:\nGot=>( {:?} ) Expected=>({:?})\nReason: [ {:?} ]\n",
                mem_size,Vkvm.get_vcpu_mmap_size(), op
            )))
        })?;

        if !init_registers(&v_cpu0_fd)?{
            panic!("Register init failed!!");
        }

        if !bind_cpu_id(&v_cpu0_fd,vcpu_id as u64,Vkvm)? {
            panic!("Cpuid Bind FAILED for vcpu: {:?}\nvcpu_id:{}\n",v_cpu0_fd,vcpu_id);
        }



        Ok(true)
    }

    // ****************************************  MAIN API  ****************************************

    // ****************************************  INNER FUNCTIONS  ****************************************

    fn open_dev_kvm() -> r_KVM<Kvm> {
        let kvm = Kvm::new().unwrap_or_else(|e| {
            eprintln!("{}", DBG_STR(&format!("{:?}", e)[..]));
            exit(-1);
        });
        assert_eq!(kvm.get_api_version(), 12);
        assert!(kvm.check_extension(Cap::UserMemory));
        assert!(kvm.check_extension(Cap::ImmediateExit));
        Ok(kvm)
    }

    fn bind_cpu_id(vcpu:& VcpuFd,vcpuid:u64,kvm:Kvm) -> r_KVM<bool> {
        let mut allocated_cpuid = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES).map_err(|op| {
            e_KVM::InvalidMaximum(DBG_STR(&format!(
                "Larger argument passed then expected:\nMax=>({:?})\nReason: [ {:?} ]\n",
                KVM_MAX_CPUID_ENTRIES, op
            )))
        })?;

        for i in allocated_cpuid.as_mut_slice(){
            if i.function == 1{
                i.ebx = (vcpuid << 24) as u32;
            }
            // THis is to preven userspace access to the initrd code 
            if i.function == 0x80000001{
                i.ecx &= !(1 << 5);
            }

        }
        vcpu.set_cpuid2(&allocated_cpuid).map_err(|op| {
            e_KVM::CorruptedVCPU(DBG_STR(&format!(
                "Unbale to write alloced CPUid to vcpu:\nVcpu_id: {}\nReason: [ {:?} ]\n",
                vcpuid, op
            )))
        })?;

        Ok(true)
    }


    fn init_registers(vcpu:&VcpuFd) -> r_KVM<bool> {
        let mut spec_regs = vcpu.get_sregs().map_err(|op| {
           e_KVM::CorruptedVCPU(DBG_STR(&format!("Unable to retrieve vCpu Spec_Registers\nTarget VcpuFd:{:?}\nReason:[ {:?} ]",vcpu,op)))
        })?;
        // This is VIMP Spec_reg it controls the vCPU operation and architectural intricises
        spec_regs.cr0 = /* Paging */ (1 << 31) | /* Alignment */ (1 << 18) | /* Write protected CPU */ (1 << 16) | /* Use FPU for numeric errors */ (1 << 5) | /* Modern cpu req */ (1 << 4) | /* Monitor Copressing */ (1 << 1) | /* Switch Real -> Proctected mode*/ 1;
        spec_regs.cr4 = 0x000006f8; // Some more extended features used by the CPU and OS
        spec_regs.efer = 0x00000d01; // To eneter long mode [64 bit registers mode BABY!!]

        // Flat unpaged code segment
        spec_regs.cs.base = 0; // Set base reg
        spec_regs.cs.limit = 0xffffffff; // Set max size of code segemnt
        spec_regs.cs.g = 1; // Makes segemnts in pages(4kB)
        spec_regs.cs.db = 0; // In 64 bit ignored if cs.l =1 
        spec_regs.cs.l = 1; // Enable 64 bit mdoe for the cs code segment
        spec_regs.cs.present = 1; // To show valid code segment
        spec_regs.cs.dpl = 0; // Kernel mode for this segment 
        spec_regs.cs.s = 1; // To mark segemnt as code segment   
        spec_regs.cs.type_ = 11;
        vcpu.set_sregs(&spec_regs).map_err(|op| {
           e_KVM::CorruptedVCPU(DBG_STR(&format!("Unable to update the vCpu Spec_Registers\nTarget VcpuFd:{:?}\nReason:[ {:?} ]",vcpu,op)))
        })?;

        let regs = kvm_regs{
        rip: KERNEL_LOAD_ADDR,
        rsp: BOOT_STACK_ADDR,
        rflags: 0x0002,   // Set for x86
        rbp: 0,
        rsi: BOOT_PARAMS_ADDR,
        ..Default::default()
        };

        vcpu.set_regs(&regs).map_err(|op| {
           e_KVM::CorruptedVCPU(DBG_STR(&format!("Unable to update the vCpu Registers\nTarget VcpuFd:{:?}\nReason:[ {:?} ]",vcpu,op)))
        })?;       
        Ok(true)
    }

    fn create_vcpu(Vm_fd: VmFd,id: u64) -> r_KVM<VcpuFd>{
        let v_cpu0_fd = Vm_fd.create_vcpu(id).map_err(|op| {
            e_KVM::Custom(DBG_STR(&format!(
                "Unable to init vcpu{}\nReason:[ {:?} ]",
                0, op
            )))
        })?;

        Vm_fd.create_irq_chip().map_err(|op| {
            e_KVM::Custom(DBG_STR(&format!(
                "Unable to set In-kernel interrupt controller:\nReason: [ {:?} ]\n",
                op
            )))
        })?; // Setup in kernel interrupts for smp systems

        let pit = kvm_pit_config{flags:0,..Default::default()};
        Vm_fd.create_pit2(pit).map_err(|op| {
            e_KVM::Custom(DBG_STR(&format!(
                "Unable to set In-kernel timed interrupts:\nReason: [ {:?} ]\n",
                op
            )))
        })?; // Setup to maintian in kernel timed interrupts

        
        Ok(v_cpu0_fd)
    }

    fn test_code(mmaped_addr: *mut u8) {
        unsafe {
            let mut slice_view = slice::from_raw_parts_mut(mmaped_addr, mem_size as usize);
            slice_view.write(asm_test_code).unwrap();
        };
    }

    fn init_guest_mem() -> r_KVM<*mut u8> {
        let ret: *mut u8 = unsafe {
            libc::mmap(
                null_mut(),
                mem_size as usize,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC, // Allocated unprotected memory
                // to be used for
                // reading,writing and executing
                libc::MAP_ANONYMOUS | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE, // Following flags are
                // for
                -1,
                0,
            ) as *mut u8
        };
        assert_ne!(ret, libc::MAP_FAILED as *mut u8);
        Ok(ret)
    }

    fn init_userspace(mmaped_addr: *mut u8) -> r_KVM<kvm_userspace_memory_region> {
        Ok(kvm_userspace_memory_region {
            slot: 0,
            guest_phys_addr: guest_addr,
            memory_size: mem_size,
            userspace_addr: mmaped_addr as u64,
            flags: KVM_MEM_LOG_DIRTY_PAGES,
        })
    }

    // ****************************************  INNER FUNCTIONS  ****************************************
}
