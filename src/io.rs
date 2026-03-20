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
pub mod IO{



#[derive(Debug,Clone)]
pub enum e_IO{
FailedToRegisterDevice(String),
FailedToUnregisterDevice(String),
UnableToGetDBUS(String),
ShutdownNotReady(String),
Custom(String)
}

impl std::fmt::Display for e_IO{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
        e_IO::FailedToRegisterDevice(x) => write!(f,"[IO ERROR: Failed to register a device interface to DBUS]\n{}",x),
        e_IO::FailedToUnregisterDevice(x) => write!(f,"[IO ERROR: Failed to unregister a device interface to DBUS]\n{}",x),
        e_IO::UnableToGetDBUS(x) => write!(f,"[IO ERROR: Failed to capture the DBUS]\n{}",x),
        e_IO::Custom(x) => write!(f, "[IO ERROR: Custom Error]\n{}",x),
        e_IO::ShutdownNotReady(x) => write!(f, "[IO ERROR: Dbus is busy can't Shutdown]\n{}",x),
        }
    }


}

pub type r_IO<T> = Result<T,e_IO>;



}