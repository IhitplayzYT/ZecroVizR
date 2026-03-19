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

#[allow(non_camel_case_types,non_snake_case,dead_code)]


#[derive(Debug, Clone, PartialEq)]
pub enum e_KVM {
    UnableToOpen(String),
    Custom(String),
    MemoryInsufficient(String),
    InvalidMaximum(String),
    InvalidMinimum(String),

}

impl std::fmt::Display for e_KVM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            e_KVM::UnableToOpen(x) => write!(f, "[KVM ERROR: UnableToOpenFile]\n{}", x),
            e_KVM::Custom(x) => write!(f, "[KVM ERROR: Custom Error]\n{}", x),
            e_KVM::MemoryInsufficient(x) => write!(f, "[KVM ERROR: Insufficient Memory]\n{}",x),
            e_KVM::InvalidMaximum(x) => write!(f, "[KVM ERROR: Invalid Count [Exceeds Maximum]]\n{}",x),
            e_KVM::InvalidMinimum(x) => write!(f, "[KVM ERROR: Invalid Count [Less than Minimum]]\n{}",x),

        }
    }
}

pub type r_KVM<T> = Result<T, e_KVM>;
