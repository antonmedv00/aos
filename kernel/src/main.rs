#![no_std]
#![no_main]

mod alloc;

use core::panic::PanicInfo;

use kernel::acpi::pci::PCI;
use kernel::{print, println};
use kernel::acpi::tables::ACPI;
use kernel::drivers::keyboard::Keyboard;
use kernel::drivers::video::framebuffer::Framebuffer;
use kernel::drivers::video::printer::{Color, Printer};
use kernel::memory::MemoryPool;

#[no_mangle]
#[link_section = ".ltext.astart"]
extern "sysv64" fn astart(acpi_addr: u64, _free_ptr: *const MemoryPool, _free_size: usize, fb: Framebuffer<'static>) -> ! {
    Printer::init_global(fb, kernel::drivers::video::fonts::SF_PRO, 30.0, Color::new(255.0, 255.0, 255.0));

    let acpi = ACPI::parse(acpi_addr).unwrap();
    let pci = PCI::enumerate(&acpi).unwrap();

    let mut kb = Keyboard::new();

    loop {
        if let Some(x) = kb.read_char().unwrap() {
            print!("{x}");
        }
    }
}

#[used]
#[link_section = ".ltext.kstart"]
static KSTART: [u8; 16] = [0x48, 0xc7, 0xc4, 0xf0, 0xff, 0xff, 0xff, 0x48, 0xc7, 0xc0, 0x00, 0x00, 0x00, 0xaf, 0xff, 0xe0];

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("[Panic]: {}", info);

    loop {}
}
