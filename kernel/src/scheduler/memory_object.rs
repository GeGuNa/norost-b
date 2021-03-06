use crate::memory::frame::PPN;
use alloc::boxed::Box;
use core::any::Any;
use core::ops::Range;

/// Objects which can be mapped into an address space.
pub trait MemoryObject
where
	Self: Any,
{
	/// The physical pages used by this object that must be mapped.
	fn physical_pages(&self) -> Box<[PPN]>;

	/// Mark a range of physical pages as dirty. May panic if the range
	/// is invalid.
	fn mark_dirty(&mut self, range: Range<usize>) {
		let _ = range;
	}
}
