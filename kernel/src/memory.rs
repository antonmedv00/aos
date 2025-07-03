extern crate alloc;

use core::alloc::Layout;
use alloc::alloc::alloc;
use anyhow::{anyhow, Error, Result};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::{addr, PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame, Size2MiB, Size4KiB, Translate};

pub const KERNEL_START: u64 = 0xffffffff_af000000;
pub const KERNEL_END:   u64 = 0xffffffff_ffffffff;
pub const KERNEL_SIZE:  u64 = KERNEL_END - KERNEL_START + 1;

#[derive(Clone, Copy)]
pub struct MemoryPool {
    pub start: u64,
    pub end: u64
}

impl MemoryPool {
    pub fn single(start: u64) -> MemoryPool {
        if !VirtAddr::new(start).is_aligned(Size2MiB::SIZE) {
            panic!("MemoryPool::single expects 2MB-aligned address: 0x{start:x}");
        }

        MemoryPool::align(start, start + 1)
    }

    pub fn align(start: u64, end: u64) -> MemoryPool {
        MemoryPool {
            start: addr::align_down(start, Size2MiB::SIZE),
            end:   addr::align_up(end, Size2MiB::SIZE)
        }
    }
    
    pub fn size(&self) -> u64 {
        self.end - self.start
    }

    pub unsafe fn map<A: FrameAllocator<Size4KiB>>(&self, page_table: &mut OffsetPageTable, falloc: &mut A, vstart: u64) -> Result<()> {
        let pstart = self.start;
        let pend = self.end - 1;
        let vend = vstart + self.size() - 1;

        let vstart = Page::<Size2MiB>::from_start_address(VirtAddr::new(vstart)).map_err(Error::msg)?;
        let vend = Page::containing_address(VirtAddr::new(vend));
        let pstart = PhysFrame::from_start_address(PhysAddr::new(pstart)).map_err(Error::msg)?;
        let pend = PhysFrame::containing_address(PhysAddr::new(pend));

        let pages = Page::range_inclusive(vstart, vend);
        let frames = PhysFrame::range_inclusive(pstart, pend);

        if pages.len() != frames.len() { return Err(anyhow!("Incorrect mapping")); }

        for (page, frame) in pages.zip(frames) {
            let mut map_page = || -> Result<(), MapToError<_>> {
                page_table
                    .map_to(page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, falloc)?
                    .flush();

                Ok(())
            };

            map_page()
                .or_else(|e| {
                    match e {
                        MapToError::PageAlreadyMapped(_) => Ok(()),
                        _                                => Err(anyhow!("{e:?}"))
                    }
                })?;
        }

        Ok(())
    }
}

pub struct GlobalFrameAllocator;

unsafe impl<S: PageSize> FrameAllocator<S> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        unsafe {
            let (ptframe, _) = Cr3::read();
            let pt = &mut *(ptframe.start_address().as_u64() as *mut PageTable);
            let page_table = OffsetPageTable::new(pt, VirtAddr::zero());

            let layout = Layout::from_size_align(S::SIZE as usize, S::SIZE as usize).ok()?;
            let ptr = alloc(layout);

            let addr = page_table.translate_addr(VirtAddr::from_ptr(ptr))?;
            PhysFrame::from_start_address(addr).ok()
        }
    }
}

// TODO: maybe store page table in a static struct

pub unsafe fn map(pool: MemoryPool, virt: u64) {
    let (ptframe, _) = Cr3::read();
    let pt = &mut *(ptframe.start_address().as_u64() as *mut PageTable);
    let mut page_table = OffsetPageTable::new(pt, VirtAddr::zero());

    pool.map(&mut page_table, &mut GlobalFrameAllocator, virt).unwrap();
}

pub unsafe fn unmap(vstart: u64, count: usize) {
    let (ptframe, _) = Cr3::read();
    let pt = &mut *(ptframe.start_address().as_u64() as *mut PageTable);
    let mut page_table = OffsetPageTable::new(pt, VirtAddr::zero());

    let vstart = Page::<Size2MiB>::from_start_address(VirtAddr::new(vstart)).unwrap();
    let vend = vstart + count as u64;
    for page in Page::range(vstart, vend) {
        page_table.unmap(page).unwrap().1
            .flush();
    }
}
