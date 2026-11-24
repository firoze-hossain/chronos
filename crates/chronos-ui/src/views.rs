//! crates/chronos-ui/src/views.rs
//!
//! Visual layout and interactive components for all 4 Chronos tabs:
//! World Clock, Alarms, Stopwatch, and Timers, plus Modal Dialogs.

use crate::draw::Surface;
use crate::font::{draw_large_digital, draw_text, draw_text_font, draw_text_small, Font};
use crate::geom::{Color, Rect};
use crate::map::WorldMapWidget;
use chronos_core::stopwatch::StopwatchState;
use chronos_core::timer::{TimerState, QUICK_PRESETS};
use chronos_core::world::ALL_CITIES;
use chronos_core::{ClockHub, ClockTab};

pub const HEADER_HEIGHT: i32 = 44;

pub struct ChronosView;

impl ChronosView {
    /// Main entry point to render the full Chronos window / surface.
    pub fn render<S: Surface + ?Sized>(s: &mut S, bounds: Rect, hub: &ClockHub) {
        // Base window background
        s.fill_rect(bounds, Color::SURFACE_BG);

        // Header Tab Bar
        let header_rect = Rect::new(bounds.x, bounds.y, bounds.w, HEADER_HEIGHT);
        Self::render_header(s, header_rect, hub);

        // Content Area
        let content_rect = Rect::new(
            bounds.x,
            bounds.y + HEADER_HEIGHT,
            bounds.w,
            bounds.h - HEADER_HEIGHT,
        );

        match hub.active_tab {
            ClockTab::WorldClock => Self::render_world_clock(s, content_rect, hub),
            ClockTab::Alarms => Self::render_alarms(s, content_rect, hub),
            ClockTab::Stopwatch => Self::render_stopwatch(s, content_rect, hub),
            ClockTab::Timers => Self::render_timers(s, content_rect, hub),
        }

        // Modals & Overlay Alerts
        if hub.show_add_city_modal {
            Self::render_add_city_modal(s, bounds, hub);
        } else if hub.show_add_alarm_modal {
            Self::render_add_alarm_modal(s, bounds, hub);
        }

        // Active Ringing Alert Banners
        if hub.alarm_alert_active {
            Self::render_alarm_alert_overlay(s, bounds, hub);
        } else if hub.timer_alert_active {
            Self::render_timer_alert_overlay(s, bounds, hub);
        }
    }

    /// Renders the top navigation segmented tab bar with "+" button.
    fn render_header<S: Surface + ?Sized>(s: &mut S, r: Rect, hub: &ClockHub) {
        s.fill_rect(r, Color::SURFACE_HEADER);
        s.draw_line(r.x, r.y + r.h - 1, r.x + r.w - 1, r.y + r.h - 1, Color::BORDER_DARK);

        // App Logo / Title
        draw_text(s, r.x + 14, r.y + 15, "Chronos", Color::TEXT_PRIMARY);

        // Segmented Tabs Pill Container
        let tabs = [
            (ClockTab::WorldClock, "World Clock"),
            (ClockTab::Alarms, "Alarms"),
            (ClockTab::Stopwatch, "Stopwatch"),
            (ClockTab::Timers, "Timers"),
        ];

        let tab_w = 95;
        let tab_h = 28;
        let total_tabs_w = tab_w * (tabs.len() as i32);
        let start_x = r.x + (r.w - total_tabs_w) / 2;
        let tab_y = r.y + (r.h - tab_h) / 2;

        let container_rect = Rect::new(start_x - 3, tab_y - 2, total_tabs_w + 6, tab_h + 4);
        s.fill_rounded_rect(container_rect, 6, Color::rgb(15, 18, 25));
        s.draw_rounded_rect(container_rect, 6, Color::BORDER_DARK);

        for (i, (tab_kind, label)) in tabs.iter().enumerate() {
            let tx = start_x + (i as i32) * tab_w;
            let trect = Rect::new(tx, tab_y, tab_w, tab_h);

            let is_active = hub.active_tab == *tab_kind;
            if is_active {
                s.fill_rounded_rect(trect, 5, Color::ACCENT_BLUE);
                draw_text(s, tx + 10, tab_y + 8, label, Color::TEXT_PRIMARY);
            } else {
                draw_text(s, tx + 10, tab_y + 8, label, Color::TEXT_SECONDARY);
            }
        }

        // "+" Action Button (Active on WorldClock and Alarms tabs)
        let can_add = hub.active_tab == ClockTab::WorldClock || hub.active_tab == ClockTab::Alarms;
        if can_add {
            let plus_w = 28;
            let plus_h = 28;
            let plus_x = r.x + r.w - plus_w - 14;
            let plus_y = r.y + (r.h - plus_h) / 2;
            let plus_rect = Rect::new(plus_x, plus_y, plus_w, plus_h);

            s.fill_rounded_rect(plus_rect, 6, Color::ACCENT_BLUE);
            // Draw crisp '+' symbol
            s.draw_line(plus_x + 8, plus_y + 14, plus_x + 20, plus_y + 14, Color::TEXT_PRIMARY);
            s.draw_line(plus_x + 14, plus_y + 8, plus_x + 14, plus_y + 20, Color::TEXT_PRIMARY);
        }
    }

