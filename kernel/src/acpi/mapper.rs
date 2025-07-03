use core::cell::RefCell;
use core::ptr::NonNull;
use acpi::{AcpiHandler, PhysicalMapping};

use crate::memory;
use crate::memory::MemoryPool;

#[derive(Clone)]
pub struct AcpiMapper {
    top: RefCell<u64>
}

impl AcpiMapper {
    pub fn new() -> AcpiMapper {
        AcpiMapper { top: RefCell::new(0xfffffffe_00000000) }
    }
}

impl AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let pool = MemoryPool::align(physical_address as u64, physical_address as u64 + size as u64);
        let offset = physical_address as u64 - pool.start;
        let mapped_size = pool.size();

        let virt = *self.top.borrow();
        *self.top.borrow_mut() += mapped_size;

        memory::map(pool, virt);

        PhysicalMapping::new(
            physical_address,
            NonNull::new((virt + offset) as *mut T).expect("Impossible"),
            mapped_size as usize,
            mapped_size as usize,
            self.clone()
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}
