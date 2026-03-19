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

#![allow(non_camel_case_types,non_upper_case_globals,non_snake_case,dead_code)]
mod kvm;
mod utils;
mod ZecroVM;
mod io;
#[allow(unused_imports, dead_code, non_camel_case_types, non_snake_case)]
use std::env;
fn main() {
    let flags = crate::utils::utils::parse_args();
    
    
    
    println!("Hello, world!");
}
