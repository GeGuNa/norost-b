#![no_std]
#![feature(asm_sym)]
#![feature(naked_functions)]
#![feature(optimize_attribute)]
#![feature(slice_ptr_get)]

#[cfg(feature = "userspace")]
#[macro_use]
pub mod syscall;

#[cfg(feature = "rt")]
mod rt;

#[repr(align(4096))]
#[repr(C)]
pub struct Page([u128; 256]);
