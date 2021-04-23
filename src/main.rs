#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use byteorder::ByteOrder;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::ResultExt;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, FileType, RegularFile};
use uefi::table::boot::{EventType, TimerTrigger, Tpl};
use byteorder::{LittleEndian};

const WIDTH: usize = 512;
const HEIGHT: usize = 384;
const PIXELS: usize = WIDTH * HEIGHT;
const FPS: u64 = 30;
const FPS_100NS: u64 = 1_000_000_0/*00ns*/ / FPS;

#[entry]
fn efi_main(handle: Handle, system_table: uefi::table::SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize");
    info!("Hello, world!");
    let boot_services = system_table.boot_services();
    // INIT TIMER
    let timer_event = unsafe { boot_services.create_event(
        EventType::TIMER, Tpl::APPLICATION, None
    ) }.unwrap().unwrap();
    let mut timer_events = [timer_event];
    // INIT & LOAD FROM FS
    let fs = boot_services.get_image_file_system(handle).unwrap().unwrap();
    let fs = unsafe { &mut *fs.get() };
    let mut dir = fs.open_volume().unwrap().unwrap();
    let mut seek_file: RegularFile = open_regular_file(&mut dir, "seek.bin");
    let seek_info = load_seek_info(&mut seek_file);
    let frames = seek_info.len() - 1;
    info!("Frames: {}", frames);
    let mut data_file: RegularFile = open_regular_file(&mut dir, "data.bin");
    info!("Loading Data...");
    let data = {
        let mut d = vec![0; get_file_size(&mut data_file) as usize];
        data_file.read(&mut d).unwrap().unwrap();
        d
    };
    info!("Data Loaded.");
    // INIT GOP
    // Note: we should initialize gop at last, because init_gop disable text console
    let gop = init_gop(boot_services);
    let current_size = gop.current_mode_info().resolution();
    let base = ((current_size.0 / 2) - (WIDTH/2), (current_size.1 / 2)-(HEIGHT/2));
    // MAIN
    let mut frame = 0;
    boot_services.set_timer(timer_event, TimerTrigger::Periodic(FPS_100NS)).unwrap().unwrap();
    let mut frame_buffer = vec![BltPixel::new(255, 0, 0); PIXELS];
    info!("frame_buffer len: {}", frame_buffer.len());
    let vec = data;
    loop {
        // logic
        let mut vi = seek_info[frame] as usize;
        let mut pi = 0;
        while pi < PIXELS {
            let head = vec[vi];
            vi += 1;
            let is_fill = (head >> 7) == 0;
            let cnt = 1 + match (head >> 5) & 0b11 {
                0b00 => (head & 0b11111) as usize,
                0b01 => {
                    let low = (head & 0b11111) as usize;
                    let high = vec[vi] as usize;
                    vi += 1;
                    (high << 5) | low
                },
                0b10 => {
                    let low = (head & 0b11111) as usize;
                    let mid = vec[vi] as usize;
                    vi += 1;
                    let high = vec[vi] as usize;
                    vi += 1;
                    (high << 13) | (mid << 5) | low
                },
                _ => panic!("invalid cnt")
            };
            if is_fill {
                let color = vec[vi];
                vi += 1;
                for _ in 0..cnt {
                    frame_buffer[pi].red = color;
                    frame_buffer[pi].blue = color;
                    frame_buffer[pi].green = color;
                    pi += 1;
                }
            } else {
                for _ in 0..cnt {
                    let color = vec[vi];
                    vi += 1;
                    frame_buffer[pi].red = color;
                    frame_buffer[pi].green = color;
                    frame_buffer[pi].blue = color;
                    pi += 1;
                }
            }
        }

        // /logic
        frame = if frame == (frames-1) {
            info!("Loop...");
            0
        } else { frame + 1 };
        // actual draw
        // TODO: use vsync (if exists in UEFI)
        boot_services.wait_for_event(&mut timer_events).unwrap().unwrap();
        gop.blt(BltOp::BufferToVideo{
            buffer: &frame_buffer,
            src: BltRegion::SubRectangle {
                coords: (0, 0),
                px_stride: WIDTH,
            },
            dest: base,
            dims: (WIDTH, HEIGHT),
        }).unwrap_success();
    }
    Status::SUCCESS
}

fn open_regular_file(dir: &mut Directory, name: &str) -> RegularFile {
    match dir.open(name, FileMode::Read, FileAttribute::empty())
        .unwrap().unwrap()
        .into_type().unwrap().unwrap()
    {
        FileType::Regular(f) => f,
        _ => panic!("{} is not regular file", name)
    }
}

fn get_file_size(file: &mut RegularFile) -> u64 {
    file.get_boxed_info::<FileInfo>().unwrap().unwrap().file_size()
}

fn load_seek_info(file: &mut RegularFile) -> alloc::vec::Vec<u32> {
    let frames = (file.get_boxed_info::<FileInfo>().unwrap().unwrap().file_size() / 4) as usize;
    let mut vec = vec![0; frames + 1];
    let mut buffer = vec![0; 4];
    for i in 1..(frames+1) {
        file.read(&mut buffer).unwrap().unwrap();
        vec[i] = LittleEndian::read_u32(&buffer);
    }
    return vec
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