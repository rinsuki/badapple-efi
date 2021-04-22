#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, BltOp, BltPixel};
use uefi::ResultExt;
use uefi::table::boot::{EventType, TimerTrigger, Tpl};

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize");
    info!("Hello, world!");
    let boot_services = system_table.boot_services();
    // INIT TIMER
    let timer_event = unsafe { boot_services.create_event(
        EventType::TIMER, Tpl::APPLICATION, None
    ) }.unwrap().unwrap();
    let mut timer_events = [timer_event];
    // INIT GOP
    let gop = init_gop(boot_services);
    let current_size = gop.current_mode_info().resolution();
    let base = ((current_size.0 / 2) - (512/2), (current_size.1 / 2)-(384/2));
    // MAIN
    let mut i = 0;
    boot_services.set_timer(timer_event, TimerTrigger::Relative(1_000_000 / 60)).unwrap().unwrap();
    loop {
        // logic
        i = if i == 255 { 0 } else { i + 1 };
        // actual draw
        // TODO: use vsync (if exists in UEFI)
        boot_services.wait_for_event(&mut timer_events).unwrap().unwrap();
        boot_services.set_timer(timer_event, TimerTrigger::Relative(1_000_000 / 60)).unwrap().unwrap();
        gop.blt(BltOp::VideoFill{
            color: BltPixel::new(
                i,
                0x95,
                0xD9,
            ),
            dest: base,
            dims: (512, 384),
        }).unwrap_success();
    }
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