    /// Tab 1: World Clock view with Solar Terminator Map and City Cards.
    fn render_world_clock<S: Surface + ?Sized>(s: &mut S, r: Rect, hub: &ClockHub) {
        let padding = 12;
        let map_h = ((r.h as f32) * 0.46) as i32;
        let map_rect = Rect::new(r.x + padding, r.y + padding, r.w - 2 * padding, map_h);

        // 1. Render World Map Widget
        let map_widget = WorldMapWidget::new(map_rect);
        map_widget.render(s, &hub.world_mgr);

        // 2. Render City Cards in Grid below Map
        let cards_y = map_rect.y + map_rect.h + padding;
        let cards_h = r.y + r.h - cards_y - padding;
        let cards_w = r.w - 2 * padding;

        let num_cities = hub.world_mgr.pinned.len();
        if num_cities == 0 {
            let empty_rect = Rect::new(r.x + padding, cards_y, cards_w, cards_h);
            s.fill_rounded_rect(empty_rect, 8, Color::SURFACE_CARD);
            draw_text(s, empty_rect.x + 20, empty_rect.y + 30, "No world clocks pinned.", Color::TEXT_SECONDARY);
            draw_text(s, empty_rect.x + 20, empty_rect.y + 50, "Click '+' in the top right to add a city.", Color::ACCENT_BLUE);
            return;
        }

        // Layout: 2 columns if width >= 500, else 1 column
        let cols = if cards_w >= 520 { 2 } else { 1 };
        let col_gap = 12;
        let card_w = (cards_w - (cols - 1) * col_gap) / cols;
        let card_h = 76;

        let local_offset = hub.world_mgr.pinned.first().map(|p| p.city.utc_offset_minutes).unwrap_or(360);

        for (idx, pinned) in hub.world_mgr.pinned.iter().enumerate() {
            let col = (idx as i32) % cols;
            let row = (idx as i32) / cols;
            let cx = r.x + padding + col * (card_w + col_gap);
            let cy = cards_y + row * (card_h + 8);

            if cy + card_h > r.y + r.h {
                break; // Card overflow clipping
            }

            let card_rect = Rect::new(cx, cy, card_w, card_h);
            let is_selected = idx == hub.world_mgr.selected_idx;

            // Card background & selection glow
            let bg_color = if is_selected {
                Color::rgb(26, 32, 46)
            } else {
                Color::SURFACE_CARD
            };
            let border_color = if is_selected {
                Color::ACCENT_BLUE
            } else {
                Color::BORDER_DARK
            };

            s.fill_rounded_rect(card_rect, 8, bg_color);
            s.draw_rounded_rect(card_rect, 8, border_color);

            let city_time = hub.world_mgr.city_time(&pinned.city);
            let is_day = hub.world_mgr.is_daytime(&pinned.city);

            // Vector Analog Clock on the card
            let clock_cx = cx + 38;
            let clock_cy = cy + card_h / 2;
            let clock_rad = 28;
            s.draw_analog_clock(
                clock_cx,
                clock_cy,
                clock_rad,
                city_time.hour,
                city_time.minute,
                city_time.second,
                is_day,
            );

            // City Name & Relative Offset
            let info_x = clock_cx + clock_rad + 14;
            draw_text(s, info_x, cy + 14, pinned.city.name, Color::TEXT_PRIMARY);

            let rel_str = hub.world_mgr.relative_offset_str(&pinned.city, local_offset);
            draw_text_small(s, info_x, cy + 34, &rel_str, Color::TEXT_MUTED);

            let sun_str = format!("SR {:02}:{:02} • SS {:02}:{:02}", pinned.city.sunrise_hour, pinned.city.sunrise_min, pinned.city.sunset_hour, pinned.city.sunset_min);
            draw_text_small(s, info_x, cy + 50, &sun_str, Color::rgba(140, 160, 190, 180));

            // Digital Time on Right Edge of Card
            let (h12, ampm) = if city_time.hour >= 12 {
                (if city_time.hour == 12 { 12 } else { city_time.hour - 12 }, "PM")
            } else {
                (if city_time.hour == 0 { 12 } else { city_time.hour }, "AM")
            };
            let time_str = format!("{:02}:{:02}", h12, city_time.minute);

            let dig_x = cx + card_w - 78;
            draw_text(s, dig_x, cy + 18, &time_str, Color::TEXT_PRIMARY);
            draw_text_small(s, dig_x + 44, cy + 20, ampm, Color::TEXT_SECONDARY);

            // Delete 'x' Button
            let del_x = cx + card_w - 20;
            let del_y = cy + 6;
            draw_text_small(s, del_x, del_y, "x", Color::TEXT_MUTED);
        }
    }

