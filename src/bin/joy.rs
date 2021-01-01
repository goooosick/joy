use joy::*;

use sdl2::audio::AudioCVT;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::audio::{AudioFormat, AudioFormatNum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use structopt::StructOpt;

use std::collections::VecDeque;
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

    let mut gameboy = Gameboy::new(cart);

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
    let mut audio_device = audio_system.open_playback(None, &audio_spec, |_| AudioOutput {
        samples: VecDeque::new(),
    })?;
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
    audio_device.resume();

    let mut event_pump = sdl_context.event_pump()?;
    let mut paused = false;

    const INTERVAL: Duration = Duration::from_nanos(16666667);
    let mut time = Instant::now() - INTERVAL;

    // main loop
    'running: loop {
        let cycles = (time.elapsed().as_secs_f32() * GB_CLOCK_SPEED as f32) as u32;
        time = Instant::now();

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
                gameboy.emulate(
                    cycles,
                    JoypadState {
                        left: keyboard.is_scancode_pressed(Scancode::Left),
                        right: keyboard.is_scancode_pressed(Scancode::Right),
                        up: keyboard.is_scancode_pressed(Scancode::Up),
                        down: keyboard.is_scancode_pressed(Scancode::Down),
                        start: keyboard.is_scancode_pressed(Scancode::C),
                        select: keyboard.is_scancode_pressed(Scancode::V),
                        button_a: keyboard.is_scancode_pressed(Scancode::Z),
                        button_b: keyboard.is_scancode_pressed(Scancode::X),
                    },
                );
            }

            // audio
            {
                let buffer = gameboy.consume_audio_buffer();

                let samples = std::mem::replace(buffer, Vec::new());
                *buffer = audio_cvt.convert(samples);

                audio_device.lock().samples.extend(buffer.iter());

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

        std::thread::sleep(INTERVAL.checked_sub(time.elapsed()).unwrap_or_default());
    }

    Ok(())
}

struct AudioOutput {
    samples: VecDeque<u8>,
}

impl AudioCallback for AudioOutput {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        let len = out.len().min(self.samples.len());

        out[..len].copy_from_slice(&self.samples.drain(..len).collect::<Vec<_>>());
        out[len..]
            .iter_mut()
            .for_each(|x| *x = Self::Channel::SILENCE);
    }
}
