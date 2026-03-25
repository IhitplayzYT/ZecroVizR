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


#[allow(non_camel_case_types, non_snake_case, unused_imports,non_upper_case_globals,dead_code)]

    /// Kvm error enum 
    /// The error returned by the functions
    ///
    /// # Derived 
    ///    -- Debug
    ///    -- Clone
    ///    -- PartialEq
    /// 
    /// # Usage
    /// ```
    /// e_KVM::Custom("Misc err".to_string());
    /// e_KVM::UnableToOpen("diagnostic".to_string());
    /// 
    /// ``` 
    ///

    #[derive(Debug, Clone, PartialEq)]
    pub enum e_KVM {
        UnableToOpen(String),
        MemoryInsufficient(String),
        InvalidMaximum(String),
        InvalidMinimum(String),
        OverflowsCapacity(String),
        Custom(String), // For nested Errors and small unmapped errors and also for error propagation
    }

    impl std::fmt::Display for e_KVM {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                e_KVM::UnableToOpen(x) => write!(f, "[KVM ERROR: UnableToOpenFile]\n{}", x),
                e_KVM::Custom(x) => write!(f, "[KVM ERROR: Custom Error]\n{}", x),
                e_KVM::MemoryInsufficient(x) => write!(f, "[KVM ERROR: Insufficient Memory]\n{}",x),
                e_KVM::InvalidMaximum(x) => write!(f, "[KVM ERROR: Invalid Count [Exceeds Maximum]]\n{}",x),
                e_KVM::InvalidMinimum(x) => write!(f, "[KVM ERROR: Invalid Count [Less than Minimum]]\n{}",x),
                e_KVM::OverflowsCapacity(x) => write!(f, "[KVM ERROR: Violates the KVM's Constraints]\n{}",x),
            }
        }
    }


    /// Return Type
    ///  Result<T,e_KVM>
    /// # Meaning 
    /// -- Success: T -> Parametrised type specified 
    /// -- Failure: e_KVM -> Suitable e_KVM error
    /// 
    
    pub type r_KVM<T> = Result<T, e_KVM>;
