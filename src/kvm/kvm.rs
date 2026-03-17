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
    #![allow(non_camel_case_types, non_snake_case, unused_imports)]
    // ****************************************  IMPORTS  ****************************************
    use crate::kvm::arch::{self, arch::asm_test_code};
    use kvm_bindings::{self, KVM_MEM_LOG_DIRTY_PAGES, kvm_userspace_memory_region};
    use kvm_ioctls::{Cap, Kvm, VmFd};

    use crate::kvm::kvm_err::*;
    use crate::utils::utils::DBG_STR;
    use crate::*;
    use core::slice;
    use std::{fmt::write, process::exit, ptr::null_mut};
    // ****************************************  IMPORTS  ****************************************

    // ****************************************  GLOBAL CONSTANTS  ****************************************
    const mem_size: u64 = 512 * 1024 * 1024; // 512 MB of guest memory 
    const guest_addr: u64 = 0x0000; // THe starting point of the guest memory space
    // ****************************************  GLOBAL CONSTANTS  ****************************************

    // ****************************************  MAIN API  ****************************************

    pub fn INIT_KVM(DEBUG_FLAG: bool) -> r_KVM<bool> {
        let Vkvm = open_dev_kvm()?; // Validated KVM and /dev/kvm created 
        let Vm_fd: VmFd = Vkvm.create_vm().map_err(|op| {
            e_KVM::UnableToOpen(DBG_STR(&format!(
                "Unable to Open:{:?}\nReason: [ {:?} ]\n",
                "/dev/kvm", op
            )))
        })?; // Initilaised the VM

        let guest_memory: *mut u8 = init_guest_mem()?;
        let user_space: kvm_userspace_memory_region = init_userspace(guest_memory as *mut u8)?;

        unsafe { Vm_fd.set_user_memory_region(user_space).unwrap() };

        //  OPTIONAL PART
        if DEBUG_FLAG {
            test_code(guest_memory as *mut u8);
        }
        //  OPTIONAL PART

        let v_cpu0_fd = Vm_fd.create_vcpu(0).map_err(|op| {
            e_KVM::Custom(DBG_STR(&format!(
                "Unable to init vcpu{}\nReason:[ {:?} ]",
                0, op
            )))
        })?;

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

    fn test_code(mmaped_addr: *mut u8) {
        unsafe {
            let mut slice_view = slice::from_raw_parts(mmaped_addr, mem_size as usize);
            // TODO:
            // FIXME:
            // Write the assm code to this
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
