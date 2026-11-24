//! crates/chronos-core/src/alarm.rs
//!
//! Precision multi-alarm scheduling, day repetition bitmasks, snooze logic, and alert triggers.

use crate::time::DateTime;

#[derive(Clone, Debug, PartialEq)]
pub struct Alarm {
    pub id: u32,
    pub hour: u8,
    pub minute: u8,
    pub label: String,
    pub sound: String,
    /// Bitmask for repeating days: bit 0 = Sun, 1 = Mon, 2 = Tue, 3 = Wed, 4 = Thu, 5 = Fri, 6 = Sat.
    /// 0 means non-repeating (one-time alarm).
    pub repeat_mask: u8,
    pub enabled: bool,
    pub snooze_enabled: bool,
    pub snooze_duration_mins: u8,
    pub is_ringing: bool,
    pub snooze_until: Option<DateTime>,
}

impl Alarm {
    pub fn new(id: u32, hour: u8, minute: u8, label: &str, repeat_mask: u8) -> Self {
        Alarm {
            id,
            hour: hour.min(23),
            minute: minute.min(59),
            label: label.to_string(),
            sound: "Radial (Default)".to_string(),
            repeat_mask,
            enabled: true,
            snooze_enabled: true,
            snooze_duration_mins: 9,
            is_ringing: false,
            snooze_until: None,
        }
    }

    /// Formats time in 12-hour string (e.g. "07:00", "AM").
    pub fn format_12h(&self) -> (String, &'static str) {
        let is_pm = self.hour >= 12;
        let h12 = match self.hour % 12 {
            0 => 12,
            h => h,
        };
        (format!("{:02}:{:02}", h12, self.minute), if is_pm { "PM" } else { "AM" })
    }

    /// Formats repetition schedule: e.g. "Every day", "Weekdays", "Weekends", "Mon, Wed", "Never".
    pub fn repeat_summary(&self) -> String {
        if self.repeat_mask == 0 {
            return "Once".to_string();
        }
        if self.repeat_mask == 0b01111111 {
            return "Every day".to_string();
        }
        if self.repeat_mask == 0b00111110 {
            return "Weekdays".to_string();
        }
        if self.repeat_mask == 0b01000001 {
            return "Weekends".to_string();
        }

        let mut parts = Vec::new();
        let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for i in 0..7 {
            if (self.repeat_mask & (1 << i)) != 0 {
                parts.push(names[i]);
            }
        }
        parts.join(", ")
    }

    /// Checks if this alarm should trigger right now at the given date/time.
    pub fn should_trigger(&self, now: &DateTime) -> bool {
        if !self.enabled {
            return false;
        }

        // Check if snoozed
        if let Some(snooze_dt) = &self.snooze_until {
            if now.hour == snooze_dt.hour && now.minute == snooze_dt.minute && now.second == 0 {
                return true;
            }
            return false;
        }

        if now.hour != self.hour || now.minute != self.minute || now.second != 0 {
            return false;
        }

        if self.repeat_mask == 0 {
            // One-time alarm
            true
        } else {
            // Check day of week
            (self.repeat_mask & (1 << now.day_of_week)) != 0
        }
    }
}

#[derive(Clone, Debug)]
pub struct AlarmManager {
    pub alarms: Vec<Alarm>,
    pub next_id: u32,
    pub active_ringing_id: Option<u32>,
}

impl Default for AlarmManager {
    fn default() -> Self {
        let mut mgr = AlarmManager {
            alarms: Vec::new(),
            next_id: 1,
            active_ringing_id: None,
        };
        // Add sample default alarms:
        // 7:00 AM Weekdays "Morning Routine"
        mgr.add_alarm(7, 0, "Morning Routine", 0b00111110);
        // 9:30 AM Mon-Fri "Team Standup"
        mgr.add_alarm(9, 30, "Team Standup", 0b00111110);
        mgr
    }
}

impl AlarmManager {
    pub fn add_alarm(&mut self, hour: u8, minute: u8, label: &str, repeat_mask: u8) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.alarms.push(Alarm::new(id, hour, minute, label, repeat_mask));
        id
    }

    pub fn delete_alarm(&mut self, id: u32) -> bool {
        if let Some(pos) = self.alarms.iter().position(|a| a.id == id) {
            self.alarms.remove(pos);
            if self.active_ringing_id == Some(id) {
                self.active_ringing_id = None;
            }
            true
        } else {
            false
        }
    }

    pub fn delete_alarm_at(&mut self, idx: usize) -> bool {
        if idx < self.alarms.len() {
            let id = self.alarms[idx].id;
            self.delete_alarm(id)
        } else {
            false
        }
    }

    pub fn toggle_alarm(&mut self, id: u32) -> bool {
        if let Some(a) = self.alarms.iter_mut().find(|a| a.id == id) {
            a.enabled = !a.enabled;
            if !a.enabled {
                a.is_ringing = false;
                a.snooze_until = None;
            }
            true
        } else {
            false
        }
    }

    pub fn toggle_alarm_at(&mut self, idx: usize) -> bool {
        if idx < self.alarms.len() {
            let id = self.alarms[idx].id;
            self.toggle_alarm(id)
        } else {
            false
        }
    }

    pub fn check_tick(&mut self, now: &DateTime) -> Option<u32> {
        let mut triggered = None;
        for a in &mut self.alarms {
            if a.should_trigger(now) && !a.is_ringing {
                a.is_ringing = true;
                a.snooze_until = None;
                if a.repeat_mask == 0 {
                    a.enabled = false; // Disables one-time alarm after firing
                }
                triggered = Some(a.id);
                break;
            }
        }
        if let Some(id) = triggered {
            self.active_ringing_id = Some(id);
        }
        triggered
    }

    pub fn snooze_active(&mut self, now: &DateTime) {
        if let Some(id) = self.active_ringing_id {
            if let Some(a) = self.alarms.iter_mut().find(|a| a.id == id) {
                a.is_ringing = false;
                let snooze_secs = a.snooze_duration_mins as u32 * 60;
                let mut snooze_dt = *now;
                snooze_dt.advance_seconds(snooze_secs);
                a.snooze_until = Some(snooze_dt);
            }
            self.active_ringing_id = None;
        }
    }

    pub fn dismiss_active(&mut self) {
        if let Some(id) = self.active_ringing_id {
            if let Some(a) = self.alarms.iter_mut().find(|a| a.id == id) {
                a.is_ringing = false;
                a.snooze_until = None;
            }
            self.active_ringing_id = None;
        } else {
            // Dismiss any ringing or snoozed alarm
            for a in &mut self.alarms {
                a.is_ringing = false;
                a.snooze_until = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alarm_management() {
        let mut mgr = AlarmManager::default();
        assert_eq!(mgr.alarms.len(), 2);
        let id = mgr.add_alarm(8, 15, "Gym", 0);
        assert!(mgr.alarms.iter().any(|a| a.id == id));
        mgr.toggle_alarm(id);
        assert!(!mgr.alarms.iter().find(|a| a.id == id).unwrap().enabled);
        mgr.delete_alarm(id);
        assert!(!mgr.alarms.iter().any(|a| a.id == id));
    }

    #[test]
    fn test_alarm_trigger() {
        let mut mgr = AlarmManager::default();
        let dt = DateTime::new(2026, 11, 24, 7, 0, 0); // Tuesday 7:00 AM
        let triggered = mgr.check_tick(&dt);
        assert_eq!(triggered, Some(1));
        assert_eq!(mgr.active_ringing_id, Some(1));
        mgr.dismiss_active();
        assert_eq!(mgr.active_ringing_id, None);
    }
}