    /// Tab 2: Alarms view with schedules, toggle switches, and empty state.
    fn render_alarms<S: Surface + ?Sized>(s: &mut S, r: Rect, hub: &ClockHub) {
        let padding = 16;
        if hub.alarm_mgr.alarms.is_empty() {
            // macOS / Ubuntu style clean empty state
            let box_w = 340;
            let box_h = 160;
            let box_x = r.x + (r.w - box_w) / 2;
            let box_y = r.y + (r.h - box_h) / 2 - 20;

            let empty_rect = Rect::new(box_x, box_y, box_w, box_h);
            s.fill_rounded_rect(empty_rect, 10, Color::SURFACE_CARD);
            s.draw_rounded_rect(empty_rect, 10, Color::BORDER_DARK);

            draw_text(s, box_x + 95, box_y + 35, "No Alarms Set", Color::TEXT_PRIMARY);
            draw_text_small(s, box_x + 50, box_y + 65, "Add an alarm to keep track of your schedule.", Color::TEXT_SECONDARY);
            draw_text_small(s, box_x + 75, box_y + 85, "Click '+' in the top bar to get started.", Color::ACCENT_BLUE);

            // Large add alarm button inside empty state
            let add_btn = Rect::new(box_x + 100, box_y + 112, 140, 30);
            s.fill_rounded_rect(add_btn, 6, Color::ACCENT_BLUE);
            draw_text(s, add_btn.x + 28, add_btn.y + 8, "+ Add Alarm", Color::TEXT_PRIMARY);
            return;
        }

        // Alarms list
        let mut y = r.y + padding;
        let card_h = 68;

        for (_idx, alarm) in hub.alarm_mgr.alarms.iter().enumerate() {
            if y + card_h > r.y + r.h - padding {
                break;
            }

            let card_rect = Rect::new(r.x + padding, y, r.w - 2 * padding, card_h);
            s.fill_rounded_rect(card_rect, 8, Color::SURFACE_CARD);
            s.draw_rounded_rect(card_rect, 8, Color::BORDER_DARK);

            let (time_str, ampm) = alarm.format_12h();

            // Large digital time
            let time_color = if alarm.enabled { Color::TEXT_PRIMARY } else { Color::TEXT_MUTED };
            draw_text_font(s, card_rect.x + 16, y + 16, &time_str, time_color, &Font::regular());
            draw_text_small(s, card_rect.x + 65, y + 18, ampm, Color::TEXT_SECONDARY);

            // Label & Repetition summary
            draw_text(s, card_rect.x + 105, y + 14, &alarm.label, time_color);
            draw_text_small(s, card_rect.x + 105, y + 36, &alarm.repeat_summary(), Color::TEXT_SECONDARY);

            // Toggle Switch
            let switch_w = 44;
            let switch_h = 24;
            let switch_x = card_rect.x + card_rect.w - switch_w - 50;
            let switch_y = y + (card_h - switch_h) / 2;
            let switch_rect = Rect::new(switch_x, switch_y, switch_w, switch_h);

            if alarm.enabled {
                s.fill_rounded_rect(switch_rect, 12, Color::STATUS_SUCCESS);
                // Thumb on right
                s.fill_circle(switch_x + switch_w - 12, switch_y + 12, 9, Color::rgb(255, 255, 255));
            } else {
                s.fill_rounded_rect(switch_rect, 12, Color::rgb(50, 56, 70));
                // Thumb on left
                s.fill_circle(switch_x + 12, switch_y + 12, 9, Color::rgb(140, 150, 165));
            }

            // Delete button
            let del_rect = Rect::new(card_rect.x + card_rect.w - 32, y + (card_h - 22) / 2, 22, 22);
            s.fill_rounded_rect(del_rect, 4, Color::rgba(60, 20, 25, 120));
            draw_text_small(s, del_rect.x + 7, del_rect.y + 6, "x", Color::STATUS_ERROR);

            y += card_h + 10;
        }
    }

