//! crates/chronos-ui/src/draw.rs
//!
//! Software rasterizer, geometry drawers, and vector analog clock rendering.

use crate::geom::{Color, Rect};

pub trait Surface {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
    fn get_pixel(&self, x: i32, y: i32) -> Color;

    fn fill_rect(&mut self, r: Rect, color: Color) {
        let x0 = r.x.max(0);
        let y0 = r.y.max(0);
        let x1 = (r.x + r.w).min(self.width());
        let y1 = (r.y + r.h).min(self.height());
        for y in y0..y1 {
            for x in x0..x1 {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn draw_rect(&mut self, r: Rect, color: Color) {
        if r.w <= 0 || r.h <= 0 {
            return;
        }
        self.draw_line(r.x, r.y, r.x + r.w - 1, r.y, color);
        self.draw_line(r.x, r.y + r.h - 1, r.x + r.w - 1, r.y + r.h - 1, color);
        self.draw_line(r.x, r.y, r.x, r.y + r.h - 1, color);
        self.draw_line(r.x + r.w - 1, r.y, r.x + r.w - 1, r.y + r.h - 1, color);
    }

    fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.put_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let mut x = r;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.put_pixel(cx + x, cy + y, color);
            self.put_pixel(cx + y, cy + x, color);
            self.put_pixel(cx - y, cy + x, color);
            self.put_pixel(cx - x, cy + y, color);
            self.put_pixel(cx - x, cy - y, color);
            self.put_pixel(cx - y, cy - x, color);
            self.put_pixel(cx + y, cy - x, color);
            self.put_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        let r2 = r * r;
        for dy in -r..=r {
            let y = cy + dy;
            if y < 0 || y >= self.height() {
                continue;
            }
            let dx_max = ((r2 - dy * dy) as f32).sqrt() as i32;
            let x0 = (cx - dx_max).max(0);
            let x1 = (cx + dx_max).min(self.width() - 1);
            for x in x0..=x1 {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn fill_rounded_rect(&mut self, r: Rect, rad: i32, color: Color) {
        let rad = rad.min(r.w / 2).min(r.h / 2).max(0);
        if rad == 0 {
            self.fill_rect(r, color);
            return;
        }

        // Central cross
        self.fill_rect(Rect::new(r.x + rad, r.y, r.w - 2 * rad, r.h), color);
        self.fill_rect(Rect::new(r.x, r.y + rad, rad, r.h - 2 * rad), color);
        self.fill_rect(Rect::new(r.x + r.w - rad, r.y + rad, rad, r.h - 2 * rad), color);

        // Four corner circles
        self.fill_circle(r.x + rad, r.y + rad, rad, color);
        self.fill_circle(r.x + r.w - rad - 1, r.y + rad, rad, color);
        self.fill_circle(r.x + rad, r.y + r.h - rad - 1, rad, color);
        self.fill_circle(r.x + r.w - rad - 1, r.y + r.h - rad - 1, rad, color);
    }

    fn draw_rounded_rect(&mut self, r: Rect, rad: i32, color: Color) {
        let rad = rad.min(r.w / 2).min(r.h / 2).max(0);
        if rad == 0 {
            self.draw_rect(r, color);
            return;
        }
        // Top and bottom edges
        self.draw_line(r.x + rad, r.y, r.x + r.w - rad - 1, r.y, color);
        self.draw_line(r.x + rad, r.y + r.h - 1, r.x + r.w - rad - 1, r.y + r.h - 1, color);
        // Left and right edges
        self.draw_line(r.x, r.y + rad, r.x, r.y + r.h - rad - 1, color);
        self.draw_line(r.x + r.w - 1, r.y + rad, r.x + r.w - 1, r.y + r.h - rad - 1, color);

        // Approximate corners
        let mut x = rad;
        let mut y = 0;
        let mut err = 0;
        while x >= y {
            // Top-left
            self.put_pixel(r.x + rad - x, r.y + rad - y, color);
            self.put_pixel(r.x + rad - y, r.y + rad - x, color);
            // Top-right
            self.put_pixel(r.x + r.w - 1 - rad + x, r.y + rad - y, color);
            self.put_pixel(r.x + r.w - 1 - rad + y, r.y + rad - x, color);
            // Bottom-left
            self.put_pixel(r.x + rad - x, r.y + r.h - 1 - rad + y, color);
            self.put_pixel(r.x + rad - y, r.y + r.h - 1 - rad + x, color);
            // Bottom-right
            self.put_pixel(r.x + r.w - 1 - rad + x, r.y + r.h - 1 - rad + y, color);
            self.put_pixel(r.x + r.w - 1 - rad + y, r.y + r.h - 1 - rad + x, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Renders an authentic analog clock face with hour marks, minute marks,
    /// hour hand, minute hand, orange second hand, and center cap.
    fn draw_analog_clock(
        &mut self,
        cx: i32,
        cy: i32,
        radius: i32,
        hours: u8,
        mins: u8,
        secs: u8,
        is_daytime: bool,
    ) {
        let dial_bg = if is_daytime { Color::CLOCK_FACE } else { Color::CLOCK_FACE_DARK };
        let dial_rim = if is_daytime { Color::rgb(210, 215, 225) } else { Color::rgb(60, 65, 75) };
        let tick_major = if is_daytime { Color::rgb(50, 55, 65) } else { Color::rgb(220, 225, 235) };
        let tick_minor = if is_daytime { Color::rgb(170, 175, 185) } else { Color::rgb(100, 105, 115) };
        let hand_color = if is_daytime { Color::rgb(30, 35, 45) } else { Color::rgb(240, 243, 250) };
        let sec_color = Color::CLOCK_HAND_SEC;

        // Dial face background
        self.fill_circle(cx, cy, radius, dial_bg);
        self.draw_circle(cx, cy, radius, dial_rim);
        if radius > 15 {
            self.draw_circle(cx, cy, radius - 1, dial_rim);
        }

        // Draw 12 hour ticks and 60 minute ticks
        for i in 0..60 {
            let angle_rad = (i as f32 * 6.0 - 90.0) * core::f32::consts::PI / 180.0;
            let cos_a = angle_rad.cos();
            let sin_a = angle_rad.sin();
            let is_hour = i % 5 == 0;
            let tick_len = if is_hour { (radius / 6).max(3) } else { (radius / 12).max(1) };
            let r_outer = radius - 3;
            let r_inner = r_outer - tick_len;

            let x0 = cx + (r_inner as f32 * cos_a) as i32;
            let y0 = cy + (r_inner as f32 * sin_a) as i32;
            let x1 = cx + (r_outer as f32 * cos_a) as i32;
            let y1 = cy + (r_outer as f32 * sin_a) as i32;

            let col = if is_hour { tick_major } else { tick_minor };
            self.draw_line(x0, y0, x1, y1, col);
        }

        // Hour Hand: 360 deg / 12 hrs = 30 deg/hr + 0.5 deg/min
        let hr_angle_rad = ((hours % 12) as f32 * 30.0 + mins as f32 * 0.5 - 90.0) * core::f32::consts::PI / 180.0;
        let hr_len = (radius as f32 * 0.55) as i32;
        let hx = cx + (hr_len as f32 * hr_angle_rad.cos()) as i32;
        let hy = cy + (hr_len as f32 * hr_angle_rad.sin()) as i32;
        self.draw_line(cx, cy, hx, hy, hand_color);
        // Double thickness for hour hand
        self.draw_line(cx + 1, cy, hx + 1, hy, hand_color);
        self.draw_line(cx, cy + 1, hx, hy + 1, hand_color);

        // Minute Hand: 360 deg / 60 mins = 6 deg/min + 0.1 deg/sec
        let min_angle_rad = (mins as f32 * 6.0 + secs as f32 * 0.1 - 90.0) * core::f32::consts::PI / 180.0;
        let min_len = (radius as f32 * 0.82) as i32;
        let mx = cx + (min_len as f32 * min_angle_rad.cos()) as i32;
        let my = cy + (min_len as f32 * min_angle_rad.sin()) as i32;
        self.draw_line(cx, cy, mx, my, hand_color);
        self.draw_line(cx + 1, cy, mx + 1, my, hand_color);

        // Second Hand: vibrant orange with counterweight
        let sec_angle_rad = (secs as f32 * 6.0 - 90.0) * core::f32::consts::PI / 180.0;
        let sec_len = (radius as f32 * 0.88) as i32;
        let sec_tail = (radius as f32 * 0.20) as i32;
        let sx = cx + (sec_len as f32 * sec_angle_rad.cos()) as i32;
        let sy = cy + (sec_len as f32 * sec_angle_rad.sin()) as i32;
        let tx = cx - (sec_tail as f32 * sec_angle_rad.cos()) as i32;
        let ty = cy - (sec_tail as f32 * sec_angle_rad.sin()) as i32;
        self.draw_line(tx, ty, sx, sy, sec_color);

        // Center Pivot Pin
        self.fill_circle(cx, cy, 3, sec_color);
        self.fill_circle(cx, cy, 1, Color::rgb(255, 255, 255));
    }
}

pub struct BufferSurface32<'a> {
    pub pixels: &'a mut [u32],
    pub width: i32,
    pub height: i32,
    pub pitch: i32,
}

impl<'a> BufferSurface32<'a> {
    pub fn new(pixels: &'a mut [u32], width: i32, height: i32, pitch: i32) -> Self {
        BufferSurface32 {
            pixels,
            width,
            height,
            pitch,
        }
    }
}

impl<'a> Surface for BufferSurface32<'a> {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = (y * self.pitch + x) as usize;
        if idx < self.pixels.len() {
            if color.a == 255 {
                self.pixels[idx] = color.to_u32();
            } else if color.a > 0 {
                let bg = Color::from_u32(self.pixels[idx]);
                self.pixels[idx] = color.blend_over(bg).to_u32();
            }
        }
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Color::rgba(0, 0, 0, 0);
        }
        let idx = (y * self.pitch + x) as usize;
        if idx < self.pixels.len() {
            Color::from_u32(self.pixels[idx])
        } else {
            Color::rgba(0, 0, 0, 0)
        }
    }
}

pub struct OwnedBufferSurface {
    pub pixels: Vec<u32>,
    pub width: i32,
    pub height: i32,
}

impl OwnedBufferSurface {
    pub fn new(w: i32, h: i32) -> Self {
        OwnedBufferSurface {
            pixels: vec![Color::DARK_BG.to_u32(); (w * h) as usize],
            width: w,
            height: h,
        }
    }
}

impl Surface for OwnedBufferSurface {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            if color.a == 255 {
                self.pixels[idx] = color.to_u32();
            } else if color.a > 0 {
                let bg = Color::from_u32(self.pixels[idx]);
                self.pixels[idx] = color.blend_over(bg).to_u32();
            }
        }
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Color::rgba(0, 0, 0, 0);
        }
        let idx = (y * self.width + x) as usize;
        if idx < self.pixels.len() {
            Color::from_u32(self.pixels[idx])
        } else {
            Color::rgba(0, 0, 0, 0)
        }
    }
}
