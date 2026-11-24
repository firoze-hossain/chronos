//! crates/chronos-core/src/time.rs
//!
//! Date, time, timezone conversion, solar geometry, and formatting primitives.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub day_of_week: u8, // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
}

impl Default for DateTime {
    fn default() -> Self {
        DateTime {
            year: 2026,
            month: 11,
            day: 24,
            hour: 18,
            minute: 48,
            second: 0,
            millisecond: 0,
            day_of_week: 2, // Tuesday
        }
    }
}

impl DateTime {
    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        let dow = day_of_week(year, month, day);
        DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond: 0,
            day_of_week: dow,
        }
    }

    /// Advance time by the given number of milliseconds.
    pub fn advance_millis(&mut self, delta_ms: u64) {
        let total_ms = self.millisecond as u64 + delta_ms;
        self.millisecond = (total_ms % 1000) as u16;
        let delta_secs = (total_ms / 1000) as u32;
        if delta_secs > 0 {
            self.advance_seconds(delta_secs);
        }
    }

    /// Advance time by seconds, properly adjusting minutes, hours, days, months, and years.
    pub fn advance_seconds(&mut self, secs: u32) {
        let cur_sec = self.second as u32 + secs;
        self.second = (cur_sec % 60) as u8;
        let cur_min = self.minute as u32 + (cur_sec / 60);
        self.minute = (cur_min % 60) as u8;
        let cur_hr = self.hour as u32 + (cur_min / 60);
        self.hour = (cur_hr % 24) as u8;
        let days_to_add = cur_hr / 24;

        if days_to_add > 0 {
            for _ in 0..days_to_add {
                self.day += 1;
                let max_days = days_in_month(self.year, self.month);
                if self.day > max_days {
                    self.day = 1;
                    self.month += 1;
                    if self.month > 12 {
                        self.month = 1;
                        self.year += 1;
                    }
                }
            }
            self.day_of_week = day_of_week(self.year, self.month, self.day);
        }
    }

    /// Convert to total seconds since midnight.
    pub fn seconds_since_midnight(&self) -> u32 {
        self.hour as u32 * 3600 + self.minute as u32 * 60 + self.second as u32
    }

    /// Formats time in 12-hour format with AM/PM: e.g. "10:24 AM".
    pub fn format_12h(&self) -> (String, &'static str) {
        let is_pm = self.hour >= 12;
        let h12 = match self.hour % 12 {
            0 => 12,
            h => h,
        };
        (format!("{:02}:{:02}", h12, self.minute), if is_pm { "PM" } else { "AM" })
    }

    /// Formats time in 24-hour format: e.g. "18:48:02".
    pub fn format_24h_with_seconds(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    pub fn format_hms(&self) -> String {
        self.format_24h_with_seconds()
    }

    /// Formats date: e.g. "Tuesday, November 24, 2026".
    pub fn format_full_date(&self) -> String {
        let dow_name = match self.day_of_week {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            _ => "Saturday",
        };
        let m_name = match self.month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            _ => "December",
        };
        format!("{}, {} {}, {}", dow_name, m_name, self.day, self.year)
    }

    /// Shift this time by offset_minutes relative to UTC.
    pub fn with_offset(&self, offset_minutes: i32) -> Self {
        let total_mins = self.hour as i32 * 60 + self.minute as i32 + offset_minutes;
        let mut new_day = self.day as i32;
        let mut new_dow = self.day_of_week as i32;
        let norm_mins = if total_mins < 0 {
            new_day -= 1;
            new_dow = (new_dow + 6) % 7;
            (total_mins % 1440 + 1440) % 1440
        } else if total_mins >= 1440 {
            new_day += 1;
            new_dow = (new_dow + 1) % 7;
            total_mins % 1440
        } else {
            total_mins
        };

        let new_hour = (norm_mins / 60) as u8;
        let new_min = (norm_mins % 60) as u8;

        let max_days = days_in_month(self.year, self.month) as i32;
        let (final_year, final_month, final_day) = if new_day < 1 {
            let prev_month = if self.month == 1 { 12 } else { self.month - 1 };
            let prev_year = if self.month == 1 { self.year - 1 } else { self.year };
            (prev_year, prev_month, days_in_month(prev_year, prev_month))
        } else if new_day > max_days {
            let next_month = if self.month == 12 { 1 } else { self.month + 1 };
            let next_year = if self.month == 12 { self.year + 1 } else { self.year };
            (next_year, next_month, 1)
        } else {
            (self.year, self.month, new_day as u8)
        };

        DateTime {
            year: final_year,
            month: final_month,
            day: final_day,
            hour: new_hour,
            minute: new_min,
            second: self.second,
            millisecond: self.millisecond,
            day_of_week: new_dow as u8,
        }
    }
}

pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 => 31,
        2 => if is_leap_year(year) { 29 } else { 28 },
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

pub fn day_of_week(year: u16, month: u8, day: u8) -> u8 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year as i32;
    if month < 3 {
        y -= 1;
    }
    let m = (month.saturating_sub(1) as usize).min(11);
    let dow = (y + y / 4 - y / 100 + y / 400 + t[m] + day as i32) % 7;
    if dow < 0 {
        ((dow + 7) % 7) as u8
    } else {
        dow as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_time_calculations() {
        let dt = DateTime::new(2026, 11, 24, 18, 48, 0);
        assert_eq!(dt.day_of_week, 2); // Tuesday
        let (time_str, period) = dt.format_12h();
        assert_eq!(time_str, "06:48");
        assert_eq!(period, "PM");
        assert_eq!(dt.format_24h_with_seconds(), "18:48:00");
    }

    #[test]
    fn test_time_advance() {
        let mut dt = DateTime::new(2026, 11, 24, 23, 59, 50);
        dt.advance_seconds(15);
        assert_eq!(dt.hour, 0);
        assert_eq!(dt.minute, 0);
        assert_eq!(dt.second, 5);
        assert_eq!(dt.day, 25);
    }
}
