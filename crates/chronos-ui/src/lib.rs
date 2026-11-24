//! crates/chronos-ui/src/lib.rs
//!
//! # Chronos UI Engine
//!
//! Pixel-precise 2D rendering engine for Chronos Clock:
//! - Solar terminator day/night equirectangular world map
//! - Vector analog dial clock with real-time hour, minute, and second sweep hands
//! - High-precision digital typography with scalable LCD/LED font
//! - Tabbed interface (World Clock, Alarms, Stopwatch, Timers)
//! - Interactive modals for adding cities and customizing alarms

pub mod geom;
pub mod draw;
pub mod font;
pub mod map;
pub mod views;

pub use geom::{Color, Point, Rect};
pub use draw::{Surface, BufferSurface32};
pub use font::{Font, draw_text, draw_text_small, draw_large_digital};
pub use map::WorldMapWidget;
pub use views::{ChronosView, HEADER_HEIGHT};

use chronos_core::{ClockHub, ClockTab};
use chronos_core::stopwatch::StopwatchState;
use chronos_core::timer::{TimerState, QUICK_PRESETS};
use chronos_core::world::ALL_CITIES;

/// Handles mouse click events on the Chronos window surface, updating the hub state.
/// Returns true if the state changed and redraw is required.
pub fn handle_click(hub: &mut ClockHub, bounds: Rect, click_x: i32, click_y: i32) -> bool {
    let p = Point::new(click_x, click_y);
    if !bounds.contains(p) {
        return false;
    }

    // 1. Handle Active Alerts Overlay (Modal priority)
    if hub.alarm_alert_active {
        let banner_w = 360;
        let _banner_h = 100;
        let bx = bounds.x + (bounds.w - banner_w) / 2;
        let by = bounds.y + 16;
        let snooze_btn = Rect::new(bx + 160, by + 56, 80, 28);
        let dismiss_btn = Rect::new(bx + 250, by + 56, 90, 28);

        if snooze_btn.contains(p) {
            hub.snooze_active_alarm();
            return true;
        } else if dismiss_btn.contains(p) {
            hub.dismiss_active_alarm();
            return true;
        }
        return true; // Consume clicks while alert is active
    }

    if hub.timer_alert_active {
        let banner_w = 360;
        let bx = bounds.x + (bounds.w - banner_w) / 2;
        let by = bounds.y + 16;
        let dismiss_btn = Rect::new(bx + banner_w - 100, by + 46, 80, 28);
        if dismiss_btn.contains(p) {
            hub.timer_alert_active = false;
            hub.timer.reset();
            return true;
        }
        return true;
    }

    // 2. Handle Add City Modal
    if hub.show_add_city_modal {
        let modal_w = 380;
        let modal_h = 360;
        let modal_x = bounds.x + (bounds.w - modal_w) / 2;
        let modal_y = bounds.y + (bounds.h - modal_h) / 2;

        let close_rect = Rect::new(modal_x + modal_w - 32, modal_y + 16, 20, 20);
        if close_rect.contains(p) {
            hub.close_add_city_modal();
            return true;
        }

        // Check city list clicks
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
            if item_rect.contains(p) {
                hub.world_mgr.pin_city(city.id);
                hub.close_add_city_modal();
                return true;
            }
        }
        return true;
    }

    // 3. Handle Add Alarm Modal
    if hub.show_add_alarm_modal {
        let modal_w = 340;
        let modal_h = 320;
        let modal_x = bounds.x + (bounds.w - modal_w) / 2;
        let modal_y = bounds.y + (bounds.h - modal_h) / 2;

        let close_rect = Rect::new(modal_x + modal_w - 32, modal_y + 16, 20, 20);
        if close_rect.contains(p) {
            hub.close_add_alarm_modal();
            return true;
        }

        let tp_y = modal_y + 54;
        // Hour -/+
        let h_minus = Rect::new(modal_x + 40, tp_y, 28, 28);
        let h_plus = Rect::new(modal_x + 102, tp_y, 28, 28);
        if h_minus.contains(p) {
            hub.modal_alarm_h = if hub.modal_alarm_h == 1 { 12 } else { hub.modal_alarm_h - 1 };
            return true;
        }
        if h_plus.contains(p) {
            hub.modal_alarm_h = if hub.modal_alarm_h == 12 { 1 } else { hub.modal_alarm_h + 1 };
            return true;
        }

        // Minute -/+
        let m_minus = Rect::new(modal_x + 152, tp_y, 28, 28);
        let m_plus = Rect::new(modal_x + 214, tp_y, 28, 28);
        if m_minus.contains(p) {
            hub.modal_alarm_m = if hub.modal_alarm_m == 0 { 55 } else { hub.modal_alarm_m.saturating_sub(5) };
            return true;
        }
        if m_plus.contains(p) {
            hub.modal_alarm_m = if hub.modal_alarm_m >= 55 { 0 } else { hub.modal_alarm_m + 5 };
            return true;
        }

        // AM/PM toggle
        let ampm_rect = Rect::new(modal_x + 254, tp_y, 44, 28);
        if ampm_rect.contains(p) {
            hub.modal_alarm_pm = !hub.modal_alarm_pm;
            return true;
        }

        // Repeat day pills
        let rep_y = tp_y + 44 + 16 + 28 + 16;
        let pill_w = 34;
        let pill_h = 26;
        let pill_gap = 8;
        let pills_start_x = modal_x + 20;

        for day_idx in 0..7 {
            let px = pills_start_x + (day_idx as i32) * (pill_w + pill_gap);
            let prect = Rect::new(px, rep_y, pill_w, pill_h);
            if prect.contains(p) {
                hub.modal_alarm_repeat ^= 1 << day_idx;
                return true;
            }
        }

        // Cancel / Save buttons
        let act_y = modal_y + modal_h - 48;
        let btn_w = 120;
        let btn_h = 32;

        let cancel_rect = Rect::new(modal_x + 35, act_y, btn_w, btn_h);
        if cancel_rect.contains(p) {
            hub.close_add_alarm_modal();
            return true;
        }

        let save_rect = Rect::new(modal_x + modal_w - btn_w - 35, act_y, btn_w, btn_h);
        if save_rect.contains(p) {
            hub.save_modal_alarm();
            return true;
        }
        return true;
    }

    // 4. Handle Header Clicks (Tabs & Add button)
    let header_rect = Rect::new(bounds.x, bounds.y, bounds.w, HEADER_HEIGHT);
    if header_rect.contains(p) {
        let tabs = [
            ClockTab::WorldClock,
            ClockTab::Alarms,
            ClockTab::Stopwatch,
            ClockTab::Timers,
        ];
        let tab_w = 95;
        let tab_h = 28;
        let total_tabs_w = tab_w * (tabs.len() as i32);
        let start_x = header_rect.x + (header_rect.w - total_tabs_w) / 2;
        let tab_y = header_rect.y + (header_rect.h - tab_h) / 2;

        for (i, tab_kind) in tabs.iter().enumerate() {
            let tx = start_x + (i as i32) * tab_w;
            let trect = Rect::new(tx, tab_y, tab_w, tab_h);
            if trect.contains(p) {
                hub.set_tab(*tab_kind);
                return true;
            }
        }

        // Check "+" Button
        let can_add = hub.active_tab == ClockTab::WorldClock || hub.active_tab == ClockTab::Alarms;
        if can_add {
            let plus_w = 28;
            let plus_h = 28;
            let plus_x = header_rect.x + header_rect.w - plus_w - 14;
            let plus_y = header_rect.y + (header_rect.h - plus_h) / 2;
            let plus_rect = Rect::new(plus_x, plus_y, plus_w, plus_h);

            if plus_rect.contains(p) {
                if hub.active_tab == ClockTab::WorldClock {
                    hub.open_add_city_modal();
                } else if hub.active_tab == ClockTab::Alarms {
                    hub.open_add_alarm_modal();
                }
                return true;
            }
        }
        return false;
    }

    // 5. Handle Tab Specific Content Clicks
    let r = Rect::new(bounds.x, bounds.y + HEADER_HEIGHT, bounds.w, bounds.h - HEADER_HEIGHT);

    match hub.active_tab {
        ClockTab::WorldClock => {
            let padding = 12;
            let map_h = ((r.h as f32) * 0.46) as i32;
            let cards_y = r.y + padding + map_h + padding;
            let cards_w = r.w - 2 * padding;
            let cols = if cards_w >= 520 { 2 } else { 1 };
            let col_gap = 12;
            let card_w = (cards_w - (cols - 1) * col_gap) / cols;
            let card_h = 76;

            for (idx, _pinned) in hub.world_mgr.pinned.iter().enumerate() {
                let col = (idx as i32) % cols;
                let row = (idx as i32) / cols;
                let cx = r.x + padding + col * (card_w + col_gap);
                let cy = cards_y + row * (card_h + 8);
                let card_rect = Rect::new(cx, cy, card_w, card_h);

                if card_rect.contains(p) {
                    // Check delete button
                    let del_rect = Rect::new(cx + card_w - 24, cy, 24, 24);
                    if del_rect.contains(p) {
                        hub.world_mgr.unpin_city(idx);
                        return true;
                    } else {
                        hub.world_mgr.selected_idx = idx;
                        return true;
                    }
                }
            }
        }
        ClockTab::Alarms => {
            let padding = 16;
            if hub.alarm_mgr.alarms.is_empty() {
                // Check empty state "+ Add Alarm" button
                let box_w = 340;
                let box_h = 160;
                let box_x = r.x + (r.w - box_w) / 2;
                let box_y = r.y + (r.h - box_h) / 2 - 20;
                let add_btn = Rect::new(box_x + 100, box_y + 112, 140, 30);
                if add_btn.contains(p) {
                    hub.open_add_alarm_modal();
                    return true;
                }
            } else {
                let mut y = r.y + padding;
                let card_h = 68;

                for idx in 0..hub.alarm_mgr.alarms.len() {
                    let card_rect = Rect::new(r.x + padding, y, r.w - 2 * padding, card_h);
                    if card_rect.contains(p) {
                        // Switch toggle
                        let switch_w = 44;
                        let switch_h = 24;
                        let switch_x = card_rect.x + card_rect.w - switch_w - 50;
                        let switch_y = y + (card_h - switch_h) / 2;
                        let switch_rect = Rect::new(switch_x, switch_y, switch_w, switch_h);

                        // Delete button
                        let del_rect = Rect::new(card_rect.x + card_rect.w - 32, y + (card_h - 22) / 2, 22, 22);

                        if switch_rect.contains(p) {
                            hub.alarm_mgr.toggle_alarm_at(idx);
                            return true;
                        } else if del_rect.contains(p) {
                            hub.alarm_mgr.delete_alarm_at(idx);
                            return true;
                        }
                    }
                    y += card_h + 10;
                }
            }
        }
        ClockTab::Stopwatch => {
            let padding = 16;
            let disp_h = 90;
            let disp_y = r.y + padding;
            let btn_y = disp_y + disp_h + 14;
            let btn_h = 36;
            let btn_w = 110;
            let gap = 20;

            let b1_x = r.x + (r.w - (2 * btn_w + gap)) / 2;
            let b2_x = b1_x + btn_w + gap;

            let b1_rect = Rect::new(b1_x, btn_y, btn_w, btn_h);
            let b2_rect = Rect::new(b2_x, btn_y, btn_w, btn_h);

            // B1: Lap or Reset
            if b1_rect.contains(p) {
                if hub.stopwatch.state == StopwatchState::Running {
                    hub.stopwatch.record_lap();
                } else {
                    hub.stopwatch.reset();
                }
                return true;
            }

            // B2: Start or Stop
            if b2_rect.contains(p) {
                if hub.stopwatch.state == StopwatchState::Running {
                    hub.stopwatch.pause();
                } else {
                    hub.stopwatch.start();
                }
                return true;
            }
        }
        ClockTab::Timers => {
            let padding = 16;
            let presets_y = r.y + padding;
            let p_row_y = presets_y + 16;
            let p_h = 28;
            let p_gap = 8;
            let p_w = (r.w - 2 * padding - 7 * p_gap) / 8;

            // 1. Check Quick Start Presets
            for (i, (mins, _)) in QUICK_PRESETS.iter().enumerate() {
                let px = r.x + padding + (i as i32) * (p_w + p_gap);
                let prect = Rect::new(px, p_row_y, p_w, p_h);
                if prect.contains(p) {
                    hub.timer.set_duration(0, *mins as u8, 0);
                    hub.timer.start();
                    return true;
                }
            }

            // 2. Main Countdown Card & Steppers
            let timer_card_y = p_row_y + p_h + 14;
            let timer_card_h = 135;
            let steppers_y = timer_card_y + timer_card_h + 14;
            let step_card_rect = Rect::new(r.x + padding, steppers_y, r.w - 2 * padding, 60);

            if step_card_rect.contains(p) {
                let slot_w = step_card_rect.w / 3;
                for i in 0..3 {
                    let sx = step_card_rect.x + (i as i32) * slot_w;
                    let sy = step_card_rect.y + 10;
                    let minus_rect = Rect::new(sx + 15, sy + 6, 24, 24);
                    let plus_rect = Rect::new(sx + slot_w - 40, sy + 6, 24, 24);

                    if minus_rect.contains(p) {
                        match i {
                            0 => hub.timer.adjust_hours(-1),
                            1 => hub.timer.adjust_mins(-1),
                            2 => hub.timer.adjust_secs(-1),
                            _ => {}
                        }
                        return true;
                    }
                    if plus_rect.contains(p) {
                        match i {
                            0 => hub.timer.adjust_hours(1),
                            1 => hub.timer.adjust_mins(1),
                            2 => hub.timer.adjust_secs(1),
                            _ => {}
                        }
                        return true;
                    }
                }
            }

            // 3. Action Buttons (Cancel, Start/Pause/Resume)
            let act_y = steppers_y + 60 + 14;
            let act_h = 36;
            let act_w = 110;
            let gap = 20;

            let a1_x = r.x + (r.w - (2 * act_w + gap)) / 2;
            let a2_x = a1_x + act_w + gap;

            let a1_rect = Rect::new(a1_x, act_y, act_w, act_h);
            let a2_rect = Rect::new(a2_x, act_y, act_w, act_h);

            if a1_rect.contains(p) {
                hub.timer.reset();
                return true;
            }

            if a2_rect.contains(p) {
                match hub.timer.state {
                    TimerState::Idle => hub.timer.start(),
                    TimerState::Running => hub.timer.pause(),
                    TimerState::Paused => hub.timer.resume(),
                    TimerState::Finished => hub.timer.reset(),
                }
                return true;
            }
        }
    }

    false
}
