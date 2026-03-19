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

    #![allow(non_camel_case_types, non_snake_case, unused_imports,non_upper_case_globals,dead_code)]
    // ****************************************  IMPORTS  ****************************************
    use crate::kvm::arch::{self, arch::asm_test_code};
    use crate::kvm::vcpu::vcpu::{ExecMode, Vcpu_wrapper, e_VCPU, r_VCPU, vcpu_setup,spawn_vcpu_threads};

    use kvm_bindings::{KVM_MAX_CPUID_ENTRIES, KVM_MEM_LOG_DIRTY_PAGES, kvm_pit_config, kvm_regs, kvm_userspace_memory_region};
    use kvm_ioctls::{Cap, DeviceFd, Kvm, VcpuFd, VmFd};
    use libc::KERNEL_VERSION;

    use crate::kvm::kvm_err::*;
    use crate::utils::utils::DBG_STR;
    use crate::io::IO::{r_IO,e_IO};
    use crate::*;
    use core::{panic, sync};
    use std::alloc::alloc;
    use std::sync::{Arc, Barrier, Mutex, RwLock};
    use std::thread;
    use std::io::Write;
    use std::slice;
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
    pub struct DeviceBus{
        mode: ExecMode, // To be used by the methods of the Device Bus
        device: RwLock<Vec<Arc<dyn Device+Send+Sync>>>, // ReadWrite locked vector of devices interface behind a concurrent smart pointer
        barrier:Option<Arc<Barrier>>, // Used for smp to ensure every worker is at same point
        collapse: Mutex<bool>, // To shutdown all threads as a fail safe
    }

    impl DeviceBus{

        pub fn new(mode:ExecMode,vcpu_cnt: usize) -> Arc<Self>{
            let barrier;
            if let ExecMode::Smp = mode{
                barrier = Some(Arc::new(Barrier::new(vcpu_cnt)));
            }else{
                barrier = None
            }
            Arc::new(Self { mode: mode, device: RwLock::new(Vec::new()), barrier, collapse: Mutex::new(false) })
        }

        pub fn register_dev<T: Device+Send+Sync+'static>(&self,device:T) -> r_IO<bool>{
            self.device.try_write().map_err(|op| {
                e_IO::FailedToRegisterDevice(
                    DBG_STR(&format!("Comprimise: Poisoned RWLock!\nReason: [{:?}]\n",op))
                )
            })
            ?.push(Arc::new(device));
            Ok(true)
        }
        pub fn unregister_dev<T: Device+Send+Sync>(&self,device:T) -> r_IO<bool> {
            let idx  = self.device.try_read().map_err(|op| {
                e_IO::UnableToGetDBUS(DBG_STR(&format!("Comprimise: Poisoned RWLock!\nReason: [{:?}]\n",op)))
            })?.iter().position(|op| {
                let op_ptr = &**op as *const dyn Device as *const ();
                let dev_ptr = &device as *const T as *const ();
                op_ptr == dev_ptr
            });
            
            if let Some(x) = idx{
            self.device.try_write().map_err(|op| {
                e_IO::FailedToUnregisterDevice(DBG_STR(&format!("Comprimise: Poisoned RWLock during Unregistering!\nReason: [{:?}]\n",op)))
            })?.remove(x);
            }
            Ok(true)
        }

        pub fn pio_read(&self,port:u16,data: &mut [u8]) -> r_IO<bool>{
            self.device.try_read().map_err(|op| {
                e_IO::UnableToGetDBUS(DBG_STR(&format!("Comprimise: Poisoned RWLock!\nReason: [{:?}]\n",op)))
            })?.iter().for_each(|dev| {
                if let Some((l,r)) = dev.pio_range(){
                    if port > l && port< r{ 
                        dev.pio_read(port, data).unwrap();
                        return;
                    }
                }
            });
            Ok(true)
        }

        pub fn pio_write(&self,port:u16,data: &[u8]) -> r_IO<bool>{
            self.device.try_read().map_err(|op| {
                e_IO::UnableToGetDBUS(DBG_STR(&format!("Comprimise: Poisoned RWLock!\nReason: [{:?}]\n",op)))
            })?.iter().for_each(|dev| {
                if let Some((l,r)) = dev.pio_range(){
                    if port > l && port < r{
                        dev.pio_write(port, data).unwrap();
                        return;
                    }
                }
            });
            Ok(true)
        }

        pub fn mmio_read(&self,addr:u64,data: &mut [u8]) -> r_IO<bool>{
            self.device.try_read().map_err(|op| {
                e_IO::UnableToGetDBUS(DBG_STR(&format!("Comprimise: Poisoned RWLock!\nReason: [{:?}]\n",op)))
            })?.iter().for_each(|dev| {
                if let Some((l,r)) = dev.mmio_range(){
                    if addr > l  && addr < r{
                        dev.mmio_read(addr, data).unwrap();
                        return;
                    }
                }
            });
            Ok(true)
        }

        pub fn mmio_write(&self,addr:u64,data: &[u8]) -> r_IO<bool>{
            self.device.try_read().map_err(|op| {
                e_IO::UnableToGetDBUS(DBG_STR(&format!("Comprimise: Poisoned RWLock!\nReason: [{:?}]\n",op)))
            })?.iter().for_each(|dev| {
                if let Some((l,r)) = dev.mmio_range(){
                    if addr > l && addr < r {
                        dev.mmio_write(addr, data).unwrap();
                        return;
                    }
                }
            });
            Ok(true)
        }


        pub fn smp_init(&self) {
            if let Some(barrier) = &self.barrier {
                barrier.wait();
            }
        }

        pub fn try_shutdown(&self) -> r_IO<bool>{
            self.collapse.try_lock().map_err(|op| {
                e_IO::ShutdownNotReady(
                    DBG_STR(&format!("Can't get shutdown lock!\nReason: [{:?}]\n",op))
                )
            }).unwrap_err();
            Ok(true)
        }


    }

    pub trait Device: Send + Sync{
        fn pio_read(&self,port:u16,data: &mut [u8]) -> r_IO<bool>;
        fn pio_write(&self,port:u16,data: &[u8]) -> r_IO<bool>;
        fn mmio_read(&self,addr:u64,data: &mut [u8]) -> r_IO<bool>;
        fn mmio_write(&self,addr:u64,data: &[u8]) -> r_IO<bool>;
        fn pio_range(&self) -> Option<(u16,u16)> ;
        fn mmio_range(&self) -> Option<(u64,u64)> ;
    }

    pub struct Vmm{
        kvm: Kvm,
        vm_fd: VmFd,
        guest_mem:*mut u8,
        exec_mode:ExecMode,
        device:Arc<DeviceBus>,
        vcpu_cnt: u64,
    }

    unsafe impl Send for Vmm{}
    unsafe impl Sync for Vmm{}

    // ****************************************  DATA STRUCTURES  ****************************************


    // ****************************************  MAIN API  ****************************************

    pub fn INIT_KVM(setup:vcpu_setup) -> r_KVM<bool> {
        let setup = validate_vcpu_config(setup).map_err(|op| {
            e_KVM::Custom(format!("{:?}",e_VCPU::InvalidVcpuSetup(DBG_STR(&format!(
                "Invalid vCPU config recieved\nReason: [ {:?} ]\n",
                op
            )))))
        })?;
        
        let mode = exec_mode_eval(setup); // An enum for if let matching 

        let Vkvm = open_dev_kvm()?; // Validated KVM and /dev/kvm created 

        let Vm_fd: VmFd = Vkvm.create_vm().map_err(|op| {
            e_KVM::UnableToOpen(DBG_STR(&format!(
                "Unable to Open:{:?}\nReason: [ {:?} ]\n",
                "/dev/kvm", op
            )))
        })?; // Initilaised the VM


        let guest_memory = init_guest_mem()?;
        let user_space = init_userspace(guest_memory)?;

        unsafe { Vm_fd.set_user_memory_region(user_space).map_err(|e| {
                e_KVM::MemoryInsufficient(DBG_STR(&format!(
                    "KVM_SET_USER_MEMORY_REGION failed\nReason: [ {:?} ]", e
                )))
            })?;
        }

        //  OPTIONAL TEST PART 
        if setup.dbg {
            test_code(guest_memory as *mut u8);
        }
        //  OPTIONAL TEST PART


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
        })?; // Setup to maintain in kernel timed interrupts

        let mx_vcpu_ids = Vkvm.get_max_vcpu_id() as u64;
        if setup.cnt > mx_vcpu_ids {
            return Err(e_KVM::OverflowsCapacity(
                DBG_STR("The number of vcpus provided exceeds the KVM's max vcpu_id capacity")
            ));
        }

        let mut vCPUs: Vec<VcpuFd> = Vec::new();
        for id in 0..setup.cnt {
            let fd = Vm_fd.create_vcpu(id).map_err(|op| {
                e_KVM::OverflowsCapacity(DBG_STR(&format!(
                    "Unable to create any more vcpus:\nReason: [ {:?} ]\n",
                    op
                )))
            })?;
            if let Ok(true) = init_registers(&fd, id == 0){
            }else{
                panic!("A vcpu wasn't able to be started")
            }
            if let Ok(true) = bind_cpu_id(&fd, id, &Vkvm){
            }else{
                panic!("A vcpu wasn't binded to its cpu_id")
                
            }
            vCPUs.push(fd);
        }

        let Dbus = DeviceBus::new(mode, setup.cnt as usize);

        let handles = spawn_vcpu_threads(vCPUs,Dbus,mode);

        for (id,h) in handles.into_iter().enumerate() {
            if let Err(e) = h.join() {
                return Err(e_KVM::Custom(DBG_STR(&format!("A thread panicked for the vCPU with Id:{}",id))));
            }

        }

        Ok(true)
    }

    // ****************************************  MAIN API  ****************************************

    // ****************************************  INNER FUNCTIONS  ****************************************

    fn validate_vcpu_config(setup:vcpu_setup) -> r_VCPU<vcpu_setup>{
        let max_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u64;
        if (setup.smp && (setup.cnt > 1 && setup.cnt < max_threads)) && (!setup.smp && (setup.cnt > 0 && setup.cnt < max_threads)){
            Ok(setup)
        }else{ 
            Err(e_VCPU::InvalidVcpuSetup(DBG_STR("Invaild vCPU config")))
        }
    }

    fn exec_mode_eval(setup:vcpu_setup) -> ExecMode{
        match (setup.cnt,setup.smp) {
             (1,_) => ExecMode::SingleThreaded,
             (_,false) => ExecMode::MultiThreaded,
             (_,true) => ExecMode::Smp
        }
    }

    fn open_dev_kvm() -> r_KVM<Kvm> {
        let kvm = Kvm::new().unwrap_or_else(|e| {
            eprintln!("{}", DBG_STR(&format!("{:?}", e)[..]));
            exit(-1);
        });
        assert_eq!(kvm.get_api_version(), 12);
        assert!(kvm.check_extension(Cap::UserMemory));
        assert!(kvm.check_extension(Cap::ImmediateExit));
        assert!(kvm.check_extension(Cap::Irqchip));
        Ok(kvm)
    }

    fn bind_cpu_id(vcpu:&VcpuFd,vcpuid:u64,kvm:&Kvm) -> r_KVM<bool> {
        let mut allocated_cpuid = kvm.get_supported_cpuid(KVM_MAX_CPUID_ENTRIES).map_err(|op| {
            e_KVM::InvalidMaximum(DBG_STR(&format!(
                "Larger argument passed then expected:\nMax=>({:?})\nReason: [ {:?} ]\n",
                KVM_MAX_CPUID_ENTRIES, op
            )))
        })?;

        for i in allocated_cpuid.as_mut_slice(){
            if i.function == 1{
                i.ebx = (i.ebx & 0x00ffffff) | ((vcpuid << 24) as u32);
            }
            // THis is to preven userspace access to the initrd code 
            if i.function == 0x80000001{
                i.ecx &= !(1 << 5);
            }

        }
        vcpu.set_cpuid2(&allocated_cpuid).map_err(|op| {
            e_KVM::Custom(format!("{:?}",e_VCPU::CorruptedVCPU(DBG_STR(&format!(
                "Unable to write alloced CPUid to vcpu:\nVcpu_id: {}\nReason: [ {:?} ]\n",
                vcpuid, op
            )))))
        })?;

        Ok(true)
    }


    fn init_registers(vcpu:&VcpuFd,bsp: bool) -> r_KVM<bool> {
        let mut spec_regs = vcpu.get_sregs().map_err(|op| {
           e_KVM::Custom(format!("{:?}",e_VCPU::CorruptedVCPU(DBG_STR(&format!("Unable to retrieve vCpu Spec_Registers\nTarget VcpuFd:{:?}\nReason:[ {:?} ]",vcpu,op)))))
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
           e_KVM::Custom(format!("{:?}",e_VCPU::CorruptedVCPU(DBG_STR(&format!("Unable to update the vCpu Spec_Registers\nTarget VcpuFd:{:?}\nReason:[ {:?} ]",vcpu,op)))))
        })?;

        let regs = kvm_regs{
        rip: if bsp {KERNEL_LOAD_ADDR} else {0},
        rsp: if bsp {BOOT_STACK_ADDR} else {0},
        rsi: if bsp {BOOT_PARAMS_ADDR} else {0},
        rflags: 0x0002,   // Set for x86
        rbp: 0,
        ..Default::default()
        };

        vcpu.set_regs(&regs).map_err(|op| {
           e_KVM::Custom(format!("{:?}",e_VCPU::CorruptedVCPU(DBG_STR(&format!("Unable to update the vCpu Registers\nTarget VcpuFd:{:?}\nReason:[ {:?} ]",vcpu,op)))))
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
