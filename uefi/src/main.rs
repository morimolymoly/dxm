#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use r_efi_alloc;

extern crate alloc;

use alloc::string::*;

#[macro_use]
mod logger;
mod utils;

use core::char::from_u32;
use core::borrow::BorrowMut;
use core::mem::MaybeUninit;

use r_efi::efi::protocols::simple_text_input::InputKey;
use r_efi::*;

#[global_allocator]
static GLOBAL_ALLOCATOR: r_efi_alloc::global::Bridge = r_efi_alloc::global::Bridge::new();

#[alloc_error_handler]
fn rust_oom_handler(_layout: core::alloc::Layout) -> ! {
    panic!();
}

static mut SYSTEM_TABLE: MaybeUninit<efi::SystemTable> = MaybeUninit::uninit();

pub fn system_table() -> &'static efi::SystemTable {
    unsafe { &*SYSTEM_TABLE.as_ptr() }
}

pub fn runtime_services() -> &'static efi::RuntimeServices {
    unsafe { &*system_table().runtime_services }
}

pub fn boot_services() -> &'static efi::BootServices {
    unsafe { &*system_table().boot_services }
}

extern "efiapi" fn handle_exit_boot_services(_event: base::Event, _context: *mut core::ffi::c_void) {
    info!("[~] ExitBootServices() has been called.");
}

extern "efiapi" fn handle_set_virtual_address_map(
    _event: base::Event,
    _context: *mut core::ffi::c_void,
) {
    info!("[~] SetVirtualAddressMap() has been called.");
}

#[no_mangle]
pub fn efi_run(_h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    unsafe { SYSTEM_TABLE = MaybeUninit::new(st.read()) };

    // Register to events relevant for runtime drivers.
    let mut event_virtual_address: base::Event = core::ptr::null_mut();
    let mut status = (boot_services().create_event_ex)(
        efi::EVT_NOTIFY_SIGNAL,
        efi::TPL_CALLBACK,
        core::prelude::v1::Some(handle_set_virtual_address_map),
        runtime_services() as *const _ as *mut core::ffi::c_void,
        &efi::EVENT_GROUP_VIRTUAL_ADDRESS_CHANGE,
        event_virtual_address.borrow_mut(),
    );

    if status.is_error() {
        error!(
            "[-] Creating VIRTUAL_ADDRESS_CHANGE event failed: {:#x}",
            status.as_usize()
        );
        return status;
    }

    let mut event_boot_services: base::Event = core::ptr::null_mut();
    status = (boot_services().create_event_ex)(
        efi::EVT_NOTIFY_SIGNAL,
        efi::TPL_CALLBACK,
        core::prelude::v1::Some(handle_exit_boot_services),
        runtime_services() as *const _ as *mut core::ffi::c_void,
        &efi::EVENT_GROUP_EXIT_BOOT_SERVICES,
        event_boot_services.borrow_mut(),
    );

    if status.is_error() {
        error!(
            "[-] Creating EXIT_BOOT_SERVICES event failed: {:#x}",
            status.as_usize()
        );
        return status;
    }

    for i in 0..10 {
        let mut x = InputKey::default();
        let mut s: usize = 0;
        let _ = unsafe {
            ((*(*st).boot_services).wait_for_event)(1, &mut (*(*st).con_in).wait_for_key, &mut s)
        };

        let r = unsafe { ((*(*st).con_in).read_key_stroke)((*st).con_in, &mut x) };

        if r.is_error() {
            log!("err");
            return r;
        }

        let s = String::from("aiueo");
        log!("{} {}", s, from_u32(x.unicode_char as u32).unwrap());
    }

    efi::Status::SUCCESS
}

#[no_mangle]
pub extern "C" fn efi_main(h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    #[cfg(debug_assertions)]
    {
        utils::wait_for_debugger();
    }
    unsafe {
        let mut allocator = r_efi_alloc::alloc::Allocator::from_system_table(st, efi::LOADER_DATA);
        let _attachment = GLOBAL_ALLOCATOR.attach(&mut allocator);

        log!("hello from uefi runtime driver!");
        efi_run(h, st)
    }
}