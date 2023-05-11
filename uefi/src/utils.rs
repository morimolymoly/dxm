use crate::error;
use core::arch::asm;

static mut GDB_ATTACHED: bool = false;

pub fn wait_for_debugger() {
    unsafe {
        while !GDB_ATTACHED {
            asm!("pause");
        }
    }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "[-] Panic in {} at ({}, {}):",
            location.file(),
            location.line(),
            location.column()
        );
    }

    loop {}
}