#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::ResultExt;

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize");
    info!("Hello, world!");
    if let Ok(mut gopp) = system_table.boot_services().locate_protocol::<GraphicsOutput>() {
        let gop = unsafe { &mut *gopp.unwrap().get() };
        for mode in gop.modes() {
            let inf = mode.unwrap();
            let inff = inf.info();
            let (width, height) = inff.resolution();
            info!("Mode: {}, {}", width, height);
        }
    }
    Status::SUCCESS
}