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
#[allow(non_camel_case_types,non_upper_case_globals,non_snake_case,dead_code)]
    use kvm_ioctls::VcpuFd;
    use std::fmt::Display;


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

#[derive(Debug)]
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






}