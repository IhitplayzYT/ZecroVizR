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
#[derive(Debug, Clone, PartialEq)]
pub enum e_KVM {
    UnableToOpen(String),
    Custom(String),
}

impl std::fmt::Display for e_KVM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            e_KVM::UnableToOpen(x) => write!(f, "[KVM ERROR: UnableToOpenFile]\n{}", x),
            e_KVM::Custom(x) => write!(f, "[KVM ERROR: Custom Message]\n{}", x),
        }
    }
}

pub type r_KVM<T> = Result<T, e_KVM>;
