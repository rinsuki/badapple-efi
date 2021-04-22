#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, BltOp, BltPixel};
use uefi::ResultExt;

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize");
    info!("Hello, world!");
    let gop = init_gop(system_table.boot_services());
    let current_size = gop.current_mode_info().resolution();
    let base = ((current_size.0 / 2) - (512/2), (current_size.1 / 2)-(384/2));
    gop.blt(BltOp::VideoFill{
        color: BltPixel::new(
            0x00,
            0x95,
            0xD9,
        ),
        dest: base,
        dims: (512, 384),
    }).unwrap_success();
    loop {}
    Status::SUCCESS
}

fn init_gop(boot_services: &BootServices) -> &mut GraphicsOutput {
    if let Ok(gopp) = boot_services.locate_protocol::<GraphicsOutput>() {
        let gop = unsafe { gopp.unwrap().get().as_mut() }.unwrap();
        info!("Show Available Resolutions:");
        let mut selected_mode = None;
        let mut score = i32::MIN;
        for mode in gop.modes() {
            let mode = mode.unwrap();
            let (width, height) = mode.info().resolution();
            let current_score = if width == 512 && height == 384 {
                i32::MAX
            } else if (width % 4 == 0 && height % 3 == 0) && ((width / 4) == (width / 3)) {
                (i32::MAX / 2) - (width/4) as i32
            } else {
                -((width * height) as i32)
            };
            info!("{} x {} (Score: {})", width, height, current_score);
            if width < 512 || height < 384 {
                continue;
            }
            if score < current_score {
                score = current_score;
                selected_mode = Some(mode);
            }
        }
        if let Some(mode) = selected_mode {
            let (width, height) = mode.info().resolution();
            info!("Choiced: {} x {} (Score: {})", width, height, score);
            gop.set_mode(&mode).expect("failed to set mode").expect("failed to set mode");
        }
        return gop;
    }
    panic!("Failed to Initialize GOP (or unavailable)");
}