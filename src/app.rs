//I have no idea what half of this stuff does - Komali


// app struct and its functions, one of which is the application mainloop
use sdl2::event::{Event, WindowEvent};
use sdl2::get_error;
#[cfg(feature = "bg")]
use sdl2::gfx::rotozoom::RotozoomSurface;
#[cfg(any(feature = "icon", feature = "bg"))]
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
#[cfg(feature = "bg")]
use sdl2::pixels::PixelFormatEnum;
#[cfg(feature = "bg")]
use sdl2::rect::Rect;
#[cfg(feature = "bg")]
use sdl2::render::TextureAccess;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf;

use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::{Duration, Instant};

use mist_core::{
    config::{Config, Panel},
    dialogs,
    parse::MsfParser,
    timing, Run,
};

use crate::comparison::Comparison;
use crate::keybinds::Keybinds;
use crate::panels::RenderPanel;
use crate::render;
use crate::splits::Split;
use crate::state::TimerState;
// struct that holds information about the running app and its state
pub struct App {
    _context: sdl2::Sdl,
    ev_pump: sdl2::EventPump,
    timer: Instant,
    canvas: WindowCanvas,
    ttf: sdl2::ttf::Sdl2TtfContext,
    state: TimerState,
    comparison: Comparison,
    run: Run,
    config: Config,
    msf: MsfParser,
}
const PI: f32 = 3.1415926535897932384626;
fn rainbow(num: i32) -> Vec<[u8; 3]> {
    let mut arr = vec![[0, 0, 0]];
    for i in 0..num {
      let r = sin_to_rgb(i, 0.0 * PI * 0.66666666667, num);
      let g = sin_to_rgb(i, 1.0 * PI * 0.66666666667, num);
      let b = sin_to_rgb(i, 2.0 * PI * 0.66666666667, num);
      arr.push([r, g, b]);
    }
    arr.remove(0);
    return arr;
}
  
fn sin_to_rgb(i: i32, b: f32, num: i32) -> u8 {
    let sin = (PI / (num as f32) * 2.0 * (i as f32) + b).sin();
    let int = (sin * 127.0).floor() as i32 + 128;
    return int as u8;
}

