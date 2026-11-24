//! crates/chronos-ui/src/map.rs
//!
//! Equirectangular world map rasterizer with real-time solar terminator shading
//! and interactive glowing city pins.

use crate::draw::Surface;
use crate::font::draw_text_small;
use crate::geom::{Color, Rect};
use chronos_core::world::{City, WorldClockManager};

/// Landmass detection using a 64x32 low-resolution world grid (1 = land, 0 = ocean).
/// 64 columns (-180° to +180°), 32 rows (+85° to -75°).
/// Compact representation: 32 rows of u64 (64 bits each).
static WORLD_LAND_MASK: [u64; 32] = [
    // 85°N to 70°N: Arctic, Greenland, North Siberia
    0b0000000000000000001111000000000000000000000000000011111111000000,
    0b0000000000000000011111100000000000000000000000000111111111100000,
    0b0000000000000000111111110000000000000000000000001111111111110000,
    // 70°N to 55°N: Alaska, Canada, Scandinavia, Russia
    0b0111111111100001111111110000000111000000000000011111111111111100,
    0b0111111111111111111111110000001111100000000000111111111111111100,
    0b0011111111111111111111110000011111110000000001111111111111111100,
    // 55°N to 40°N: North America, UK, Europe, Central Asia, East Asia
    0b0001111111111111111111100000001111111100000011111111111111111110,
    0b0000111111111111111111000000000111111111111111111111111111111110,
    0b0000111111111111111110000000000111111111111111111111111111111110,
    // 40°N to 25°N: USA, Mediterranean, Middle East, India, China, Japan
    0b0000011111111111111110000000000111111111111111111111111111111110,
    0b0000001111111111111100000000001111111111111111111111111111111000,
    0b0000000111111111111000000000011111111111111111111111111111110000,
    // 25°N to 10°N: Central America, Caribbean, Sahara, Arabian Peninsula, South Asia, SE Asia
    0b0000000011111111100000000000011111111111111111111111111111100000,
    0b0000000001111111000000000000111111111111111111111111111110000000,
    0b0000000000111110000000000000111111111111111111101111111100000000,
    // 10°N to 5°S: Equator, Northern South America, Central Africa, Indonesia
    0b0000000000011110000000000000111111111111111000000111111000000000,
    0b0000000000001111111000000000011111111111110000000001111000000000,
    0b0000000000001111111110000000001111111111100000000000111100000000,
    // 5°S to 20°S: South America, Southern Africa, Australia north
    0b0000000000000111111111000000000111111111100000000000011100000000,
    0b0000000000000011111111000000000011111111100000000000000111100000,
    0b0000000000000011111111000000000001111111000000000000111111110000,
    // 20°S to 35°S: Argentina/Chile, South Africa, Australia mainland
    0b0000000000000001111111000000000000111110000000000000111111111000,
    0b0000000000000001111110000000000000011100000000000000111111111000,
    0b0000000000000000111110000000000000000000000000000000011111110000,
    // 35°S to 50°S: Patagonia, New Zealand
    0b0000000000000000011100000000000000000000000000000000000001110000,
    0b0000000000000000001100000000000000000000000000000000000000110000,
    0b0000000000000000000100000000000000000000000000000000000000000000,
    // 50°S to 65°S: Southern Ocean
    0b0000000000000000000000000000000000000000000000000000000000000000,
    0b0000000000000000000000000000000000000000000000000000000000000000,
    // 65°S to 85°S: Antarctica
    0b0000000000000001111111111111111111111111111111111111110000000000,
    0b0000000000001111111111111111111111111111111111111111111100000000,
    0b1111111111111111111111111111111111111111111111111111111111111111,
];

pub fn is_land(lat: f32, lon: f32) -> bool {
    // Map lat (-90 to +90) into row (0 to 31)
    let norm_lat = (85.0 - lat) / 165.0; // 0.0 at 85N, 1.0 at 80S
    let row = (norm_lat * 31.99).clamp(0.0, 31.0) as usize;

    // Map lon (-180 to +180) into col (0 to 63)
    let norm_lon = (lon + 180.0) / 360.0;
    let col = (norm_lon * 63.99).clamp(0.0, 63.0) as usize;

    let row_bits = WORLD_LAND_MASK[row];
    let bit = (row_bits >> (63 - col)) & 1;
    bit == 1
}

pub struct WorldMapWidget {
    pub bounds: Rect,
}

impl WorldMapWidget {
    pub fn new(bounds: Rect) -> Self {
        WorldMapWidget { bounds }
    }

