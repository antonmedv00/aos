use acpi::AcpiTables;
use anyhow::{anyhow, Result};

use crate::println;
use crate::acpi::mapper::AcpiMapper;

pub struct ACPI {
    pub tables: AcpiTables<AcpiMapper>
}

impl ACPI {
    pub fn parse(addr: u64) -> Result<ACPI> {
        println!("[ACPI] Parsing Tables..");

        let mapper = AcpiMapper::new();
        let tables = unsafe { AcpiTables::from_rsdp(mapper, addr as usize).map_err(|e| anyhow!("{e:?}"))? };

        println!("[ACPI] Success");

        Ok(ACPI { tables })
    }
}
