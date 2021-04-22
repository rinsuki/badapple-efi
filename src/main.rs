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
    init_gop(system_table.boot_services());
    Status::SUCCESS
}

fn init_gop(boot_services: &BootServices) {
    if let Ok(gopp) = boot_services.locate_protocol::<GraphicsOutput>() {
        let mut gop = unsafe { gopp.unwrap().get().as_mut() }.unwrap();
        info!("Show Available Resolutions:");
        let mut usedMode = None;
        let mut score = i32::MIN;
        for mode in gop.modes() {
            let inf = mode.unwrap();
            let inff = inf.info();
            let (width, height) = inff.resolution();
            let currentScore = if width == 512 && height == 384 {
                i32::MAX
            } else if (width % 512 == 0 && height % 384 == 0) && ((width / 512) == (height / 384)) {
                i32::MAX - (width/512) as i32
            } else if (width % 4 == 0 && height % 3 == 0) && ((width / 4) == (width / 3)) {
                (i32::MAX / 2) - (width/4) as i32
            } else {
                -((width * height) as i32)
            };
            info!("{} x {} (Score: {})", width, height, currentScore);
            if width < 512 || height < 384 {
                continue;
            }
            if score < currentScore {
                score = currentScore;
                usedMode = Some(inf);
            }
        }
        if let Some(mode) = usedMode {
            let (width, height) = mode.info().resolution();
            info!("Choiced: {} x {} (Score: {})", width, height, score);
            gop.set_mode(&mode).expect("failed to set mode").expect("failed to set mode");
        }
        return;
    }
    panic!("Failed to Initialize GOP (or unavailable)");
}