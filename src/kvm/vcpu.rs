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


    /// Vcpu Wrapper struct
    /// # Members:
    /// -- id:u64 -> The namespace id of vcpu
    /// -- vcpu_fd:VcpuFd -> The Fd of the Vcpu
    /// 
    /// # Usage
    /// ``` 
    /// Vcpu_wrapper{id:0,vcpu_fd};
    /// ``` 
    ///
    
    #[derive(Debug)]
    pub struct Vcpu_wrapper{
    pub id: u64,
    pub vcpu_fd: VcpuFd,
    }

    /// Vcpu setup struct
    ///
    /// # Members:
    /// -- cnt: u64 -> The total count of vcpus to be created 
    /// -- smp: bool -> Flag for smp config (true -> smp,false -> Single/Multi)
    /// -- dbg: bool -> The debug flag
    /// # Usage
    /// ```
    /// vcpu_setup{cnt: 2,dbg:false,smp:true};
    /// 
    /// ``` 
    ///

    #[derive(Debug,Clone, Copy)]
    pub struct vcpu_setup{
        pub cnt: u64,
        pub smp: bool,
        pub dbg: bool,
    }

    /// Vcpu error enum 
    /// The error returned by the functions
    ///
    /// # Derived 
    ///    -- Debug
    ///    -- Clone
    ///    -- PartialEq
    /// 
    /// # Provides
    ///    -- InvalidVcpuSetup(String)
    ///    -- CorruptedVCPU(String)
    ///    -- Custom(String)
    ///  
    /// # Usage
    /// ```
    /// e_VCPU::Custom("Misc err".to_string());
    /// e_VCPU::CorruptedVCPU("diagnostic".to_string());
    /// 
    /// ``` 
    ///
 

    #[derive(Debug,Clone,PartialEq)]
    pub enum e_VCPU{
    InvalidVcpuSetup(String),
    CorruptedVCPU(String),
    Custom(String),
    }

    /// ExecMode enum 
    ///
    /// # Derived 
    ///    -- Debug
    ///    -- Clone
    ///    -- Copy
    ///    -- PartialEq 
    /// 
    /// # Provides
    ///    -- SingleThreaded
    ///    -- MultiThreaded
    ///    -- Smp
    ///  
    /// # Usage
    /// ```
    /// ExecMode::Smp;
    /// ExecMode::MultiThreaded
    /// ``` 
    ///

    #[derive(Debug,Clone, Copy,PartialEq)]
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


    /// Return Type
    ///  Result<T,e_VCPU>
    /// 
    /// # Meaning 
    /// -- Success: T -> Parametrised type specified 
    /// -- Failure: e_VCPU -> Suitable e_VCPU error
    /// 

    pub type r_VCPU<T> = Result<T,e_VCPU>;



    impl vcpu_setup {

        /// vcpu_setup Helper function
        /// Initialises the Vcpu struct and validates it
        /// 
        /// # Arguments
        ///   -- cnt:u64 -> The cnt of threads 
        ///   -- smp:bool -> The smp flag
        ///   -- dbg:bool ->  The debug flag
        /// 
        /// # Example
        /// ```
        /// vcpu_setup::new(2,false,true);
        /// 
        /// ``` 

        fn new(cnt:u64,smp:bool,dbg:bool) -> Self{
        let max_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u64;
            Self{
               cnt:if cnt > 0 && cnt < max_threads {cnt}else {1},
               smp: if smp && (cnt >0 && cnt < max_threads) {smp}else {false},
               dbg 
            }
        }
    }

    /// Spawns, initialises and satisfies the dependencies of the Vcpu based on execmode 
    /// 
    /// # Arguments
    ///   -- vcpus: Vec<VcpuFd> -> A vector of Vcpus to be spawned 
    ///   -- Dbus: Arc<DeviceBus> -> The Dbus to be shared by the vcpus
    ///   -- mode: ExecMode> -> The execmode of the Vcpus provided 
    ///
    /// # Returns
    ///     Vec<JoinHandle<()>> -> Vectors of join handles to provide wait,join operations of the Vcpus and also allow for returning the value returned by the each of the processes ran on the Vpcus
    /// #
    /// 
    /// # Example
    /// ```
    /// spawn_vcpu_threads(vec![t1,t2,...],dbus,ExecMode::MultiThreaded);
    /// 
    /// ``` 
    /// 

    pub fn spawn_vcpu_threads(vcpus: Vec<VcpuFd>,Dbus: Arc<DeviceBus>,mode: ExecMode) -> Vec<JoinHandle<()>>{
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

    /// Starts the execution of the vpcu 
    /// 
    /// # Arguments
    ///   -- vcpu: VcpuFd -> The vcpu to start executing to be spawned 
    ///   -- Dbus: Arc<DeviceBus> -> The Dbus to be provided to the executing Vcpu 
    ///   -- id: u64 -> The namespaced id of the vcpu
    /// 
    /// # Example
    /// ```
    /// exec_mode(vcpu_fd,0,dbus);
    /// 
    /// ```
    ///    

    pub fn exec_vcpu(vcpu: VcpuFd,id: u64,Dbus: Arc<DeviceBus>) {
    // TODO: Finish this fxn 
    // FIXME:
    }



}
