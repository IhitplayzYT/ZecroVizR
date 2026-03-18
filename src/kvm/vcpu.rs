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
    use kvm_ioctls::VcpuFd;
    use std::thread;
    use std::sync::{Arc,Mutex};

#[allow(non_camel_case_types,non_snake_case)]

#[derive(Debug)]
pub struct Vcpu_wrapper{
pub id: u64,
pub vcpu_fd: VcpuFd,
}

#[derive(Debug,Clone, Copy)]
pub struct vcpu_setup{
    pub cnt: u64,
    pub smp: bool,
}

impl vcpu_setup {
    fn new(cnt:u64,smp:bool) -> Self{
    let max_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u64;
        Self{
           cnt:if cnt > 0 && cnt < max_threads {cnt}else {1},
           smp: if smp && (cnt >0 && cnt < max_threads) {smp}else {false},
        }
    }
}






}