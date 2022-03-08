#![no_std]
#![no_main]
#![forbid(unused_must_use)]
#![feature(alloc_error_handler)]
#![feature(asm_const, asm_sym)]
#![feature(const_trait_impl, inline_const)]
#![feature(derive_default_enum)]
#![feature(drain_filter)]
#![feature(let_else)]
#![feature(maybe_uninit_extra, maybe_uninit_slice, maybe_uninit_uninit_array)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(optimize_attribute)]
#![feature(slice_index_methods)]
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)] // It seems like this feature is mostly complete, really.
#![feature(trait_upcasting)]
#![feature(waker_getters)]

extern crate alloc;

use core::panic::PanicInfo;

macro_rules! bi_from {
	(newtype $a:ident <=> $b:ident) => {
		impl From<$a> for $b {
			fn from(a: $a) -> $b {
				a.0
			}
		}

		impl From<$b> for $a {
			fn from(b: $b) -> $a {
				$a(b)
			}
		}
	};
}

macro_rules! default {
	(newtype $ty:ty = $value:expr) => {
		impl Default for $ty {
			fn default() -> Self {
				Self($value)
			}
		}
	};
}

#[macro_use]
mod log;

mod arch;
mod boot;
mod driver;
mod ffi;
mod memory;
mod object_table;
mod power;
mod scheduler;
mod sync;
mod time;

#[export_name = "main"]
pub extern "C" fn main(boot_info: &boot::Info) -> ! {
	unsafe {
		log::init();
	}

	for region in boot_info.memory_regions() {
		use memory::{
			frame::{MemoryRegion, PPN},
			Page,
		};
		let (base, size) = (region.base as usize, region.size as usize);
		let align = (Page::SIZE - base % Page::SIZE) % Page::SIZE;
		let base = base + align;
		let count = (size - align) / Page::SIZE;
		if let Ok(base) = PPN::try_from_usize(base) {
			let region = MemoryRegion { base, count };
			unsafe {
				memory::frame::add_memory_region(region);
			}
		}
	}

	unsafe {
		memory::r#virtual::init();
		arch::init();
	}

	unsafe {
		driver::init(boot_info);
	}

	assert!(!boot_info.drivers().is_empty(), "no drivers");

	let mut processes = alloc::vec::Vec::with_capacity(8);

	for driver in boot_info.drivers() {
		let process = scheduler::process::Process::from_elf(driver.as_slice()).unwrap();
		processes.push(process);
	}

	processes.leak()[0].run()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	fatal!("Panic!");
	fatal!("{:#?}", info);
	loop {
		power::halt();
	}
}
