use joy::*;

use sdl2::audio::AudioCVT;
use sdl2::audio::AudioFormat;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use structopt::StructOpt;

use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, StructOpt)]
#[structopt(name = "Joy", about = "A gameboy emulator.")]
struct Args {
    /// Gameboy cartridge.
    #[structopt(name = "FILE")]
    file: String,

    /// Window scaling.
    #[structopt(short = "s", long = "scale", default_value = "2")]
    scale: u32,
}

fn main() -> Result<(), String> {
    let args = Args::from_args();

    let cart = load_cartridge(args.file).expect("load cartridge failed");
    let title = cart.title();

    let mut gameboy = GameBoy::new(cart);

    let sdl_context = sdl2::init()?;

    // window
    let video_system = sdl_context.video()?;
    let window = video_system
        .window(
            format!("Joy - {}", title).as_str(),
            GB_LCD_WIDTH as u32 * args.scale,
            GB_LCD_HEIGHT as u32 * args.scale,
        )
        .resizable()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            GB_LCD_WIDTH as u32,
            GB_LCD_HEIGHT as u32,
        )
        .map_err(|e| e.to_string())?;

    // audio
    let audio_system = sdl_context.audio()?;
    let audio_spec = AudioSpecDesired {
        freq: None,
        channels: Some(2),
        samples: None,
    };
    let audio_device = audio_system.open_queue::<u8, _>(None, &audio_spec)?;
    {
        let spec = audio_device.spec();
        println!("audio spec: ");
        println!("    driver: {}", audio_system.current_audio_driver());
        println!("    channels: {}", spec.channels);
        println!("    frequency: {}", spec.freq);
        println!("    buffer size: {} * {}", spec.samples, spec.channels);
    }
    let audio_cvt = AudioCVT::new(
        AudioFormat::U8,
        2,
        (GB_CLOCK_SPEED / AUDIO_FREQ_DIVIDER) as i32,
        AudioFormat::U8,
        2,
        audio_device.spec().freq,
    )?;

    let duration = Duration::new(0, 1_000_000_000u32 / 60);
    let mut event_pump = sdl_context.event_pump()?;

    audio_device.resume();

    let mut paused = false;

    // main loop
    'running: loop {
        let current = Instant::now();

        // events
        {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::LShift),
                        ..
                    } => paused = !paused,
                    Event::KeyDown {
                        keycode: Some(Keycode::S),
                        ..
                    } => gameboy.save_game(),
                    _ => {}
                }
            }
        }

        if !paused {
            // emulate
            {
                let keyboard = event_pump.keyboard_state();
                gameboy.emulate(JoypadState {
                    left: keyboard.is_scancode_pressed(Scancode::Left),
                    right: keyboard.is_scancode_pressed(Scancode::Right),
                    up: keyboard.is_scancode_pressed(Scancode::Up),
                    down: keyboard.is_scancode_pressed(Scancode::Down),
                    start: keyboard.is_scancode_pressed(Scancode::C),
                    select: keyboard.is_scancode_pressed(Scancode::V),
                    button_a: keyboard.is_scancode_pressed(Scancode::Z),
                    button_b: keyboard.is_scancode_pressed(Scancode::X),
                });
            }

            // audio
            {
                let buffer = gameboy.consume_audio_buffer();

                let samples = std::mem::replace(buffer, Vec::new());
                std::mem::replace(buffer, audio_cvt.convert(samples));

                audio_device.queue(buffer.as_slice());
                buffer.clear();
            }

            // graphics
            {
                texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                    buffer.copy_from_slice(gameboy.get_frame_buffer());
                })?;
                canvas.copy(&texture, None, None)?;
                canvas.present();
            }
        }

        if let Some(time) = duration.checked_sub(current.elapsed()) {
            thread::sleep(time);
        } else {
            eprintln!("frame takes too long!!!");
        }
    }

    Ok(())
}
