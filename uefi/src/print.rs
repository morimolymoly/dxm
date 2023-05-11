use atomic_refcell::AtomicRefCell;
use core::fmt;
use x86_64::instructions::port::PortWriteOnly;

pub struct SERIAL;
pub static PORT: AtomicRefCell<PortWriteOnly<u8>> = AtomicRefCell::new(PortWriteOnly::new(0x3f8));

impl fmt::Write for SERIAL {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut port = PORT.borrow_mut();
        for b in s.bytes() {
            unsafe { port.write(b) }
        }
        Ok(())
    }
}
macro_rules! log {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        writeln!(crate::print::SERIAL, $($arg)*).unwrap();
    }};
}