    /// Tab 3: Stopwatch view with centisecond digital display, controls, and split table.
    fn render_stopwatch<S: Surface + ?Sized>(s: &mut S, r: Rect, hub: &ClockHub) {
        let padding = 16;
        let sw = &hub.stopwatch;

        // 1. Giant Stopwatch Display
        let (h, m, sec, cs) = sw.time_parts();
        let time_str = format!("{:02}:{:02}:{:02}.{:02}", h, m, sec, cs);

        let disp_h = 90;
        let disp_rect = Rect::new(r.x + padding, r.y + padding, r.w - 2 * padding, disp_h);
        s.fill_rounded_rect(disp_rect, 10, Color::SURFACE_CARD);
        s.draw_rounded_rect(disp_rect, 10, Color::BORDER_DARK);

        // Center the large digital text
        let char_w = 20;
        let char_h = 36;
        let total_w = 11 * (char_w + 4);
        let start_x = disp_rect.x + (disp_rect.w - total_w) / 2;
        let start_y = disp_rect.y + (disp_rect.h - char_h) / 2;
        draw_large_digital(s, start_x, start_y, &time_str, char_w, char_h, Color::TEXT_PRIMARY);

        // 2. Action Control Buttons
        let btn_y = disp_rect.y + disp_rect.h + 14;
        let btn_h = 36;
        let btn_w = 110;
        let gap = 20;

        let b1_x = r.x + (r.w - (2 * btn_w + gap)) / 2;
        let b2_x = b1_x + btn_w + gap;

        // Button 1: Lap (when running) or Reset (when stopped/paused)
        let b1_rect = Rect::new(b1_x, btn_y, btn_w, btn_h);
        if sw.state == StopwatchState::Running {
            s.fill_rounded_rect(b1_rect, 6, Color::ACCENT_BLUE);
            draw_text(s, b1_x + 38, btn_y + 11, "Lap", Color::TEXT_PRIMARY);
        } else {
            let is_disabled = sw.elapsed_ms == 0;
            let bg = if is_disabled { Color::rgb(40, 45, 58) } else { Color::rgb(70, 75, 90) };
            let fg = if is_disabled { Color::TEXT_MUTED } else { Color::TEXT_PRIMARY };
            s.fill_rounded_rect(b1_rect, 6, bg);
            draw_text(s, b1_x + 32, btn_y + 11, "Reset", fg);
        }

        // Button 2: Start / Stop
        let b2_rect = Rect::new(b2_x, btn_y, btn_w, btn_h);
        if sw.state == StopwatchState::Running {
            s.fill_rounded_rect(b2_rect, 6, Color::STATUS_ERROR);
            draw_text(s, b2_x + 36, btn_y + 11, "Stop", Color::TEXT_PRIMARY);
        } else {
            s.fill_rounded_rect(b2_rect, 6, Color::STATUS_SUCCESS);
            draw_text(s, b2_x + 34, btn_y + 11, "Start", Color::TEXT_PRIMARY);
        }

        // 3. Laps Telemetry Table
        let table_y = btn_y + btn_h + 16;
        let table_h = r.y + r.h - table_y - padding;
        let table_rect = Rect::new(r.x + padding, table_y, r.w - 2 * padding, table_h);

        s.fill_rounded_rect(table_rect, 8, Color::SURFACE_CARD);
        s.draw_rounded_rect(table_rect, 8, Color::BORDER_DARK);

        // Table Header
        let th_y = table_rect.y + 8;
        s.draw_line(table_rect.x, th_y + 18, table_rect.x + table_rect.w - 1, th_y + 18, Color::BORDER_DARK);
        draw_text_small(s, table_rect.x + 20, th_y, "LAP", Color::TEXT_MUTED);
        draw_text_small(s, table_rect.x + table_rect.w / 2 - 25, th_y, "SPLIT", Color::TEXT_MUTED);
        draw_text_small(s, table_rect.x + table_rect.w - 100, th_y, "TOTAL", Color::TEXT_MUTED);

        if sw.laps.is_empty() {
            draw_text_small(s, table_rect.x + 20, th_y + 35, "No laps recorded. Press 'Lap' while running.", Color::TEXT_MUTED);
            return;
        }

        let fastest_idx = sw.fastest_lap_index();
        let slowest_idx = sw.slowest_lap_index();

        let mut row_y = th_y + 24;
        let row_h = 22;

        // Display laps in reverse order (most recent first)
        for lap in sw.laps.iter().rev() {
            if row_y + row_h > table_rect.y + table_rect.h - 6 {
                break;
            }

            let lap_idx = lap.lap_number - 1;
            let (color, tag) = if Some(lap_idx) == fastest_idx && sw.laps.len() > 1 {
                (Color::STATUS_SUCCESS, " (Fastest)")
            } else if Some(lap_idx) == slowest_idx && sw.laps.len() > 1 {
                (Color::STATUS_ERROR, " (Slowest)")
            } else {
                (Color::TEXT_PRIMARY, "")
            };

            let lap_label = format!("Lap {}{}", lap.lap_number, tag);
            let split_str = chronos_core::stopwatch::format_ms(lap.split_ms);
            let total_str = chronos_core::stopwatch::format_ms(lap.total_ms);

            draw_text_small(s, table_rect.x + 20, row_y + 4, &lap_label, color);
            draw_text_small(s, table_rect.x + table_rect.w / 2 - 25, row_y + 4, &split_str, color);
            draw_text_small(s, table_rect.x + table_rect.w - 100, row_y + 4, &total_str, Color::TEXT_SECONDARY);

            row_y += row_h;
        }
    }

