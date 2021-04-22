#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;
use uefi::ResultExt;

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize");
    info!("Hello, world!");
    Status::SUCCESS
}