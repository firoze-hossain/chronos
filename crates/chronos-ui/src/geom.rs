//! crates/chronos-ui/src/geom.rs
//!
//! Geometric primitives and Fluent/macOS color palette for Chronos.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }

    pub fn contains_xy(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub const fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn from_u32(val: u32) -> Self {
        Color {
            a: ((val >> 24) & 0xFF) as u8,
            r: ((val >> 16) & 0xFF) as u8,
            g: ((val >> 8) & 0xFF) as u8,
            b: (val & 0xFF) as u8,
        }
    }

    pub fn blend_over(self, bg: Color) -> Color {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return bg;
        }
        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;
        let r = ((self.r as u32 * alpha + bg.r as u32 * inv_alpha) / 255) as u8;
        let g = ((self.g as u32 * alpha + bg.g as u32 * inv_alpha) / 255) as u8;
        let b = ((self.b as u32 * alpha + bg.b as u32 * inv_alpha) / 255) as u8;
        Color { r, g, b, a: 255 }
    }

    // Signature UI Theme Colors
    pub const DARK_BG: Color = Color::rgb(22, 24, 29);
    pub const PANEL_BG: Color = Color::rgb(30, 33, 40);
    pub const CARD_BG: Color = Color::rgb(38, 42, 51);
    pub const CARD_BORDER: Color = Color::rgb(55, 60, 72);
    pub const ACCENT_ORANGE: Color = Color::rgb(255, 107, 0); // Ubuntu Yaru & macOS Orange
    pub const ACCENT_BLUE: Color = Color::rgb(56, 189, 248);  // NovaOS Fluent Blue
    pub const ACCENT_GREEN: Color = Color::rgb(46, 204, 113); // Start Button / Fast Lap
    pub const ACCENT_RED: Color = Color::rgb(235, 87, 87);    // Stop Button / Slow Lap
    pub const TEXT_PRIMARY: Color = Color::rgb(243, 244, 246);
    pub const TEXT_SECONDARY: Color = Color::rgb(156, 163, 175);
    pub const TEXT_MUTED: Color = Color::rgb(107, 114, 128);
    pub const CLOCK_FACE: Color = Color::rgb(245, 247, 250);
    pub const CLOCK_FACE_DARK: Color = Color::rgb(28, 30, 36);
    pub const CLOCK_HAND: Color = Color::rgb(20, 20, 24);
    pub const CLOCK_HAND_SEC: Color = Color::rgb(255, 107, 0);

    // Semantic UI aliases
    pub const SURFACE_BG: Color = Self::DARK_BG;
    pub const SURFACE_HEADER: Color = Self::PANEL_BG;
    pub const SURFACE_CARD: Color = Self::CARD_BG;
    pub const BORDER_DARK: Color = Self::CARD_BORDER;
    pub const STATUS_SUCCESS: Color = Self::ACCENT_GREEN;
    pub const STATUS_ERROR: Color = Self::ACCENT_RED;
}