    /// Tab 4: Timers view with countdown display, progress ring, quick presets, and steppers.
    fn render_timers<S: Surface + ?Sized>(s: &mut S, r: Rect, hub: &ClockHub) {
        let padding = 16;
        let tm = &hub.timer;

        // 1. Quick Start Presets (Ubuntu-style grid: 1m, 2m, 3m, 5m, 10m, 15m, 30m, 1h)
        let presets_y = r.y + padding;
        draw_text_small(s, r.x + padding, presets_y, "QUICK START", Color::TEXT_MUTED);

        let p_row_y = presets_y + 16;
        let p_h = 28;
        let p_gap = 8;
        let p_w = (r.w - 2 * padding - 7 * p_gap) / 8;

        for (i, (mins, label)) in QUICK_PRESETS.iter().enumerate() {
            let px = r.x + padding + (i as i32) * (p_w + p_gap);
            let prect = Rect::new(px, p_row_y, p_w, p_h);

            let is_match = tm.total_duration_ms == (*mins as u64) * 60 * 1000;
            let bg = if is_match { Color::ACCENT_BLUE } else { Color::SURFACE_CARD };
            let fg = if is_match { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };

            s.fill_rounded_rect(prect, 6, bg);
            s.draw_rounded_rect(prect, 6, Color::BORDER_DARK);
            draw_text_small(s, px + (p_w - (label.len() as i32) * 6) / 2, p_row_y + 8, label, fg);
        }

        // 2. Main Countdown Card with Circular Progress Arc / Bar
        let timer_card_y = p_row_y + p_h + 14;
        let timer_card_h = 135;
        let timer_card_rect = Rect::new(r.x + padding, timer_card_y, r.w - 2 * padding, timer_card_h);

        s.fill_rounded_rect(timer_card_rect, 10, Color::SURFACE_CARD);
        s.draw_rounded_rect(timer_card_rect, 10, Color::BORDER_DARK);

        // Huge remaining time
        let (th, tm_val, ts) = tm.remaining_parts();
        let time_str = format!("{:02}:{:02}:{:02}", th, tm_val, ts);

        let char_w = 22;
        let char_h = 40;
        let total_w = 8 * (char_w + 4);
        let start_x = timer_card_rect.x + (timer_card_rect.w - total_w) / 2;
        let start_y = timer_card_rect.y + 24;
        let time_col = if tm.state == TimerState::Finished {
            Color::STATUS_ERROR
        } else {
            Color::TEXT_PRIMARY
        };
        draw_large_digital(s, start_x, start_y, &time_str, char_w, char_h, time_col);

        // Progress Bar
        let prog_ratio = tm.progress_ratio();
        let bar_w = timer_card_rect.w - 40;
        let bar_h = 8;
        let bar_x = timer_card_rect.x + 20;
        let bar_y = start_y + char_h + 20;

        let bar_rect = Rect::new(bar_x, bar_y, bar_w, bar_h);
        s.fill_rounded_rect(bar_rect, 4, Color::rgb(35, 42, 58));

        let fill_w = ((bar_w as f32) * prog_ratio) as i32;
        if fill_w > 0 {
            let fill_rect = Rect::new(bar_x, bar_y, fill_w, bar_h);
            let bar_color = if tm.state == TimerState::Finished {
                Color::STATUS_ERROR
            } else {
                Color::ACCENT_ORANGE
            };
            s.fill_rounded_rect(fill_rect, 4, bar_color);
        }

        // Status badge
        let status_str = match tm.state {
            TimerState::Idle => "READY",
            TimerState::Running => "RUNNING",
            TimerState::Paused => "PAUSED",
            TimerState::Finished => "TIME'S UP!",
        };
        draw_text_small(s, timer_card_rect.x + (timer_card_rect.w - (status_str.len() as i32) * 6) / 2, bar_y + 14, status_str, Color::TEXT_MUTED);

        // 3. Time Steppers (HH : MM : SS adjust buttons)
        let steppers_y = timer_card_y + timer_card_h + 14;
        let step_card_rect = Rect::new(r.x + padding, steppers_y, r.w - 2 * padding, 60);
        s.fill_rounded_rect(step_card_rect, 8, Color::SURFACE_CARD);
        s.draw_rounded_rect(step_card_rect, 8, Color::BORDER_DARK);

        let step_slots = [
            ("HOURS", tm.configured_hours),
            ("MINS", tm.configured_mins),
            ("SECS", tm.configured_secs),
        ];

        let slot_w = step_card_rect.w / 3;
        for (i, (label, val)) in step_slots.iter().enumerate() {
            let sx = step_card_rect.x + (i as i32) * slot_w;
            let sy = step_card_rect.y + 10;

            // "-" button
            let minus_rect = Rect::new(sx + 15, sy + 6, 24, 24);
            s.fill_rounded_rect(minus_rect, 4, Color::rgb(40, 46, 62));
            draw_text(s, minus_rect.x + 8, minus_rect.y + 4, "-", Color::TEXT_PRIMARY);

            // Value
            let val_str = format!("{:02}", val);
            draw_text(s, sx + 50, sy + 10, &val_str, Color::TEXT_PRIMARY);
            draw_text_small(s, sx + 72, sy + 12, label, Color::TEXT_MUTED);

            // "+" button
            let plus_rect = Rect::new(sx + slot_w - 40, sy + 6, 24, 24);
            s.fill_rounded_rect(plus_rect, 4, Color::rgb(40, 46, 62));
            draw_text(s, plus_rect.x + 7, plus_rect.y + 4, "+", Color::TEXT_PRIMARY);
        }

        // 4. Action Buttons (Start / Pause / Resume, Cancel)
        let act_y = steppers_y + 60 + 14;
        let act_h = 36;
        let act_w = 110;
        let gap = 20;

        let a1_x = r.x + (r.w - (2 * act_w + gap)) / 2;
        let a2_x = a1_x + act_w + gap;

        // Cancel / Reset Button
        let a1_rect = Rect::new(a1_x, act_y, act_w, act_h);
        s.fill_rounded_rect(a1_rect, 6, Color::rgb(65, 70, 85));
        draw_text(s, a1_x + 30, act_y + 11, "Cancel", Color::TEXT_PRIMARY);

        // Start / Pause / Resume Button
        let a2_rect = Rect::new(a2_x, act_y, act_w, act_h);
        match tm.state {
            TimerState::Idle => {
                s.fill_rounded_rect(a2_rect, 6, Color::STATUS_SUCCESS);
                draw_text(s, a2_x + 36, act_y + 11, "Start", Color::TEXT_PRIMARY);
            }
            TimerState::Running => {
                s.fill_rounded_rect(a2_rect, 6, Color::ACCENT_ORANGE);
                draw_text(s, a2_x + 32, act_y + 11, "Pause", Color::TEXT_PRIMARY);
            }
            TimerState::Paused => {
                s.fill_rounded_rect(a2_rect, 6, Color::STATUS_SUCCESS);
                draw_text(s, a2_x + 28, act_y + 11, "Resume", Color::TEXT_PRIMARY);
            }
            TimerState::Finished => {
                s.fill_rounded_rect(a2_rect, 6, Color::STATUS_ERROR);
                draw_text(s, a2_x + 34, act_y + 11, "Done", Color::TEXT_PRIMARY);
            }
        }
    }

