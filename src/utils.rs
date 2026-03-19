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
pub mod utils {
#[allow(non_camel_case_types,non_upper_case_globals,non_snake_case)]
    use core::panic;
    #[allow(non_camel_case_types,non_snake_case,non_upper_case_globals)]

    #[allow(non_snake_case, non_camel_case_types)]
    pub fn DBG_STR(inject: &str) -> String {
        format!(
            "****** DEBUG ******\nFile: {}\nLine: {}\nCol: {}\n{}*******************\n",
            file!(),
            line!(),
            column!(),
            inject
        )
    }


    #[derive(Debug,Clone)]
    pub struct Zecro_conf {
      vcpu_cnt: usize,
      is_smp: bool,
      dirty_log: bool,
      kargs: Vec<String>
    }

    impl Zecro_conf {
      fn new() -> Self{
        Self { vcpu_cnt:1,is_smp:false,dirty_log:false,kargs: Vec::new()}
      }

    }


pub fn parse_args() -> Zecro_conf{
  let mut ret = Zecro_conf::new();
  let args:Vec<String> = std::env::args().collect();
  let limit = std::thread::available_parallelism().unwrap().get();
  let l = args.len();
  let mut i = 0;
  while i < l{
    match args[i].as_str() {
      "-h" => {
        println!("{:?}\n",USAGE_STR);
        std::process::exit(0);
      },
      "-c" | "--vcpus" => {
        if i+1 < l{
        i += 1;
        let t:usize = args[i].parse().expect("Invalid value passed to the [-c|--vcpus] argument\nExpected: Positive Integer");
        ret.vcpu_cnt = t.clamp(1, limit);
        }else{
          panic!("No args passed for the [-c|--vcpus] flag");
        }
      },
      "--dirty-log" => ret.dirty_log = true,
      "--smp" => ret.is_smp = true,
      "-a" | "--cmdline" => {
      while i < l{
        match &args[i][..]{
          "-c" | "--vcpus" | "--dirty-log" | "--smp" | "-a" | "--cmdline" => {
            break;   
          },
          "-h" => {
          println!("{:?}\n",USAGE_STR);
          std::process::exit(0);           
          }
          _ => {},
        }
        ret.kargs.push(args[i].clone());
        i += 1;
      }
      if i == l-1{
        break;
      }        
      },
      _ => {
        panic!("Unknown argument provided");
      },
    }
    i += 1;
  }
  ret
}


const USAGE_STR: &str = "\n
Usage: ZecroVizR [OPTIONS]

Options:
  -c, --vcpus <N>    Number of virtual CPUs (default: 1)
      --smp          Enable SMP mode (irqchip + PIT; requires --vcpus >= 2)
      --dirty-log    Enable KVM dirty-page logging
  -a, --cmdline <S>  Kernel command line string
  -h, --help         Show this message
 
Modes:
  (no flags)          single-thread: 1 vCPU, inline run loop
  --vcpus 4           multi-thread:  4 vCPUs, no irqchip
  --vcpus 4 --smp     SMP:           4 vCPUs, in-kernel irqchip + PIT
";



}
