//! crates/chronos-core/src/timer.rs
//!
//! Countdown timers, quick preset buttons, progress arc geometry, and alerts.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Clone, Debug)]
pub struct Timer {
    pub state: TimerState,
    pub total_duration_ms: u64,
    pub remaining_ms: u64,
    pub label: String,
    pub sound: String,
    pub configured_hours: u8,
    pub configured_mins: u8,
    pub configured_secs: u8,
}

pub const QUICK_PRESETS: [(u32, &str); 8] = [
    (1, "1 m"),
    (2, "2 m"),
    (3, "3 m"),
    (5, "5 m"),
    (10, "10 m"),
    (15, "15 m"),
    (30, "30 m"),
    (60, "1 h"),
];

impl Default for Timer {
    fn default() -> Self {
        let mut t = Timer {
            state: TimerState::Idle,
            total_duration_ms: 15 * 60 * 1000,
            remaining_ms: 15 * 60 * 1000,
            label: "Timer".to_string(),
            sound: "Radial (Default)".to_string(),
            configured_hours: 0,
            configured_mins: 15,
            configured_secs: 0,
        };
        t.update_total_from_configured();
        t
    }
}

impl Timer {
    pub fn new(hours: u8, mins: u8, secs: u8) -> Self {
        let mut t = Timer {
            state: TimerState::Idle,
            total_duration_ms: 0,
            remaining_ms: 0,
            label: "Timer".to_string(),
            sound: "Radial (Default)".to_string(),
            configured_hours: hours,
            configured_mins: mins,
            configured_secs: secs,
        };
        t.update_total_from_configured();
        t
    }

    pub fn update_total_from_configured(&mut self) {
        let total_secs = self.configured_hours as u64 * 3600
            + self.configured_mins as u64 * 60
            + self.configured_secs as u64;
        self.total_duration_ms = total_secs * 1000;
        self.remaining_ms = self.total_duration_ms;
    }

    pub fn set_hours(&mut self, h: u8) {
        self.configured_hours = h.min(99);
        self.update_total_from_configured();
    }

    pub fn set_mins(&mut self, m: u8) {
        self.configured_mins = m.min(59);
        self.update_total_from_configured();
    }

    pub fn set_secs(&mut self, s: u8) {
        self.configured_secs = s.min(59);
        self.update_total_from_configured();
    }

    pub fn set_duration(&mut self, hours: u8, mins: u8, secs: u8) {
        self.configured_hours = hours.min(99);
        self.configured_mins = mins.min(59);
        self.configured_secs = secs.min(59);
        self.update_total_from_configured();
    }

    pub fn adjust_hours(&mut self, delta: i32) {
        let new_h = (self.configured_hours as i32 + delta).clamp(0, 99) as u8;
        self.set_hours(new_h);
    }

    pub fn adjust_mins(&mut self, delta: i32) {
        let new_m = (self.configured_mins as i32 + delta).clamp(0, 59) as u8;
        self.set_mins(new_m);
    }

    pub fn adjust_secs(&mut self, delta: i32) {
        let new_s = (self.configured_secs as i32 + delta).clamp(0, 59) as u8;
        self.set_secs(new_s);
    }

    pub fn remaining_parts(&self) -> (u8, u8, u8) {
        self.remaining_hms()
    }

    pub fn apply_preset(&mut self, minutes: u32) {
        self.configured_hours = (minutes / 60) as u8;
        self.configured_mins = (minutes % 60) as u8;
        self.configured_secs = 0;
        self.update_total_from_configured();
        self.state = TimerState::Idle;
    }

    pub fn start(&mut self) {
        if self.total_duration_ms == 0 {
            self.configured_mins = 15;
            self.update_total_from_configured();
        }
        if self.state == TimerState::Idle || self.state == TimerState::Finished {
            self.remaining_ms = self.total_duration_ms;
            self.state = TimerState::Running;
        } else if self.state == TimerState::Paused {
            self.state = TimerState::Running;
        }
    }

    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == TimerState::Paused {
            self.state = TimerState::Running;
        }
    }

    pub fn cancel(&mut self) {
        self.state = TimerState::Idle;
        self.remaining_ms = self.total_duration_ms;
    }

    pub fn reset(&mut self) {
        self.cancel();
    }

    /// Advance remaining timer time. Returns true if the timer has just finished.
    pub fn advance_millis(&mut self, delta_ms: u64) -> bool {
        if self.state == TimerState::Running {
            if self.remaining_ms <= delta_ms {
                self.remaining_ms = 0;
                self.state = TimerState::Finished;
                return true;
            } else {
                self.remaining_ms -= delta_ms;
            }
        }
        false
    }

    /// Ratio of elapsed duration from 0.0 (just started) to 1.0 (finished).
    pub fn elapsed_ratio(&self) -> f32 {
        if self.total_duration_ms == 0 {
            return 1.0;
        }
        let elapsed = self.total_duration_ms.saturating_sub(self.remaining_ms);
        (elapsed as f32 / self.total_duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Ratio of remaining duration from 1.0 (full) to 0.0 (finished).
    pub fn remaining_ratio(&self) -> f32 {
        if self.total_duration_ms == 0 {
            return 0.0;
        }
        (self.remaining_ms as f32 / self.total_duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Progress ratio (alias for elapsed_ratio).
    pub fn progress_ratio(&self) -> f32 {
        self.elapsed_ratio()
    }

    /// Split remaining time into (hours, minutes, seconds).
    pub fn remaining_hms(&self) -> (u8, u8, u8) {
        let total_secs = (self.remaining_ms + 999) / 1000;
        let s = (total_secs % 60) as u8;
        let m = ((total_secs / 60) % 60) as u8;
        let h = (total_secs / 3600) as u8;
        (h, m, s)
    }

    /// Formatted countdown display: e.g. "00:15:00".
    pub fn format_countdown(&self) -> String {
        let (h, m, s) = self.remaining_hms();
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_countdown() {
        let mut t = Timer::new(0, 1, 0); // 1 minute
        assert_eq!(t.total_duration_ms, 60_000);
        t.start();
        assert_eq!(t.state, TimerState::Running);
        let finished = t.advance_millis(30_000);
        assert!(!finished);
        assert_eq!(t.remaining_ms, 30_000);
        assert!((t.progress_ratio() - 0.5).abs() < 0.01);

        let finished = t.advance_millis(30_000);
        assert!(finished);
        assert_eq!(t.state, TimerState::Finished);
        assert_eq!(t.remaining_ms, 0);
    }
}