    /// Modal: Choose and Pin a World City.
    fn render_add_city_modal<S: Surface + ?Sized>(s: &mut S, bounds: Rect, hub: &ClockHub) {
        // Semi-transparent backdrop
        s.fill_rect(bounds, Color::rgba(0, 0, 0, 160));

        let modal_w = 380;
        let modal_h = 360;
        let modal_x = bounds.x + (bounds.w - modal_w) / 2;
        let modal_y = bounds.y + (bounds.h - modal_h) / 2;
        let modal_rect = Rect::new(modal_x, modal_y, modal_w, modal_h);

        s.fill_rounded_rect(modal_rect, 10, Color::SURFACE_CARD);
        s.draw_rounded_rect(modal_rect, 10, Color::ACCENT_BLUE);

        // Header
        draw_text(s, modal_x + 20, modal_y + 20, "Choose a City", Color::TEXT_PRIMARY);

        // Close 'x' button
        let close_rect = Rect::new(modal_x + modal_w - 32, modal_y + 16, 20, 20);
        draw_text_small(s, close_rect.x + 6, close_rect.y + 4, "x", Color::TEXT_MUTED);

        // Search Bar
        let search_rect = Rect::new(modal_x + 20, modal_y + 50, modal_w - 40, 28);
        s.fill_rounded_rect(search_rect, 5, Color::SURFACE_BG);
        s.draw_rounded_rect(search_rect, 5, Color::BORDER_DARK);
        let query_text = if hub.city_search_query.is_empty() {
            "Search city or country..."
        } else {
            &hub.city_search_query
        };
        let query_color = if hub.city_search_query.is_empty() {
            Color::TEXT_MUTED
        } else {
            Color::TEXT_PRIMARY
        };
        draw_text_small(s, search_rect.x + 8, search_rect.y + 8, query_text, query_color);

        // List of Available Cities
        let list_y = modal_y + 88;
        let item_h = 34;

        let query_lower = hub.city_search_query.to_lowercase();
        let filtered: Vec<_> = ALL_CITIES
            .iter()
            .filter(|c| {
                if query_lower.is_empty() {
                    true
                } else {
                    c.name.to_lowercase().contains(&query_lower)
                        || c.country.to_lowercase().contains(&query_lower)
                }
            })
            .collect();

        for (i, city) in filtered.iter().enumerate() {
            let cy = list_y + (i as i32) * item_h;
            if cy + item_h > modal_y + modal_h - 10 {
                break;
            }

            let item_rect = Rect::new(modal_x + 20, cy, modal_w - 40, item_h - 4);
            s.fill_rounded_rect(item_rect, 4, Color::rgb(26, 32, 46));

            let is_pinned = hub.world_mgr.pinned.iter().any(|p| p.city.id == city.id);

            draw_text(s, item_rect.x + 10, cy + 6, city.name, Color::TEXT_PRIMARY);
            draw_text_small(s, item_rect.x + 130, cy + 8, city.country, Color::TEXT_MUTED);

            let city_time = hub.world_mgr.city_time(city);
            let time_str = format!("{:02}:{:02}", city_time.hour, city_time.minute);
            draw_text_small(s, item_rect.x + item_rect.w - 90, cy + 8, &time_str, Color::TEXT_SECONDARY);

            if is_pinned {
                draw_text_small(s, item_rect.x + item_rect.w - 45, cy + 8, "Pinned", Color::TEXT_MUTED);
            } else {
                draw_text_small(s, item_rect.x + item_rect.w - 40, cy + 8, "+ Add", Color::ACCENT_BLUE);
            }
        }
    }

