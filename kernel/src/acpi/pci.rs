use acpi::mcfg::Mcfg;
use anyhow::{anyhow, Result};
use x86_64::structures::paging::{PageSize, Size2MiB};

use crate::{memory, println};
use crate::memory::MemoryPool;
use crate::acpi::tables::ACPI;

pub const PCI_START: u64 = 0xffffffff_00000000;

pub struct PCI;

impl PCI {
    pub fn enumerate(acpi: &ACPI) -> Result<PCI> {
        println!("[PCI] Enumerating Bus..");

        let mcfg = acpi.tables.find_table::<Mcfg>().map_err(|e| anyhow!("{e:?}"))?;
        for entry in mcfg.entries() {
            let base = entry.base_address;
            let seggroup = entry.pci_segment_group;

            println!("0x{:x}: SEGGROUP {} BUS {} - {}", base, seggroup, entry.bus_number_start, entry.bus_number_end);

            for bus in entry.bus_number_start..=entry.bus_number_end {
                let bus_start = base + bus as u64 * 256 * 4096;
                let pool = MemoryPool::single(x86_64::align_down(bus_start, Size2MiB::SIZE));
                unsafe { memory::map(pool, PCI_START); }

                for device in 0..32 {
                    for function in 0..8 {
                        let a = PCI::addr(PCI_START, bus % 2, device, function);
                        let vid = unsafe { *(a as *const u16) };
                        let pid = unsafe { *(a as *const u16).add(1) };
                        if vid == 0xffff { continue; }

                        println!("BUS {bus} DEV {device} FUNC {function}: VID 0x{vid:x} PID 0x{pid:x}");
                    }
                }

                unsafe { memory::unmap(PCI_START, 1); }
            }
        }

        println!("[PCI] Success");

        Ok(PCI)
    }

    fn addr(base: u64, bus: u8, device: u8, function: u8) -> u64 {
        base + ((bus as u64) << 20 | (device as u64) << 15 | (function as u64) << 12)
    }
}