    /// Renders the complete solar day/night equirectangular world map with city pins.
    pub fn render<S: Surface + ?Sized>(&self, s: &mut S, wm: &WorldClockManager) {
        if self.bounds.w <= 0 || self.bounds.h <= 0 {
            return;
        }

        // Draw outer card border / backdrop
        s.fill_rounded_rect(self.bounds, 8, Color::rgb(18, 22, 32));
        s.draw_rounded_rect(self.bounds, 8, Color::rgb(42, 48, 66));

        let map_rect = Rect::new(
            self.bounds.x + 8,
            self.bounds.y + 8,
            self.bounds.w - 16,
            self.bounds.h - 16,
        );

        if map_rect.w <= 10 || map_rect.h <= 10 {
            return;
        }

        // Step size for rasterization (adaptive resolution for high frame-rates)
        let step = if map_rect.w > 400 { 2 } else { 3 };

        let ocean_day = Color::rgb(24, 40, 64);
        let ocean_night = Color::rgb(10, 16, 26);
        let land_day = Color::rgb(48, 76, 62);
        let land_night = Color::rgb(22, 32, 38);
        let _twilight_glow = Color::rgb(70, 62, 48);

        let mut y = map_rect.y;
        while y < map_rect.y + map_rect.h {
            let lat = 85.0 - ((y - map_rect.y) as f32 / map_rect.h as f32) * 170.0;
            let mut x = map_rect.x;
            while x < map_rect.x + map_rect.w {
                let lon = -180.0 + ((x - map_rect.x) as f32 / map_rect.w as f32) * 360.0;

                let in_day = wm.is_coord_in_daylight(lat, lon);
                let land = is_land(lat, lon);

                let color = match (land, in_day) {
                    (true, true) => land_day,
                    (true, false) => land_night,
                    (false, true) => ocean_day,
                    (false, false) => ocean_night,
                };

                // Fill block of (step x step)
                let w = step.min(map_rect.x + map_rect.w - x);
                let h = step.min(map_rect.y + map_rect.h - y);
                s.fill_rect(Rect::new(x, y, w, h), color);

                x += step;
            }
            y += step;
        }

        // Draw Subtle Map Border & Equator / Prime Meridian guides
        s.draw_rect(map_rect, Color::rgb(35, 45, 65));
        let eq_y = map_rect.y + map_rect.h / 2;
        s.draw_line(map_rect.x, eq_y, map_rect.x + map_rect.w - 1, eq_y, Color::rgba(70, 85, 115, 60));

        let pm_x = map_rect.x + map_rect.w / 2;
        s.draw_line(pm_x, map_rect.y, pm_x, map_rect.y + map_rect.h - 1, Color::rgba(70, 85, 115, 60));

        // Draw City Pins for all pinned cities
        for (idx, pinned) in wm.pinned.iter().enumerate() {
            let is_selected = idx == wm.selected_idx;
            self.draw_city_pin(s, map_rect, &pinned.city, wm, is_selected);
        }

        // Draw Map Legend & Solar Status header badge
        let header_rect = Rect::new(map_rect.x + 8, map_rect.y + 8, 190, 20);
        s.fill_rounded_rect(header_rect, 4, Color::rgba(15, 20, 30, 210));
        s.draw_rounded_rect(header_rect, 4, Color::rgba(80, 95, 125, 120));

        let utc_str = format!("UTC {:02}:{:02}:{:02} • DAY/NIGHT", wm.base_utc.hour, wm.base_utc.minute, wm.base_utc.second);
        draw_text_small(s, header_rect.x + 6, header_rect.y + 5, &utc_str, Color::rgb(200, 215, 240));
    }

    fn draw_city_pin<S: Surface + ?Sized>(
        &self,
        s: &mut S,
        map_rect: Rect,
        city: &City,
        wm: &WorldClockManager,
        is_selected: bool,
    ) {
        // Project Lat/Lon to pixel coordinates
        let norm_x = (city.lon + 180.0) / 360.0;
        let norm_y = (85.0 - city.lat) / 170.0;

        let px = map_rect.x + (norm_x * map_rect.w as f32) as i32;
        let py = map_rect.y + (norm_y * map_rect.h as f32) as i32;

        if px < map_rect.x || px >= map_rect.x + map_rect.w || py < map_rect.y || py >= map_rect.y + map_rect.h {
            return;
        }

        let is_day = wm.is_daytime(city);
        let pin_color = if is_selected {
            Color::CLOCK_HAND_SEC // Orange highlight
        } else if is_day {
            Color::rgb(255, 215, 60) // Sunny gold
        } else {
            Color::rgb(100, 185, 255) // Moonlit blue
        };

        // Pin Glow and Center Point
        if is_selected {
            s.draw_circle(px, py, 7, Color::rgba(255, 105, 0, 100));
            s.draw_circle(px, py, 5, Color::rgba(255, 140, 0, 180));
        } else {
            s.draw_circle(px, py, 4, Color::rgba(pin_color.r, pin_color.g, pin_color.b, 90));
        }
        s.fill_circle(px, py, 3, pin_color);
        s.fill_circle(px, py, 1, Color::rgb(255, 255, 255));

        // Callout Tag for selected city or primary pin
        if is_selected || city.id == "dac" {
            let city_time = wm.city_time(city);
            let time_str = format!("{} {:02}:{:02}", city.name, city_time.hour, city_time.minute);
            let tag_w = (time_str.len() as i32) * 6 + 10;
            let tag_h = 16;
            let mut tag_x = px + 6;
            let mut tag_y = py - 8;

            // Bounds protection
            if tag_x + tag_w > map_rect.x + map_rect.w - 4 {
                tag_x = px - tag_w - 6;
            }
            if tag_y < map_rect.y + 4 {
                tag_y = py + 6;
            }

            let tag_rect = Rect::new(tag_x, tag_y, tag_w, tag_h);
            s.fill_rounded_rect(tag_rect, 3, Color::rgba(20, 25, 35, 220));
            s.draw_rounded_rect(tag_rect, 3, if is_selected { Color::ACCENT_ORANGE } else { Color::BORDER_DARK });
            draw_text_small(s, tag_x + 5, tag_y + 3, &time_str, Color::TEXT_PRIMARY);
        }
    }
}