    /// Modal: Add / Configure New Alarm.
    fn render_add_alarm_modal<S: Surface + ?Sized>(s: &mut S, bounds: Rect, hub: &ClockHub) {
        // Semi-transparent backdrop
        s.fill_rect(bounds, Color::rgba(0, 0, 0, 160));

        let modal_w = 340;
        let modal_h = 320;
        let modal_x = bounds.x + (bounds.w - modal_w) / 2;
        let modal_y = bounds.y + (bounds.h - modal_h) / 2;
        let modal_rect = Rect::new(modal_x, modal_y, modal_w, modal_h);

        s.fill_rounded_rect(modal_rect, 10, Color::SURFACE_CARD);
        s.draw_rounded_rect(modal_rect, 10, Color::ACCENT_BLUE);

        // Header
        draw_text(s, modal_x + 20, modal_y + 20, "Add Alarm", Color::TEXT_PRIMARY);

        // Close 'x' button
        let close_rect = Rect::new(modal_x + modal_w - 32, modal_y + 16, 20, 20);
        draw_text_small(s, close_rect.x + 6, close_rect.y + 4, "x", Color::TEXT_MUTED);

        // Time Picker (Hour : Minute + AM/PM)
        let tp_y = modal_y + 54;
        let h_str = format!("{:02}", hub.modal_alarm_h);
        let m_str = format!("{:02}", hub.modal_alarm_m);
        let ampm_str = if hub.modal_alarm_pm { "PM" } else { "AM" };

        // Hour Steppers
        let h_minus = Rect::new(modal_x + 40, tp_y, 28, 28);
        s.fill_rounded_rect(h_minus, 4, Color::rgb(40, 46, 62));
        draw_text(s, h_minus.x + 10, h_minus.y + 6, "-", Color::TEXT_PRIMARY);

        draw_text(s, modal_x + 76, tp_y + 6, &h_str, Color::TEXT_PRIMARY);

        let h_plus = Rect::new(modal_x + 102, tp_y, 28, 28);
        s.fill_rounded_rect(h_plus, 4, Color::rgb(40, 46, 62));
        draw_text(s, h_plus.x + 9, h_plus.y + 6, "+", Color::TEXT_PRIMARY);

        draw_text(s, modal_x + 138, tp_y + 6, ":", Color::TEXT_PRIMARY);

        // Minute Steppers
        let m_minus = Rect::new(modal_x + 152, tp_y, 28, 28);
        s.fill_rounded_rect(m_minus, 4, Color::rgb(40, 46, 62));
        draw_text(s, m_minus.x + 10, m_minus.y + 6, "-", Color::TEXT_PRIMARY);

        draw_text(s, modal_x + 188, tp_y + 6, &m_str, Color::TEXT_PRIMARY);

        let m_plus = Rect::new(modal_x + 214, tp_y, 28, 28);
        s.fill_rounded_rect(m_plus, 4, Color::rgb(40, 46, 62));
        draw_text(s, m_plus.x + 9, m_plus.y + 6, "+", Color::TEXT_PRIMARY);

        // AM/PM Toggle Button
        let ampm_rect = Rect::new(modal_x + 254, tp_y, 44, 28);
        s.fill_rounded_rect(ampm_rect, 4, Color::ACCENT_BLUE);
        draw_text(s, ampm_rect.x + 12, ampm_rect.y + 6, ampm_str, Color::TEXT_PRIMARY);

        // Label Input field
        let lbl_y = tp_y + 44;
        draw_text_small(s, modal_x + 20, lbl_y, "LABEL", Color::TEXT_MUTED);

        let lbl_box = Rect::new(modal_x + 20, lbl_y + 16, modal_w - 40, 28);
        s.fill_rounded_rect(lbl_box, 4, Color::SURFACE_BG);
        s.draw_rounded_rect(lbl_box, 4, Color::BORDER_DARK);
        draw_text_small(s, lbl_box.x + 8, lbl_box.y + 8, &hub.modal_alarm_label, Color::TEXT_PRIMARY);

        // Repeat Days Pills (Sun, Mon, Tue, Wed, Thu, Fri, Sat)
        let rep_y = lbl_box.y + 36;
        draw_text_small(s, modal_x + 20, rep_y, "REPEAT", Color::TEXT_MUTED);

        let day_names = ["S", "M", "T", "W", "T", "F", "S"];
        let pill_w = 34;
        let pill_h = 26;
        let pill_gap = 8;
        let pills_start_x = modal_x + 20;

        for (day_idx, day_name) in day_names.iter().enumerate() {
            let px = pills_start_x + (day_idx as i32) * (pill_w + pill_gap);
            let prect = Rect::new(px, rep_y + 16, pill_w, pill_h);

            let is_set = (hub.modal_alarm_repeat & (1 << day_idx)) != 0;
            let bg = if is_set { Color::ACCENT_BLUE } else { Color::SURFACE_BG };
            let fg = if is_set { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };

            s.fill_rounded_rect(prect, 4, bg);
            s.draw_rounded_rect(prect, 4, Color::BORDER_DARK);
            draw_text_small(s, px + 12, prect.y + 7, day_name, fg);
        }

        // Cancel and Save Action Buttons
        let act_y = modal_y + modal_h - 48;
        let btn_w = 120;
        let btn_h = 32;

        let cancel_rect = Rect::new(modal_x + 35, act_y, btn_w, btn_h);
        s.fill_rounded_rect(cancel_rect, 6, Color::rgb(55, 60, 75));
        draw_text(s, cancel_rect.x + 35, act_y + 8, "Cancel", Color::TEXT_PRIMARY);

        let save_rect = Rect::new(modal_x + modal_w - btn_w - 35, act_y, btn_w, btn_h);
        s.fill_rounded_rect(save_rect, 6, Color::STATUS_SUCCESS);
        draw_text(s, save_rect.x + 42, act_y + 8, "Save", Color::TEXT_PRIMARY);
    }

