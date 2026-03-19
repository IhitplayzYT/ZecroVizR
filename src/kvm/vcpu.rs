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
pub mod vcpu{
#![allow(non_camel_case_types, non_snake_case, unused_imports,non_upper_case_globals,dead_code)]
    use kvm_ioctls::VcpuFd;
    use std::{fmt::Display, sync::Arc, thread::{self, JoinHandle}};

    use crate::kvm::kvm::kvm::DeviceBus;


#[derive(Debug)]
pub struct Vcpu_wrapper{
pub id: u64,
pub vcpu_fd: VcpuFd,
}

#[derive(Debug,Clone, Copy)]
pub struct vcpu_setup{
    pub cnt: u64,
    pub smp: bool,
    pub dbg:bool,
}

#[derive(Debug)]
pub enum e_VCPU{
InvalidVcpuSetup(String),
CorruptedVCPU(String),
Custom(String),
}

#[derive(Debug,Clone, Copy)]
pub enum ExecMode{
    SingleThreaded,
    MultiThreaded,
    Smp,
}


impl Display for e_VCPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            e_VCPU::InvalidVcpuSetup(x) => write!(f, "[VCPU ERROR: Inconsistent vCPU config]\n{}",x),
            e_VCPU::CorruptedVCPU(x) => write!(f, "[VCPU ERROR: Corrupted/Inconsistent vCPU]\n{}",x),
            e_VCPU::Custom(x) => write!(f, "[VCPU ERROR: Custom Error]\n{}",x)
        }
    }

}


pub type r_VCPU<T> = Result<T,e_VCPU>;



impl vcpu_setup {
    fn new(cnt:u64,smp:bool,dbf:bool) -> Self{
    let max_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u64;
        Self{
           cnt:if cnt > 0 && cnt < max_threads {cnt}else {1},
           smp: if smp && (cnt >0 && cnt < max_threads) {smp}else {false},
           dbg:dbf 
        }
    }
}


pub fn spawn_vcpu_threads(vcpus: Vec<VcpuFd>,Dbus: Arc<DeviceBus>,mode:ExecMode) -> Vec<JoinHandle<()>>{

    match mode {
        ExecMode::SingleThreaded => {
            assert_eq!(vcpus.len(),1);
            let fd = vcpus.into_iter().next().unwrap();
            exec_vcpu(fd,0,Arc::clone(&Dbus));
            vec![]
        },
        ExecMode::MultiThreaded | ExecMode::Smp => {
            vcpus.into_iter().enumerate().map(|(id, fd)| {
                let c_bus = Arc::clone(&Dbus);
                thread::Builder::new()
                .name(format!("vcpu-{id}"))
                .spawn(move || exec_vcpu(fd,id as u64,Arc::clone(&c_bus)))
                .expect(&format!("A vcpu-{} failed to become a thread",id))
        }).collect()
        },
    }
}

pub fn exec_vcpu(vcpu: VcpuFd,id:u64,dbus:Arc<DeviceBus>) {
// TODO: Finish this fxn 
// FIXME:
}



}
