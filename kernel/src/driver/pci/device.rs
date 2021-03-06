use super::PCI;
use crate::memory::frame::{PageFrameIter, PPN};
use crate::object_table::Object;
use crate::object_table::{Ticket, TicketWaker};
use crate::scheduler::MemoryObject;
use crate::sync::SpinLock;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use pci::BaseAddress;

/// A single PCI device.
pub struct PciDevice {
	bus: u8,
	device: u8,
}

/// List of tasks waiting for an interrupt.
static IRQ_LISTENERS: SpinLock<Vec<TicketWaker<usize>>> = SpinLock::new(Vec::new());

// FIXME this is a quick hack to work around a race condition if an interrupt is delivered
// right before poll()
use core::sync::atomic::*;
static POLL_RACE_HACK: AtomicBool = AtomicBool::new(false);

impl PciDevice {
	pub(super) fn new(bus: u8, device: u8) -> Self {
		Self { bus, device }
	}

	pub fn config_region(&self) -> PPN {
		let pci = PCI.auto_lock();
		let pci = pci.as_ref().unwrap();
		let addr = pci.get_physical_address(self.bus, self.device, 0);
		PPN::try_from_usize(addr).unwrap()
	}
}

impl MemoryObject for PciDevice {
	fn physical_pages(&self) -> Box<[PPN]> {
		[self.config_region()].into()
	}
}

impl Object for PciDevice {
	fn poll(&self) -> Ticket<usize> {
		if POLL_RACE_HACK
			.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
			.is_ok()
		{
			return Ticket::new_complete(Ok(0));
		}
		let (ticket, waker) = Ticket::new();
		IRQ_LISTENERS.lock().push(waker);
		ticket
	}

	fn memory_object(self: Arc<Self>, offset: u64) -> Option<Arc<dyn MemoryObject>> {
		if offset == 0 {
			return Some(self);
		}

		let index = usize::try_from(offset - 1).ok()?;
		let pci = PCI.auto_lock();
		let pci = pci.as_ref().unwrap();
		let header = pci.get(self.bus, self.device, 0).unwrap();
		let bar = header.base_addresses().get(index)?;
		let (size, orig) = bar.size();
		bar.set(orig);
		let size = size?;
		if !BaseAddress::is_mmio(orig) {
			return None;
		}
		let upper = || header.base_addresses().get(index + 1).map(|e| e.get());
		let addr = BaseAddress::address(orig, upper).unwrap();
		let frames = PageFrameIter {
			base: PPN::try_from_usize(addr.try_into().unwrap()).unwrap(),
			count: size.get().try_into().unwrap(),
		};
		Some(Arc::new(BarRegion {
			frames: frames.collect(),
		}))
	}
}

/// A single MMIO region pointer to by a BAR of a PCI device.
pub struct BarRegion {
	frames: Box<[PPN]>,
}

impl MemoryObject for BarRegion {
	fn physical_pages(&self) -> Box<[PPN]> {
		self.frames.clone()
	}
}

pub(super) fn irq_handler() {
	POLL_RACE_HACK.store(true, Ordering::Relaxed);
	for e in IRQ_LISTENERS.isr_lock().drain(..) {
		e.isr_complete(Ok(0));
	}
}