    /// Alert Banner: Ringing Alarm.
    fn render_alarm_alert_overlay<S: Surface + ?Sized>(s: &mut S, bounds: Rect, _hub: &ClockHub) {
        let banner_w = 360;
        let banner_h = 100;
        let bx = bounds.x + (bounds.w - banner_w) / 2;
        let by = bounds.y + 16;
        let brect = Rect::new(bx, by, banner_w, banner_h);

        s.fill_rounded_rect(brect, 8, Color::rgb(40, 15, 20));
        s.draw_rounded_rect(brect, 8, Color::STATUS_ERROR);

        draw_text(s, bx + 20, by + 16, "ALARM RINGING!", Color::STATUS_ERROR);
        draw_text_small(s, bx + 20, by + 38, "Scheduled Alarm Notification", Color::TEXT_PRIMARY);

        // Snooze and Dismiss buttons
        let snooze_btn = Rect::new(bx + 160, by + 56, 80, 28);
        s.fill_rounded_rect(snooze_btn, 4, Color::rgb(60, 65, 80));
        draw_text_small(s, snooze_btn.x + 18, snooze_btn.y + 8, "Snooze", Color::TEXT_PRIMARY);

        let dismiss_btn = Rect::new(bx + 250, by + 56, 90, 28);
        s.fill_rounded_rect(dismiss_btn, 4, Color::STATUS_ERROR);
        draw_text_small(s, dismiss_btn.x + 22, dismiss_btn.y + 8, "Dismiss", Color::TEXT_PRIMARY);
    }

    /// Alert Banner: Timer Finished.
    fn render_timer_alert_overlay<S: Surface + ?Sized>(s: &mut S, bounds: Rect, _hub: &ClockHub) {
        let banner_w = 360;
        let banner_h = 90;
        let bx = bounds.x + (bounds.w - banner_w) / 2;
        let by = bounds.y + 16;
        let brect = Rect::new(bx, by, banner_w, banner_h);

        s.fill_rounded_rect(brect, 8, Color::rgb(35, 25, 10));
        s.draw_rounded_rect(brect, 8, Color::ACCENT_ORANGE);

        draw_text(s, bx + 20, by + 16, "TIMER FINISHED!", Color::ACCENT_ORANGE);
        draw_text_small(s, bx + 20, by + 38, "Your countdown timer has expired.", Color::TEXT_PRIMARY);

        let dismiss_btn = Rect::new(bx + banner_w - 100, by + 46, 80, 28);
        s.fill_rounded_rect(dismiss_btn, 4, Color::ACCENT_ORANGE);
        draw_text_small(s, dismiss_btn.x + 18, dismiss_btn.y + 8, "Dismiss", Color::TEXT_PRIMARY);
    }
}