impl App {
    pub fn init(context: sdl2::Sdl) -> Result<Self, String> {
        // sdl setup boilerplate
        // println!("a");
        let video = context.video()?;
        let mut window = video
            .window("mist", 400, 80)
            .position_centered()
            .resizable()
            .build()
            .map_err(|_| get_error())?;
        #[cfg(feature = "icon")]
        {
            let icon = Surface::from_file("assets/MIST.png")?;
            window.set_icon(icon);
        }
        let canvas = window.into_canvas().build().map_err(|_| get_error())?;
        let ttf = ttf::init().map_err(|_| get_error())?;
        let ev_pump = context.event_pump()?;
        let config = Config::open()?;
        // start the overarching application timer (kinda)
        let timer = Instant::now();
        // make an App that hasn't started and has an empty run
        let mut app = App {
            _context: context,
            ev_pump,
            timer,
            canvas,
            ttf,
            state: TimerState::NotRunning {
                time_str: "0.000".to_owned(),
            },
            comparison: Comparison::PersonalBest,
            run: Run::empty(),
            config: config,
            msf: MsfParser::new(),
        };
        // try to use the filepath specified in the config file
        if let Some(x) = app.config.file() {
            let f = File::open(x).map_err(|e| e.to_string())?;
            let reader = BufReader::new(f);
            app.run = app.msf.parse(reader)?;
        } else {
            match dialogs::open_run() {
                Ok(r) => match r {
                    Some((run, path)) => {
                        app.run = run;
                        app.config.set_file(&path);
                    }
                    None => app.run = Run::empty(),
                },
                Err(e) => return Err(e.to_string()),
            }
        }
        return Ok(app);
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut rainbow_int: i32 = 0;
        
        let mut no_file: bool;
        let mut path = match self.config.file() {
            Some(p) => {
                no_file = false;
                p.to_owned()
            }
            None => {
                no_file = true;
                "".to_owned()
            }
        };

        self.canvas.clear();

        let mut colors = self.config.color_list();
        let mut ahead = Color::from(colors[0]);
        let mut behind = Color::from(colors[1]);
        let mut making_up_time = Color::from(colors[2]);
        let mut losing_time = Color::from(colors[3]);
        let mut gold = Color::from(colors[4]);
        let mut bg_color = Color::from(colors[5]);

        let mut did_gold = false;

        // grab font sizes from config file and load the fonts
        let sizes = self.config.fsize();
        let mut timer_font = self.ttf.load_font(self.config.tfont(), sizes.0)?;
        timer_font.set_kerning(false);
        let mut font = self.ttf.load_font(self.config.sfont(), sizes.1)?;
        // make the texture creator used a lot later on
        let creator = self.canvas.texture_creator();
        let mut binds = Keybinds::from_raw(self.config.binds())?;
        let mut panels = {
            let mut ret = vec![];
            for panel in self.config.panels() {
                let (text, paneltype) = match panel {
                    Panel::Pace { golds } => {
                        // if *golds {
                        //     ("Pace (best)", Panel::Pace { golds: true })
                        // } else {
                            ("Pace (pb)", Panel::Pace { golds: false })
                        // }
                    }
                    Panel::SumOfBest => ("Sum of Best", Panel::SumOfBest),
                    Panel::CurrentSplitDiff { golds } => {
                        // if *golds {
                        //     (
                        //         "Split (best)",
                        //         Panel::CurrentSplitDiff { golds: true },
                        //     )
                        // } else {
                            (
                                "Split (pb)",
                                Panel::CurrentSplitDiff { golds: false },
                            )
                        // }
                    }
                };
                
                // color = Color::RGB(red, green, blue);
                let text_sur = font
                    .render(text)
                    
                    .blended(Color::WHITE)
                    .map_err(|_| get_error())?;
                let text_tex = creator
                    .create_texture_from_surface(&text_sur)
                    .map_err(|_| get_error())?;
                let time_sur = if let Panel::SumOfBest = panel {
                    let sob = self.run.gold_times().iter().sum::<u128>();
                    font.render(&timing::split_time_text(sob))
                        .blended(Color::WHITE)
                        .map_err(|_| get_error())?
                } else {
                    font.render("-  ")
                        .blended(Color::WHITE)
                        .map_err(|_| get_error())?
                };
                let time_tex = creator
                    .create_texture_from_surface(&time_sur)
                    .map_err(|_| get_error())?;
                let newpanel = RenderPanel::new(text_tex, time_tex, paneltype);
                ret.push(newpanel);
            }
            ret
        };

        #[cfg(feature = "bg")]
        let mut has_bg: bool;
        #[cfg(feature = "bg")]
        let mut bg_tex: Texture;
        #[cfg(feature = "bg")]
        let mut bg_rect: Rect;
        #[cfg(feature = "bg")]
        {
            let bg: Option<Surface> = match self.config.img() {
                Some(ref p) => Some(Surface::from_file(&p)?),
                None => None,
            };
            if let Some(x) = bg {
                has_bg = true;
                let width = self.canvas.viewport().width();
                let height = self.canvas.viewport().height();
                if !self.config.img_scaled() {
                    let mut sur = Surface::new(width, height, PixelFormatEnum::RGB24)?;
                    let cutoffx = {
                        if x.width() > width {
                            ((x.width() - width) / 2) as i32
                        } else {
                            0
                        }
                    };
                    let cutoffy = {
                        if x.height() > height {
                            ((x.height() - height) / 2) as i32
                        } else {
                            0
                        }
                    };
                    x.blit(Rect::new(cutoffx, cutoffy, width, height), &mut sur, None)?;
                    bg_tex = creator
                        .create_texture_from_surface(&sur)
                        .map_err(|_| get_error())?;
                } else {
                    let sur: Surface;
                    if x.width() > x.height() && width < x.width() {
                        if width < x.width() {
                            sur = x.rotozoom(0.0, width as f64 / x.width() as f64, true)?;
                        } else {
                            sur = x.rotozoom(0.0, x.width() as f64 / width as f64, true)?;
                        }
                    } else {
                        if height < x.height() {
                            sur = x.rotozoom(0.0, height as f64 / x.height() as f64, true)?;
                        } else {
                            sur = x.rotozoom(0.0, x.height() as f64 / height as f64, true)?;
                        }
                    }
                    bg_tex = creator
                        .create_texture_from_surface(&sur)
                        .map_err(|_| get_error())?;
                }
            } else {
                has_bg = false;
                bg_tex = creator
                    .create_texture(None, TextureAccess::Static, 1, 1)
                    .map_err(|_| get_error())?;
            }
            let sdl2::render::TextureQuery {
                width: bgw,
                height: bgh,
                ..
            } = bg_tex.query();
            bg_rect = Rect::new(0, 0, bgw, bgh);
        }
        // get the heights of different font textures
        let mut splits_height = font
            .size_of("qwertyuiopasdfghjklzxcvbnm01234567890!@#$%^&*(){}[]|\\:;'\",.<>?/`~-_=+")
            .map_err(|_| get_error())?
            .1;
        // get the x-coordinates of characters in the font spritemap
        let mut coords: Vec<u32> = {
            let mut raw: Vec<u32> = vec![];
            let mut ret: Vec<u32> = vec![0];
            for chr in "-0123456789:. ".chars() {
                let size = timer_font
                    .size_of(&chr.to_string())
                    .map_err(|_| get_error())?;
                raw.push(size.0);
                ret.push(raw.iter().sum::<u32>());
            }
            ret.push(*raw.iter().max().unwrap());

            ret
        };
        let mut font_y = timer_font
            .size_of("-0123456789:.")
            .map_err(|_| get_error())?
            .1;
        // render initial white font map. gets overwritten when color changes
        timer_font.set_outline_width(0);
        let mut map = timer_font //hi peri
            .render("- 0 1 2 3 4 5 6 7 8 9 : .")
            
            .blended(Color::WHITE)
            .map_err(|_| get_error())?;
        let mut map_tex = creator //I'll do another one like this
            .create_texture_from_surface(&map)
            .map_err(|_| get_error())?;
        map = timer_font //hi peri
            .render("- 0 1 2 3 4 5 6 7 8 9 : .")
            
            .blended(Color::WHITE)
            .map_err(|_| get_error())?;
        let mut map_tex_outline = creator //I'll do another one like this
            .create_texture_from_surface(&map)
            .map_err(|_| get_error())?;
        // set the height where overlap with splits is checked when resizing window
        let mut timer_height = font_y + splits_height;
        // set the minimum height of the window to the size of the time texture
        self.canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 20 + (splits_height * panels.len() as u32))
            .map_err(|_| get_error())?;
        self.canvas
            .window_mut()
            .set_size(400, 82)
            .map_err(|_| get_error())?;
        self.canvas
            .window_mut()
            .set_title("mist")
            .map_err(|_| get_error())?;

        // get first vec of split name textures from file
        let mut split_names = self.run.splits();
        let mut offset = self.run.offset();
        // if there is an offset, display it properly
        match offset {
            Some(x) => {
                self.state = TimerState::NotRunning {
                    time_str: format!("-{}", timing::ms_to_readable(x, false)),
                };
            }
            _ => {}
        }
        // get ms split times then convert them to pretty, summed times
        let split_times_ms: Vec<u128> = self.run.pb_times().iter().cloned().collect();
        let mut summed_times = timing::split_time_sum(&split_times_ms);
        let split_times_raw: Vec<String> = summed_times
            .iter()
            .map(|val| timing::split_time_text(*val))
            .collect();
        // initialize variables that are used in the loop for replacing textures
        let mut text_surface: Surface;
        let mut texture: Texture;
        // vectors that hold the textures for split names and their associated times
        let mut splits: Vec<Split> = vec![];

