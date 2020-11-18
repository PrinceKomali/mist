extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::{Event};
use sdl2::ttf;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::time::{Instant, Duration};
use std::thread;

use crate::render;
use crate::timing;

const SPLITS_ON_SCREEN: usize = 8; //used to limit number of splits displayed

pub struct App {
	context: sdl2::Sdl,
	ev_pump: sdl2::EventPump,
	canvas: Canvas<Window>,
	ttf: sdl2::ttf::Sdl2TtfContext,
	state: TimerState
}

enum TimerState {
	Running,
	Paused
}


impl App {
	pub fn init() -> Self {
		//sdl setup boilerplate
		let context = sdl2::init().expect("could not initialize SDL");
		let video = context.video().expect("could not initialize SDL video");
		let window = video.window("mist", 300, 500)
			.position_centered()
			.resizable()
			.build()
			.expect("could not initialize SDL window");
		let canvas = window.into_canvas().build().expect("could not initialize SDL canvas");
		let ttf = ttf::init().expect("could not initialize TTF subsystem");
		let ev_pump = context.event_pump().expect("could not initialize SDL event handler");
		App {
			context: context,
			ev_pump: ev_pump,
			canvas: canvas,
			ttf: ttf,
			state: TimerState::Paused
		}
	}

	pub fn run(&mut self) {
		self.canvas.clear();

		let mut current_index = SPLITS_ON_SCREEN;
		let timer_font = self.ttf.load_font("assets/segoe-ui-bold.ttf", 60).expect("could not open font file");
		let font = self.ttf.load_font("assets/segoe-ui-bold.ttf", 30).expect("could not open font file");
		let creator = self.canvas.texture_creator();

		let splits = App::get_splits();
		let mut on_screen: Vec<Texture> = vec![];
		for item in splits[0..current_index].iter() {
			let text_surface = font.render(item).blended(Color::WHITE).unwrap();
			let texture = creator.create_texture_from_surface(&text_surface).unwrap();
			on_screen.push(texture);
		}
		
		let mut frame_time: Instant;
		let mut total_time = Instant::now();
		let mut time_str: String;
		self.canvas.present();
		'running: loop {
			self.canvas.clear();
			frame_time = Instant::now();
			for event in self.ev_pump.poll_iter() {
				if let Event::KeyDown { scancode, .. } = event {
					println!("{:?}", scancode);
				}
				match event {
					Event::Quit {..} |
					Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
						break 'running
					},
					Event::KeyDown { keycode: Some(Keycode::Space), .. } | Event::MouseWheel { y: -1, .. } => {
						if current_index < splits.len() {
							current_index += 1;
							on_screen = vec![];
							for item in splits[current_index - SPLITS_ON_SCREEN..current_index].iter() {
								let text_surface = font.render(item).blended(Color::WHITE).unwrap();
								let texture = creator.create_texture_from_surface(&text_surface).unwrap();
								on_screen.push(texture);
							}
						}
					},
					Event::MouseWheel { y: 1, .. } => {
						if current_index != SPLITS_ON_SCREEN {
							current_index -= 1;
							on_screen = vec![];
							for item in splits[current_index - SPLITS_ON_SCREEN..current_index].iter() {
								let text_surface = font.render(item).blended(Color::WHITE).unwrap();
								let texture = creator.create_texture_from_surface(&text_surface).unwrap();
								on_screen.push(texture);
							}
						}
					},
					Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
						if let TimerState::Paused = self.state {
							self.state = TimerState::Running;
							total_time = Instant::now();
						} else {
							self.state = TimerState::Paused;
						}

					}
					_ => {}
				}
			}
			let window_width = self.canvas.viewport().width();
			render::render_rows(&on_screen, &mut self.canvas, window_width);
			if let TimerState::Running = self.state {
				time_str = timing::ms_to_readable(total_time.elapsed().as_millis());
				let time_surface = font.render(&time_str).shaded(Color::WHITE, Color::BLACK).unwrap();
				let texture = creator.create_texture_from_surface(&time_surface).unwrap();
				render::render_time(texture, &mut self.canvas);
			}
			self.canvas.present();
			thread::sleep(Duration::new(0, 1_000_000_000 / 60) - Instant::now().duration_since(frame_time));
		}
	}
	//will move to something like `parser.rs` once split files are a thing
	fn get_splits() -> [&'static str; 11] {
		["Something", "else", "words", "text", "split 5 idk", "q", "asdf", "words 2", "no", "yes", "another one"]
	}
}
