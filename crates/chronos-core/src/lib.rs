//! crates/chronos-core/src/lib.rs
//!
//! # Chronos Core Engine
//!
//! Sovereign timekeeping, international world clocks with solar terminator geometry,
//! precision multi-alarm scheduling, sub-millisecond lap stopwatch, and quick-preset countdown timers.

pub mod time;
pub mod world;
pub mod alarm;
pub mod stopwatch;
pub mod timer;

pub use time::{DateTime, day_of_week, days_in_month, is_leap_year};
pub use world::{City, PinnedCity, WorldClockManager, ALL_CITIES};
pub use alarm::{Alarm, AlarmManager};
pub use stopwatch::{Stopwatch, StopwatchState, Lap};
pub use timer::{Timer, TimerState, QUICK_PRESETS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockTab {
    WorldClock = 0,
    Alarms = 1,
    Stopwatch = 2,
    Timers = 3,
}

#[derive(Clone, Debug)]
pub struct ClockHub {
    pub active_tab: ClockTab,
    pub world_mgr: WorldClockManager,
    pub alarm_mgr: AlarmManager,
    pub stopwatch: Stopwatch,
    pub timer: Timer,

    // Modal & Dialog state
    pub show_add_city_modal: bool,
    pub city_search_query: String,

    pub show_add_alarm_modal: bool,
    pub modal_alarm_h: u8,
    pub modal_alarm_m: u8,
    pub modal_alarm_pm: bool,
    pub modal_alarm_repeat: u8,
    pub modal_alarm_label: String,
    pub modal_alarm_snooze: bool,

    pub timer_alert_active: bool,
    pub alarm_alert_active: bool,
}

impl Default for ClockHub {
    fn default() -> Self {
        ClockHub {
            active_tab: ClockTab::WorldClock,
            world_mgr: WorldClockManager::default(),
            alarm_mgr: AlarmManager::default(),
            stopwatch: Stopwatch::default(),
            timer: Timer::default(),

            show_add_city_modal: false,
            city_search_query: String::new(),

            show_add_alarm_modal: false,
            modal_alarm_h: 7,
            modal_alarm_m: 0,
            modal_alarm_pm: false,
            modal_alarm_repeat: 0b00111110, // Weekdays default
            modal_alarm_label: "Alarm".to_string(),
            modal_alarm_snooze: true,

            timer_alert_active: false,
            alarm_alert_active: false,
        }
    }
}

impl ClockHub {
    /// Advance all subsystems by the given elapsed milliseconds.
    pub fn tick(&mut self, delta_ms: u64) {
        // Advance base world time
        self.world_mgr.base_utc.advance_millis(delta_ms);

        // Advance stopwatch
        self.stopwatch.advance_millis(delta_ms);

        // Advance countdown timer
        if self.timer.advance_millis(delta_ms) {
            self.timer_alert_active = true;
        }

        // Check alarms against current local time (Dhaka / first pinned city)
        let local_dt = if let Some(local) = self.world_mgr.pinned.first() {
            self.world_mgr.city_time(&local.city)
        } else {
            self.world_mgr.base_utc
        };

        if self.alarm_mgr.check_tick(&local_dt).is_some() {
            self.alarm_alert_active = true;
        }
    }

    /// Sync RTC hardware time from host or NovaOS kernel
    pub fn sync_rtc(&mut self, year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8, offset_minutes: i32) {
        let local_dt = DateTime::new(year, month, day, hour, minute, second);
        // Base UTC is local_dt shifted backward by local offset
        self.world_mgr.base_utc = local_dt.with_offset(-offset_minutes);
    }

    pub fn snooze_active_alarm(&mut self) {
        let local_dt = if let Some(local) = self.world_mgr.pinned.first() {
            self.world_mgr.city_time(&local.city)
        } else {
            self.world_mgr.base_utc
        };
        self.alarm_mgr.snooze_active(&local_dt);
        self.alarm_alert_active = false;
    }

    pub fn dismiss_active_alarm(&mut self) {
        self.alarm_mgr.dismiss_active();
        self.alarm_alert_active = false;
    }

    pub fn set_tab(&mut self, tab: ClockTab) {
        self.active_tab = tab;
        self.show_add_city_modal = false;
        self.show_add_alarm_modal = false;
    }

    pub fn open_add_city_modal(&mut self) {
        self.city_search_query.clear();
        self.show_add_city_modal = true;
    }

    pub fn close_add_city_modal(&mut self) {
        self.show_add_city_modal = false;
        self.city_search_query.clear();
    }

    pub fn open_add_alarm_modal(&mut self) {
        self.modal_alarm_h = 7;
        self.modal_alarm_m = 0;
        self.modal_alarm_pm = false;
        self.modal_alarm_repeat = 0b00111110;
        self.modal_alarm_label = "Alarm".to_string();
        self.modal_alarm_snooze = true;
        self.show_add_alarm_modal = true;
    }

    pub fn close_add_alarm_modal(&mut self) {
        self.show_add_alarm_modal = false;
    }

    pub fn save_modal_alarm(&mut self) {
        let mut final_h = self.modal_alarm_h % 12;
        if self.modal_alarm_pm {
            final_h += 12;
        }
        self.alarm_mgr.add_alarm(
            final_h,
            self.modal_alarm_m,
            &self.modal_alarm_label,
            self.modal_alarm_repeat,
        );
        self.show_add_alarm_modal = false;
    }
}
