use joy::*;

use sdl2::audio::AudioCVT;
use sdl2::audio::AudioFormat;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use structopt::StructOpt;

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

struct AudioOutput {
    gb: Arc<Mutex<GameBoy>>,
    signal: Sender<()>,
}

impl AudioCallback for AudioOutput {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        let mut gb = self.gb.lock().unwrap();
        let buffer = gb.consume_audio_buffer();

        if buffer.len() >= out.len() {
            out.copy_from_slice(&buffer[..out.len()]);
        }
        buffer.clear();

        self.signal.send(()).unwrap();
    }
}

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

    let gameboy = Arc::new(Mutex::new(GameBoy::new(cart)));
    let gb_audio = Arc::clone(&gameboy);
    let (sender, receiver) = channel();

    let sdl_context = sdl2::init()?;

    // audio
    let audio_subsystem = sdl_context.audio()?;
    let desired_spec = AudioSpecDesired {
        freq: None,
        channels: Some(2),
        samples: Some(367), // for 22050 Hz
    };
    let audio_device =
        audio_subsystem.open_playback(None, &desired_spec, move |_| AudioOutput {
            gb: gb_audio,
            signal: sender,
        })?;
    let audio_cvt = AudioCVT::new(
        AudioFormat::U8,
        2,
        (GB_CLOCK_SPEED / AUDIO_FREQ_DIVIDER) as i32,
        AudioFormat::U8,
        2,
        audio_device.spec().freq,
    )?;
    audio_device.resume();

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

    let mut event_pump = sdl_context.event_pump()?;
    let mut save_game = false;
    let mut paused = false;

    'running: loop {
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
                    } => save_game = true,
                    _ => {}
                }
            }
        }

        if let Ok(_) = receiver.try_recv() {
            let mut gb = gameboy.lock().unwrap();

            {
                let keyboard = event_pump.keyboard_state();
                gb.emulate(JoypadState {
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

            // convert audio
            {
                let buffer = gb.consume_audio_buffer();

                let samples = std::mem::replace(buffer, Vec::new());
                std::mem::replace(buffer, audio_cvt.convert(samples));
            }

            texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                buffer.copy_from_slice(gb.get_frame_buffer());
            })?;
            canvas.copy(&texture, None, None)?;
            canvas.present();

            if save_game {
                gb.save_game();
                save_game = false;
            }
        }
    }

    Ok(())
}
