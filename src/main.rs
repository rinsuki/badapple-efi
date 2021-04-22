#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, BltOp, BltPixel};
use uefi::ResultExt;
use uefi::table::boot::{EventType, TimerTrigger, Tpl};

const WIDTH: usize = 512;
const HEIGHT: usize = 384;
const FPS: u64 = 30;
const FPS_100NS: u64 = 1_000_000_0/*00ns*/ / FPS;

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
    let base = ((current_size.0 / 2) - (WIDTH/2), (current_size.1 / 2)-(HEIGHT/2));
    // MAIN
    let mut i: u8 = 0;
    boot_services.set_timer(timer_event, TimerTrigger::Periodic(FPS_100NS)).unwrap().unwrap();
    loop {
        // logic
        i = if i == (FPS as u8) { 0 } else { i + 1 };
        // actual draw
        // TODO: use vsync (if exists in UEFI)
        boot_services.wait_for_event(&mut timer_events).unwrap().unwrap();
        gop.blt(BltOp::VideoFill{
            color: BltPixel::new(
                i * 4,
                0x95,
                0xD9,
            ),
            dest: base,
            dims: (WIDTH, HEIGHT),
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
            let current_score = if width == WIDTH && height == HEIGHT {
                i32::MAX
            } else if (width % 4 == 0 && height % 3 == 0) && ((width / 4) == (height / 3)) {
                (i32::MAX / 2) - (width/4) as i32
            } else {
                -((width * height) as i32)
            };
            info!("{} x {} (Score: {})", width, height, current_score);
            if width < WIDTH || height < HEIGHT {
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