        let mut index = 0;
        // convert the split names into textures and add them to the split name vec
        while index < split_names.len() {
            text_surface = font
                .render(&split_names[index])
                .blended(Color::WHITE)
                .map_err(|_| get_error())?;
            texture = creator
                .create_texture_from_surface(&text_surface)
                .map_err(|_| get_error())?;
            let comp = font
                .render(&split_times_raw[index])
                .blended(Color::WHITE)
                .map_err(|_| get_error())?;
            let comp_texture = creator
                .create_texture_from_surface(&comp)
                .map_err(|_| get_error())?;
            // create split struct with its corresponding times and textures
            let split = Split::new(
                split_times_ms[index],
                self.run.gold_times()[index],
                0,
                None,
                texture,
                comp_texture,
                None,
            );
            splits.push(split);
            index += 1;
        }

        let mut bottom_split_index: usize;
        let mut top_split_index = 0;
        let mut max_splits: usize;

        // if there are too few splits then set the max splits to the number of splits rather than
        // the max allowed amount
        let max_initial_splits: usize = ((500 - timer_height) / splits_height) as usize;
        if splits.len() == 0 {
            max_splits = 0;
            bottom_split_index = 0;
        } else if max_initial_splits > splits.len() {
            bottom_split_index = splits.len() - 1;
            max_splits = splits.len();
        } else {
            max_splits = max_initial_splits;
            bottom_split_index = max_initial_splits - 1;
        }
        // drop stuff that isnt needed after initializing
        drop(split_times_ms);
        drop(split_times_raw);
        //drop(split_names);
        drop(map);

        // set up variables used in the mainloop
        // framerate cap timer
        let mut frame_time: Instant;
        // display time
        let mut time_str: String;
        let mut time_str_outline: String;
        // keep track of amount of time that passed before the timer was paused
        let mut before_pause = 0;
        let mut before_pause_split = 0;
        // this one should be a static but duration isnt allowed to be static apparently
        let one_sixtieth = Duration::new(0, 1_000_000_000 / 60);
        // active split's index
        let mut current_split = 0;
        // width of the canvas
        let mut window_width: u32;
        // color of text
        let mut color = Color::WHITE;
        // used to determine if timer font map should be rerendered
        let mut old_color: Color;
        // diff between max on screen and current, used when resizing window
        let mut diff: usize;
        // number of splits
        let mut len: usize = splits.len();
        // current split in the slice of splits sent to render_time()
        let mut cur: usize;
        // elapsed time when last split happened
        let mut split_ticks = 0;
        let mut start_ticks = 0;
        // split times of current run
        let mut active_run_times: Vec<u128> = vec![];
        // variable used to hold elapsed milliseconds of the application timer
        let mut elapsed: u128;
        // set when a run ends and is a pb to signal for a pop-up window to ask if the user wants to save
        let mut save = false;
        // set when comparison has changed and textures need to be rerendered
        let mut comp_changed = false;
        self.canvas.present();

        // main loop
        'running: loop {
            rainbow_int = (rainbow_int + 11)%2500;
            // start measuring the time this loop pass took
            frame_time = Instant::now();
            // remove stuff from the backbuffer and fill the space with black
            self.canvas.set_draw_color(bg_color);
            self.canvas.clear();

