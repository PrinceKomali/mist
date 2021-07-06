// Functions for putting stuff into the correct places on the sdl buffer
use crate::panels::RenderPanel;
use crate::splits::Split;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::video::Window;

pub fn render_time(
    time_str: String,
    atlas: &Texture,
    coords: &[u32],
    (font_y, splits_height, num_panels): (u32, u32, usize),
    canvas: &mut Canvas<Window>,
    offset: i32
) -> Result<(), String> {
    let mut x = 0;
    let vp = canvas.viewport();
    let h = vp.height();
    let w = vp.width();
    let mut src = Rect::new(0, 0, 0, font_y);
    // multiply initial values by 10/10 so that the font is not smaller
    let mut dst = Rect::new(
        0,
        (h - (font_y * 1) - (splits_height * num_panels as u32)) as i32 ,
        0,
        font_y * 1,
    );
    let mut idx: usize;
    let mut char_num = 0;
    let space = coords[14] - coords[13];
    for chr in time_str.chars().rev() {
        // get the index in the coordinate slice based on the character to render
        idx = match chr {
            '-' => 0,
            '0' => 1,
            '1' => 2,
            '2' => 3,
            '3' => 4,
            '4' => 5,
            '5' => 6,
            '6' => 7,
            '7' => 8,
            '8' => 9,
            '9' => 10,
            ':' => 11,
            '.' => 12,
            _ => 0,
        };
        let width = coords[idx + 1] - coords[idx] + 2;
        // only monospace numbers so that the typically smaller punctuation looks better
        if chr == '.' || chr == ':' {
            x += width;
        } else {
            if char_num < 4 {
                x += coords[15] * 10 / 10;
            } else {
                x += coords[15];
            }
        }
        src.set_x(if idx != 0 {
            (coords[idx] - 2) as i32 + (idx as u32 * space) as i32
        } else {
            (coords[idx]) as i32 + (idx as u32 * space) as i32
        });
        src.set_width(width);
        dst.set_x((w - x - 10 - offset as u32) as i32);
        if char_num < 4 {
            dst.set_width(width * 1);
        } else {
            dst.set_width(width);
            dst.set_y((h - font_y - (splits_height * num_panels as u32)) as i32);
            dst.set_height(font_y);
        }
        canvas.copy(atlas, Some(src), Some(dst))?;
        char_num += 1;
    }
    Ok(())
}