            #[cfg(feature = "bg")]
            if has_bg {
                self.canvas.copy(&bg_tex, None, bg_rect)?;
            }
            // if the timer is doing an offset, make sure it should still be negative
            // if it shouldnt, convert to running state
            if let TimerState::OffsetCountdown { amt } = self.state {
                elapsed = self.timer.elapsed().as_millis();
                if amt <= elapsed - start_ticks {
                    self.state = TimerState::Running { timestamp: elapsed };
                    split_ticks = elapsed;
                    start_ticks = elapsed;
                }
            }
            // repeat stuff in here for every event that occured between frames
            // in order to properly respond to them
            for event in self.ev_pump.poll_iter() {
                // print events to terminal if running in debug
                #[cfg(debug_assertions)]
                println!("{:?}", event);

                match event {
                    // quit program on esc or being told by wm to close
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    // if scroll down and there are enough splits, scroll splits down
                    Event::MouseWheel { y: -1, .. } => {
                        if len != 0 && bottom_split_index < len - 1 {
                            bottom_split_index += 1;
                            top_split_index += 1;
                        }
                    }

                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if top_split_index != 0 {
                            bottom_split_index -= 1;
                            top_split_index -= 1;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(k),
                        repeat: false,
                        ..
                    } => {
                        if k == binds.start_split {
                            match self.state {
                                // if timer isnt started, start it.
                                TimerState::NotRunning { .. } if current_split == 0 => {
                                    elapsed = self.timer.elapsed().as_millis();
                                    start_ticks = elapsed;
                                    split_ticks = elapsed;
                                    self.canvas
                                        .window_mut()
                                        .set_title("mist")
                                        .map_err(|_| get_error())?;
                                    match offset {
                                        // if we are in the start offset, tell it to offset
                                        Some(x) => {
                                            self.state = TimerState::OffsetCountdown { amt: x };
                                        }
                                        None => {
                                            self.state = TimerState::Running { timestamp: elapsed };
                                        }
                                    }
                                }
                                // if it is running, either split or end
                                TimerState::Running { timestamp: t, .. } => {
                                    // only try to do this stuff if there is at least one split
                                    if len != 0 {
                                        elapsed = self.timer.elapsed().as_millis();
                                        active_run_times.push(
                                            (elapsed - split_ticks) + before_pause_split,
                                        );
                                        let sum = self.run.sum_times()[current_split];
                                        self.run.set_sum_time(
                                            (
                                                sum.0 + 1,
                                                sum.1
                                                    + ((elapsed - split_ticks)
                                                        + before_pause_split),
                                            ),
                                            current_split,
                                        );
                                        split_ticks = elapsed;
                                        before_pause_split = 0;
                                        // create the difference time shown after a split
                                        let sum = timing::split_time_sum(&active_run_times)
                                            [current_split];
                                        let diff =
                                            sum as i128 - summed_times[current_split] as i128;
                                        time_str = timing::diff_text(diff);
                                        // set diff color to gold and replace split gold
                                        if active_run_times[current_split]
                                            < splits[current_split].gold()
                                            || splits[current_split].gold() == 0
                                        {
                                            save = true;
                                            did_gold = true;
                                            // color = gold;
                                            self.run.set_gold_time(
                                                active_run_times[current_split],
                                                current_split,
                                            );
                                            splits[current_split]
                                                .set_gold(active_run_times[current_split]);
                                        }
                                        text_surface = font
                                            .render(&time_str)
                                            .blended(color)
                                            .map_err(|_| get_error())?;
                                        texture = creator
                                            .create_texture_from_surface(&text_surface)
                                            .map_err(|_| get_error())?;
                                        splits[current_split].set_diff(diff, Some(texture));
                                        time_str =
                                            timing::split_time_text((elapsed - t) + before_pause);
                                        text_surface = font
                                            .render(&time_str)
                                            .blended(Color::WHITE)
                                            .map_err(|_| get_error())?;
                                        texture = creator
                                            .create_texture_from_surface(&text_surface)
                                            .map_err(|_| get_error())?;
                                        splits[current_split].set_cur(Some(texture));
                                        // update the comparison texture if we are looking at average, because the average
                                        // will have changed
                                        if let Comparison::Average = self.comparison {
                                            let sum = self.run.sum_times()[current_split];
                                            let tm = sum.1 / sum.0;
                                            text_surface = font
                                                .render(&timing::split_time_text(tm))
                                                .blended(Color::WHITE)
                                                .map_err(|_| get_error())?;
                                            texture = creator
                                                .create_texture_from_surface(&text_surface)
                                                .map_err(|_| get_error())?;
                                            splits[current_split].set_comp_tex(texture);
                                        }
                                        // if there are still splits left, continue the run and advance the current split
                                        if current_split < len - 1 {
                                            current_split += 1;
                                            self.canvas
                                                .window_mut()
                                                .set_title("mist")
                                                .map_err(|_| get_error())?;
                                            // if the next split is offscreen set recreate_on_screen flag to change the current split slice
                                            if current_split > bottom_split_index
                                                && bottom_split_index + 1 < len
                                            {
                                                bottom_split_index += 1;
                                                top_split_index += 1;
                                            }
                                        // otherwise end the run
                                        } else {
                                            current_split += 1;
                                            self.canvas
                                                .window_mut()
                                                .set_title("mist")
                                                .map_err(|_| get_error())?;
                                            // set the state of the timer to finished, round string to 30fps
                                            self.state = TimerState::NotRunning {
                                                time_str: timing::ms_to_readable(
                                                    (elapsed - t) + before_pause,
                                                    true,
                                                ),
                                            };
                                            // if this run was a pb then set the Run struct's pb and splits
                                            if (elapsed - t) + before_pause < self.run.pb()
                                                || self.run.pb() == 0
                                            {
                                                no_file = false;
                                                index = 0;
                                                summed_times =
                                                    timing::split_time_sum(&active_run_times);
                                                let split_times_raw: Vec<String> = summed_times
                                                    .iter()
                                                    .map(|val| timing::split_time_text(*val))
                                                    .collect();
                                                while index < len {
                                                    text_surface = font
                                                        .render(&split_times_raw[index])
                                                        .blended(Color::WHITE)
                                                        .map_err(|_| get_error())?;
                                                    texture = creator
                                                        .create_texture_from_surface(text_surface)
                                                        .map_err(|_| get_error())?;
                                                    splits[index].set_comp_tex(texture);
                                                    splits[index].set_cur(None);
                                                    splits[index].set_time(active_run_times[index]);
                                                    index += 1;
                                                }
                                                save = true;
                                                self.run.set_pb((elapsed - t) + before_pause);
                                                self.run.set_pb_times(&active_run_times);
                                                active_run_times = vec![];
                                            }
                                        }
                                    // finish the run if there are no splits
                                    } else {
                                        self.canvas
                                            .window_mut()
                                            .set_title("mist")
                                            .map_err(|_| get_error())?;
                                        elapsed = self.timer.elapsed().as_millis();
                                        if no_file {
                                            no_file = false;
                                            save = true;
                                            self.run.set_pb((elapsed - t) + before_pause);
                                            self.run.set_pb_times(&active_run_times);
                                            self.run.set_gold_times(&active_run_times);
                                        }
                                        self.state = TimerState::NotRunning {
                                            time_str: timing::ms_to_readable(
                                                (elapsed - t) + before_pause,
                                                true,
                                            ),
                                        };
                                    }
                                }
                                _ => {}
                            }
                        } else if k == binds.pause {
                            elapsed = self.timer.elapsed().as_millis();
                            match self.state {
                                // if timer is paused, unpause it, put the amount of time before the pause in a variable
                                // and set the state to running
                                TimerState::Paused {
                                    time: t, split: s, ..
                                } => {
                                    self.canvas
                                        .window_mut()
                                        .set_title("mist")
                                        .map_err(|_| get_error())?;
                                    start_ticks = elapsed;
                                    split_ticks = elapsed;
                                    before_pause = t;
                                    before_pause_split = s;
                                    self.state = TimerState::Running { timestamp: elapsed };
                                }
                                // if the timer is already running, set it to paused.
                                TimerState::Running { .. } => {
                                    self.canvas
                                        .window_mut()
                                        .set_title("mist")
                                        .map_err(|_| get_error())?;
                                    elapsed = self.timer.elapsed().as_millis();
                                    self.state = TimerState::Paused {
                                        time: (elapsed - start_ticks) + before_pause,
                                        split: (elapsed - split_ticks) + before_pause_split,
                                        time_str: timing::ms_to_readable(
                                            (elapsed - start_ticks) + before_pause,
                                            true,
                                        ),
                                    };
                                }
                                _ => {}
                            }
                        } else if k == binds.reset {
                            self.canvas
                                .window_mut()
                                .set_title("mist")
                                .map_err(|_| get_error())?;
                            // reset stuff specific to the active run and return splits to the top of the list
                            active_run_times = vec![];
                            top_split_index = 0;
                            if max_splits != 0 {
                                bottom_split_index = max_splits - 1;
                            } else {
                                bottom_split_index = 0;
                            }
                            before_pause = 0;
                            before_pause_split = 0;
                            current_split = 0;
                            color = ahead;
                            // println!("{:?}", offset);
                            // if there is an offset, reset the timer to that, if not, reset timer to 0
                            match offset {
                                Some(x) => {
                                    self.state = TimerState::NotRunning {
                                        time_str: format!("-{}", timing::ms_to_readable(x, false)),
                                    };
                                }
                                None => {
                                    self.state = TimerState::NotRunning {
                                        time_str: "0.000".to_owned(),
                                    };
                                }
                            }
                            index = 0;
                            // get rid of run-specific active times and differences
                            while index < len {
                                splits[index].set_cur(None);
                                splits[index].set_diff(0, None);
                                index += 1;
                            }
                        } else if k == binds.prev_comp {
                            self.comparison.prev();
                            comp_changed = true;
                        } else if k == binds.next_comp {
                            self.comparison.next();
                            comp_changed = true;
                        } else if k == binds.load_splits {
                            // only allow opening a new file if the timer is not running
                            if let TimerState::NotRunning { .. } = self.state {
                                // save the previous run if it was updated
                                if save && dialogs::save_check() {
                                    if path == "" {
                                        let p = dialogs::get_save_as();
                                        match p {
                                            Some(s) => {
                                                path = s;
                                                let mut f = File::create(&path)
                                                    .map_err(|e| e.to_string())?;
                                                self.msf.write(&self.run, &mut f)?;
                                            }
                                            None => {}
                                        }
                                    } else {
                                        let mut f = File::open(&path).map_err(|e| e.to_string())?;
                                        self.msf.write(&self.run, &mut f)?;
                                    }
                                }
                                // open a file dialog to get a new split file + run
                                // if the user cancelled, do nothing
                                match dialogs::open_run() {
                                    Ok(s) => match s {
                                        Some((r, p)) => {
                                            self.run = r;
                                            path = p;
                                        }
                                        _ => {}
                                    },
                                    Err(e) => return Err(e.to_string()),
                                }
                                offset = self.run.offset();
                                // if there is an offset, display it properly
                                match offset {
                                    Some(x) => {
                                        self.state = TimerState::NotRunning {
                                            time_str: format!(
                                                "-{}",
                                                timing::ms_to_readable(x, false)
                                            ),
                                        };
                                    }
                                    _ => {}
                                }
                                // recreate split names, times, textures, etc
                                split_names = self.run.splits();
                                let split_times_ms: Vec<u128> =
                                    self.run.pb_times().iter().cloned().collect();
                                summed_times = timing::split_time_sum(&split_times_ms);
                                let split_times_raw: Vec<String> = summed_times
                                    .iter()
                                    .map(|val| timing::split_time_text(*val))
                                    .collect();
                                splits = vec![];
                                index = 0;
                                while index < split_names.len() {
                                    text_surface = font
                                        .render(&split_names[index])
                                        .blended(Color::WHITE)
                                        .map_err(|_| get_error())?;
                                    texture = creator
                                        .create_texture_from_surface(&text_surface)
                                        .map_err(|_| get_error())?;
                                    let comp = font
                                        .render(&split_times_raw[index])
                                        .blended(Color::WHITE)
                                        .map_err(|_| get_error())?;
                                    let comp_texture = creator
                                        .create_texture_from_surface(&comp)
                                        .map_err(|_| get_error())?;
                                    let split = Split::new(
                                        split_times_ms[index],
                                        self.run.gold_times()[index],
                                        0,
                                        None,
                                        texture,
                                        comp_texture,
                                        None,
                                    );
                                    splits.push(split);
                                    index += 1;
                                }
                                // reset max splits, length of splits, and split indices to reflect new run
                                if len == 0 {
                                    max_splits = ((self.canvas.viewport().height() - timer_height)
                                        / splits_height)
                                        as usize;
                                }
                                len = splits.len();
                                if max_splits > len {
                                    max_splits = len;
                                }
                                top_split_index = 0;
                                if max_splits != 0 {
                                    bottom_split_index = max_splits - 1;
                                } else {
                                    bottom_split_index = 0;
                                }
                            }
                        } else if k == binds.skip_split {
                            // can only skip while running
                            if let TimerState::Running { timestamp: t } = self.state {
                                // push a zero to active times. Will eventually handle zeroes properly but Not Yet (tm)
                                active_run_times.push(0);
                                text_surface = font
                                    .render("-  ")
                                    .blended(Color::WHITE)
                                    .map_err(|_| get_error())?;
                                texture = creator
                                    .create_texture_from_surface(&text_surface)
                                    .map_err(|_| get_error())?;
                                splits[current_split].set_comp_tex(texture);
                                // if this is the last split, end but we don't have to worry about setting pb and stuff
                                // otherwise just increment current split and move on
                                if len == 0 || len == 1 || current_split == len - 1 {
                                    elapsed = self.timer.elapsed().as_millis();
                                    self.canvas
                                        .window_mut()
                                        .set_title("mist")
                                        .map_err(|_| get_error())?;
                                    self.state = TimerState::NotRunning {
                                        time_str: timing::ms_to_readable(
                                            (elapsed - t) + before_pause,
                                            true,
                                        ),
                                    };
                                } else if current_split < len - 1 {
                                    current_split += 1;
                                    if current_split > bottom_split_index
                                        && bottom_split_index + 1 < len
                                    {
                                        bottom_split_index += 1;
                                        top_split_index += 1;
                                    }
                                }
                            }
                        } else if k == binds.load_config {
                            match dialogs::open_config() {
                                Ok(c) => match c {
                                    Some(conf) => {
                                        self.config = conf;
                                        binds = Keybinds::from_raw(self.config.binds())?;
                                        colors = self.config.color_list();
                                        ahead = Color::from(colors[0]);
                                        behind = Color::from(colors[1]);
                                        making_up_time = Color::from(colors[2]);
                                        losing_time = Color::from(colors[3]);
                                        gold = Color::from(colors[4]);
                                        bg_color = Color::from(colors[5]);
                                        timer_font = self.ttf.load_font(
                                            self.config.tfont(),
                                            self.config.fsize().0,
                                        )?;
                                        font = self.ttf.load_font(
                                            self.config.sfont(),
                                            self.config.fsize().1,
                                        )?;
                                        splits_height = font.size_of("qwertyuiopasdfghjklzxcvbnm01234567890!@#$%^&*(){}[]|\\:;'\",.<>?/`~-_=+").map_err(|_| get_error())?.1;
                                        coords = {
                                            let mut raw: Vec<u32> = vec![];
                                            let mut ret: Vec<u32> = vec![0];
                                            for chr in "-0123456789:. ".chars() {
                                                let size = timer_font
                                                    .size_of(&chr.to_string())
                                                    .map_err(|_| get_error())?;
                                                raw.push(size.0);
                                                ret.push(raw.iter().sum::<u32>());
                                            }
                                            ret.push(*raw.iter().max().unwrap());

                                            ret
                                        };
                                        font_y = timer_font
                                            .size_of("-0123456789:.")
                                            .map_err(|_| get_error())?
                                            .1;
                                        let map = timer_font
                                            .render("- 0 1 2 3 4 5 6 7 8 9 : .")
                                            .blended(Color::WHITE)
                                            .map_err(|_| get_error())?;
                                        map_tex = creator
                                            .create_texture_from_surface(&map)
                                            .map_err(|_| get_error())?;
                                        timer_height = font_y + splits_height;
                                        self.canvas
                                            .window_mut()
                                            .set_minimum_size(0, timer_height + 20)
                                            .map_err(|_| get_error())?;
                                        #[cfg(feature = "bg")]
                                        {
                                            let bg: Option<Surface> = match self.config.img() {
                                                Some(ref p) => Some(Surface::from_file(&p)?),
                                                None => None,
                                            };
                                            if let Some(x) = bg {
                                                has_bg = true;
                                                let width = self.canvas.viewport().width();
                                                let height = self.canvas.viewport().height();
                                                if !self.config.img_scaled() {
                                                    let mut sur = Surface::new(
                                                        width,
                                                        height,
                                                        PixelFormatEnum::RGB24,
                                                    )?;
                                                    let cutoffx = {
                                                        if x.width() > width {
                                                            ((x.width() - width) / 2) as i32
                                                        } else {
                                                            0
                                                        }
                                                    };
                                                    let cutoffy = {
                                                        if x.height() > height {
                                                            ((x.height() - height) / 2) as i32
                                                        } else {
                                                            0
                                                        }
                                                    };
                                                    x.blit(
                                                        Rect::new(cutoffx, cutoffy, width, height),
                                                        &mut sur,
                                                        None,
                                                    )?;
                                                    bg_tex = creator
                                                        .create_texture_from_surface(&sur)
                                                        .map_err(|_| get_error())?;
                                                } else {
                                                    let sur: Surface;
                                                    if x.width() > x.height() && width < x.width() {
                                                        if width < x.width() {
                                                            sur = x.rotozoom(
                                                                0.0,
                                                                width as f64 / x.width() as f64,
                                                                true,
                                                            )?;
                                                        } else {
                                                            sur = x.rotozoom(
                                                                0.0,
                                                                x.width() as f64 / width as f64,
                                                                true,
                                                            )?;
                                                        }
                                                    } else {
                                                        if height < x.height() {
                                                            sur = x.rotozoom(
                                                                0.0,
                                                                height as f64 / x.height() as f64,
                                                                true,
                                                            )?;
                                                        } else {
                                                            sur = x.rotozoom(
                                                                0.0,
                                                                x.height() as f64 / height as f64,
                                                                true,
                                                            )?;
                                                        }
                                                    }
                                                    bg_tex = creator
                                                        .create_texture_from_surface(&sur)
                                                        .map_err(|_| get_error())?;
                                                }
                                            } else {
                                                has_bg = false;
                                                bg_tex = creator
                                                    .create_texture(
                                                        None,
                                                        TextureAccess::Static,
                                                        1,
                                                        1,
                                                    )
                                                    .map_err(|_| get_error())?;
                                            }
                                            let sdl2::render::TextureQuery {
                                                width: bgw,
                                                height: bgh,
                                                ..
                                            } = bg_tex.query();
                                            bg_rect = Rect::new(0, 0, bgw, bgh);
                                        }
                                    }
                                    None => {}
                                },
                                Err(e) => return Err(e),
                            }
                        }
                    }
                    // handle vertical window resize by changing number of splits
                    Event::Window {
                        win_event: WindowEvent::Resized(..),
                        ..
                    } => {
                        // calculate the height taken by the splits and the total new height of the window
                        let height = self.canvas.viewport().height();
                        let rows_height = ((bottom_split_index - top_split_index) as u32
                            * (splits_height + 2))
                            + (splits_height * panels.len() as u32)
                            + splits_height;
                        // if there aren't any splits, we don't need to worry about changing the number of splits
                        if len != 0 {
                            // if there are too many splits, calculate how many and change indices
                            // otherwise if there are too few and there are enough to display more, change indices the other way
                            if height - timer_height < rows_height {
                                diff = ((rows_height - (height - timer_height)) / splits_height)
                                    as usize;
                                if max_splits > diff {
                                    max_splits -= diff;
                                } else {
                                    max_splits = 0;
                                }
                                if current_split > bottom_split_index - diff {
                                    top_split_index += diff;
                                    bottom_split_index = current_split;
                                } else if bottom_split_index > diff {
                                    bottom_split_index -= diff;
                                } else {
                                    bottom_split_index = 0;
                                }
                            } else if rows_height < height - timer_height {
                                diff = (((height - timer_height) - rows_height) / splits_height)
                                    as usize;
                                if current_split == bottom_split_index
                                    && current_split != len - 1
                                    && top_split_index >= diff
                                {
                                    top_split_index -= diff;
                                    max_splits += diff;
                                } else if bottom_split_index + diff > len - 1
                                    || max_splits + diff > len
                                {
                                    bottom_split_index = len - 1;
                                    max_splits = len;
                                } else {
                                    max_splits += diff;
                                    bottom_split_index = max_splits - 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            // rebuild comparisons if the comparison was swapped
            if comp_changed {
                comp_changed = false;
                index = 0;
                if let Comparison::None = self.comparison {
                    // set comp textures to just "-" if there is no comparison
                    while index < len {
                        text_surface = font
                            .render("-  ")
                            .blended(Color::WHITE)
                            .map_err(|_| get_error())?;
                        texture = creator
                            .create_texture_from_surface(&text_surface)
                            .map_err(|_| get_error())?;
                        splits[index].set_comp_tex(texture);
                        index += 1;
                    }
                } else if let Comparison::Average = self.comparison {
                    let (attempts, mut times) = {
                        let sums = self.run.sum_times();
                        let mut att = vec![];
                        let mut tm = vec![];
                        for sum in sums {
                            att.push(sum.0);
                            tm.push(sum.1);
                        }
                        (att, tm)
                    };
                    index = 0;
                    while index < attempts.len() {
                        times[index] = times[index] / {
                            if attempts[index] == 0 {
                                1
                            } else {
                                attempts[index]
                            }
                        };
                        index += 1;
                    }
                    let split_times_raw: Vec<String> = timing::split_time_sum(&times)
                        .iter()
                        .map(|val| timing::split_time_text(*val))
                        .collect();
                    index = 0;
                    while index < len {
                        text_surface = font
                            .render(&split_times_raw[index])
                            .blended(Color::WHITE)
                            .map_err(|_| get_error())?;
                        texture = creator
                            .create_texture_from_surface(&text_surface)
                            .map_err(|_| get_error())?;
                        splits[index].set_comp_tex(texture);
                        index += 1;
                    }
                } else {
                    let split_times = match self.comparison {
                        Comparison::PersonalBest => self.run.pb_times().to_vec(),
                        Comparison::Golds => self.run.gold_times().to_vec(),
                        _ => unreachable!(),
                    };
                    // rerender comparisons to either personal best or golds
                    let split_times_raw: Vec<String> = timing::split_time_sum(&split_times)
                        .iter()
                        .map(|val| timing::split_time_text(*val))
                        .collect();
                    index = 0;
                    while index < len {
                        text_surface = font
                            .render(&split_times_raw[index])
                            .blended(Color::WHITE)
                            .map_err(|_| get_error())?;
                        texture = creator
                            .create_texture_from_surface(&text_surface)
                            .map_err(|_| get_error())?;
                        splits[index].set_comp_tex(texture);
                        index += 1;
                    }
                }
            }
            // reset window width for placing text
            window_width = self.canvas.viewport().width();

            // make some changes to stuff before updating screen based on what happened in past loop
            // but only if the timer is running
            old_color = color;
            if let TimerState::Running { .. } = self.state {
                // calculates if run is ahead/behind/gaining/losing and adjusts accordingly
                elapsed = self.timer.elapsed().as_millis();
                // if we are in split 0 there's no need for fancy losing/gaining time, only ahead and behind
                if current_split == 0 && len != 0 {
                    if (elapsed - split_ticks) + before_pause_split
                        < splits[current_split].time()
                    {
                        // color = ahead;
                    } else {
                        // color = behind;
                    }
                } else if len != 0 {
                    if let Comparison::None = self.comparison {
                    } else {
                        // get the amount of time that the runner could spend on the split without being behind comparison
                        let allowed: i128;
                        allowed = (match self.comparison {
                            Comparison::PersonalBest => splits[current_split].time(),
                            Comparison::Golds => splits[current_split].gold(),
                            Comparison::Average => {
                                let sum = self.run.sum_times()[current_split];
                                sum.1 / {
                                    if sum.0 == 0 {
                                        1
                                    } else {
                                        sum.0
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }) as i128
                            - splits[current_split - 1].diff();
                        let buffer = splits[current_split - 1].diff();
                        // get amount of time that has passed in the current split
                        let time = ((elapsed - split_ticks) + before_pause_split) as i128;
                        // if the last split was ahead of comparison split
                        if buffer < 0 {
                            // if the runner has spent more time than allowed they have to be behind
                            if time > allowed {
                                color = behind;
                            // if they have spent less than the time it would take to become behind but more time than they took in the pb,
                            // then they are losing time but still ahead. default color for this is lightish green like LiveSplit
                            } else if time < allowed && time > allowed + buffer {
                                color = losing_time;
                            // if neither of those are true the runner must be ahead
                            } else {
                                color = ahead;
                            }
                        // if last split was behind comparison split
                        } else {
                            // if the runner has gone over the amount of time they should take but are still on better pace than
                            // last split then they are making up time. a sort of light red color like livesplit
                            if time > allowed && time < allowed + buffer {
                                color = making_up_time;
                            // if they are behind both the allowed time and their current pace they must be behind
                            } else if time > allowed && time > allowed + buffer {
                                color = behind;
                            // even if the last split was behind, often during part of the split the runner could finish it and come out ahead
                            } else {
                                color = ahead;
                            }
                        }
                    }
                }
                // set the split to highlight in blue when rendering
                // this value has to be adjusted to be relative to the number of splits on screen rather than
                // the total number of splits
                if current_split >= top_split_index && current_split <= bottom_split_index {
                    cur = current_split - top_split_index;
                } else {
                    // if the current split isnt on screen, pass this horrendously massive value to the render function
                    // so that it doesnt put a blue rectangle on anything (hopefully)
                    cur = usize::MAX;
                }
            // if timer isnt running then dont highlight a split or use a color
            } else {
                cur = usize::MAX;
                color = Color::WHITE;
            }
            // if the color has changed due to above calculations, recreate the font map in the new color
            if old_color != color {
                let map = timer_font
                    .render("- 0 1 2 3 4 5 6 7 8 9 : .")
                    .blended(color)
                    .map_err(|_| get_error())?;
                map_tex = creator
                    .create_texture_from_surface(&map)
                    .map_err(|_| get_error())?;
            }
            if panels.len() != 0 {
                for panel in &mut panels {
                    match panel.panel_type() {
                        Panel::SumOfBest if did_gold => {
                            did_gold = false;
                            let sob =
                                timing::split_time_text(self.run.gold_times().iter().sum::<u128>());
                            text_surface = font
                                .render(&sob)
                                .blended(Color::WHITE)
                                .map_err(|_| get_error())?;
                            texture = creator
                                .create_texture_from_surface(text_surface)
                                .map_err(|_| get_error())?;
                            panel.set_time(texture);
                        }
                        Panel::Pace { golds }
                            if matches!(self.state, TimerState::Running { .. }) =>
                        {
                            self.run.pb_times();
                            
                        }
                        Panel::CurrentSplitDiff { golds }
                            if matches!(self.state, TimerState::Running { .. })
                                && splits.len() > 1 =>
                        {
                            let time = {
                                let tm = (self.timer.elapsed().as_millis() - split_ticks) + before_pause_split;
                                if tm < self.run.gold_times()[current_split] {
                                    timing::diff_text(-1 * (self.run.gold_times()[current_split] - tm) as i128)
                                } else {
                                    timing::diff_text((tm - self.run.gold_times()[current_split]) as i128)
                                }
                            };
                            text_surface = font
                                .render(&time)
                                .blended(Color::BLUE)
                                .map_err(|_| get_error())?;
                            texture = creator
                                .create_texture_from_surface(text_surface)
                                .map_err(|_| get_error())?;
                            panel.set_time(texture);
                        }
                        _ => {}
                    }
                }
            }
            // copy the name, diff, and time textures to the canvas
            // and highlight the split relative to the top of the list marked by cur
            // function places the rows and ensures that they don't go offscreen
            
            // update the time based on the current timer state
            time_str = self.update_time(before_pause, start_ticks);
            // copy the time texture to the canvas, place individual characters from map
            render::render_time(
                time_str.clone(),
                &map_tex_outline,
                &coords,
                (font_y, splits_height, panels.len() as usize),
                &mut self.canvas, 
                0,
            )?;
            render::render_time(
                time_str.clone(),
                &map_tex,
                &coords,
                (font_y, splits_height, panels.len() as usize),
                &mut self.canvas,
                2,
            )?;
            
            self.canvas.present();


            //original render thing ends here ^

            let ms: i32 = rainbow_int;
            let rnum = 2500;
            let r = rainbow(rnum);
            let mut n = ms;//(ms as f32 * 2.5).floor() as i32 % rnum;
            let red = r[n as usize][0];
            let green = r[n as usize][1];
            let blue = r[n as usize][2];
            timer_font.set_outline_width(1);
            let mut c = Color::WHITE;
            // println!("{:?}", current_split);
            match &self.state {
                TimerState::Running { .. } => {
                    c = Color::RGB(66, 135, 245);
                }
                TimerState::NotRunning { .. } => {
                    if current_split != 0 {
                        c = Color::RGB(red, green, blue);
                    }
                    else {
                        c = Color::RGB(122, 122, 120);
                    }
                }
                TimerState::Paused { .. } => {
                    c = Color::RGB(red, green, blue);
                }
                TimerState::OffsetCountdown { .. } => {
                    c = Color::RGB(45, 23, 143);
                }
            }
            map = timer_font
                .render("- 0 1 2 3 4 5 6 7 8 9 : .")
            
                .blended(c)
                .map_err(|_| get_error())?;
            map_tex = creator
                .create_texture_from_surface(&map)
                .map_err(|_| get_error())?;
                time_str_outline = time_str.clone();
            render::render_time( time_str.clone(), &map_tex, &coords, (font_y, splits_height, panels.len() as usize), &mut self.canvas, 2,)?;
        // set the height where overlap with splits is checked when resizing window
        let mut timer_height = font_y + splits_height;

            // self.canvas.present();
            if Instant::now().duration_since(frame_time) <= one_sixtieth {
                thread::sleep(
                    // if the entire loop pass was completed in under 1/60 second, delay to keep the framerate at ~60fps
                    one_sixtieth - Instant::now().duration_since(frame_time),
                );
            }
        }
        // after the loop is exited then save the config file
        self.config.save()?;
        // if splits were updated, prompt user to save the split file
        if save && dialogs::save_check() {
            if path == "" {
                let p = dialogs::get_save_as();
                match p {
                    Some(s) => {
                        path = s;
                        let mut f = File::create(&path).map_err(|e| e.to_string())?;
                        self.msf.write(&self.run, &mut f)?;
                    }
                    None => {}
                }
            } else {
                let mut f = File::open(&path).map_err(|e| e.to_string())?;
                self.msf.write(&self.run, &mut f)?;
            }
        }
        Ok(())
    }
    // updates time string based on timer state, basically leaves it the same if timer is not running
    fn update_time(&self, before_pause: u128, start_ticks: u128) -> String {
        let time: String;
        match &self.state {
            TimerState::Running { .. } => {
                time =
                    timing::ms_to_readable((self.timer.elapsed().as_millis() - start_ticks) + before_pause, false);
            }
            TimerState::NotRunning { time_str: string }
            | TimerState::Paused {
                time_str: string, ..
            } => {
                time = string.to_owned();
            }
            TimerState::OffsetCountdown { amt: amount } => {
                if amount > &(self.timer.elapsed().as_millis() - start_ticks) {
                    let num =
                        timing::ms_to_readable(amount - (self.timer.elapsed().as_millis() - start_ticks), false);
                    time = format!("-{}", num);
                } else {
                    time = "0.000".to_owned();
                }
            }
        }
        return time;
    }
}
