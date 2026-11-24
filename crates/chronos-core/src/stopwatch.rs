//! crates/chronos-core/src/stopwatch.rs
//!
//! Precision sub-millisecond digital stopwatch with lap recording and split telemetry.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopwatchState {
    Stopped,
    Running,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lap {
    pub lap_number: usize,
    pub split_ms: u64,
    pub total_ms: u64,
}

#[derive(Clone, Debug)]
pub struct Stopwatch {
    pub state: StopwatchState,
    pub elapsed_ms: u64,
    pub last_lap_ms: u64,
    pub laps: Vec<Lap>,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Stopwatch {
            state: StopwatchState::Stopped,
            elapsed_ms: 0,
            last_lap_ms: 0,
            laps: Vec::new(),
        }
    }
}

impl Stopwatch {
    pub fn start(&mut self) {
        if self.state == StopwatchState::Stopped {
            self.elapsed_ms = 0;
            self.last_lap_ms = 0;
            self.laps.clear();
            self.state = StopwatchState::Running;
        } else if self.state == StopwatchState::Paused {
            self.state = StopwatchState::Running;
        }
    }

    pub fn pause(&mut self) {
        if self.state == StopwatchState::Running {
            self.state = StopwatchState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == StopwatchState::Paused {
            self.state = StopwatchState::Running;
        }
    }

    pub fn stop(&mut self) {
        if self.state == StopwatchState::Running {
            self.state = StopwatchState::Paused;
        }
    }

    pub fn reset(&mut self) {
        self.state = StopwatchState::Stopped;
        self.elapsed_ms = 0;
        self.last_lap_ms = 0;
        self.laps.clear();
    }

    pub fn lap(&mut self) -> Option<Lap> {
        if self.state != StopwatchState::Running {
            return None;
        }
        let lap_num = self.laps.len() + 1;
        let split = self.elapsed_ms.saturating_sub(self.last_lap_ms);
        let lap = Lap {
            lap_number: lap_num,
            split_ms: split,
            total_ms: self.elapsed_ms,
        };
        self.last_lap_ms = self.elapsed_ms;
        self.laps.push(lap);
        Some(lap)
    }

    pub fn advance_millis(&mut self, delta_ms: u64) {
        if self.state == StopwatchState::Running {
            self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms);
        }
    }

    /// Returns index of the fastest lap (minimum split_ms), if there are 2+ laps.
    pub fn fastest_lap_index(&self) -> Option<usize> {
        if self.laps.len() < 2 {
            return None;
        }
        self.laps
            .iter()
            .enumerate()
            .min_by_key(|(_, l)| l.split_ms)
            .map(|(i, _)| i)
    }

    /// Returns index of the slowest lap (maximum split_ms), if there are 2+ laps.
    pub fn slowest_lap_index(&self) -> Option<usize> {
        if self.laps.len() < 2 {
            return None;
        }
        self.laps
            .iter()
            .enumerate()
            .max_by_key(|(_, l)| l.split_ms)
            .map(|(i, _)| i)
    }

    pub fn record_lap(&mut self) -> Option<Lap> {
        self.lap()
    }

    /// Split elapsed time into (hours, minutes, seconds, centiseconds).
    pub fn time_parts(&self) -> (u8, u8, u8, u8) {
        let total_secs = self.elapsed_ms / 1000;
        let cs = ((self.elapsed_ms % 1000) / 10) as u8;
        let s = (total_secs % 60) as u8;
        let m = ((total_secs / 60) % 60) as u8;
        let h = (total_secs / 3600).min(99) as u8;
        (h, m, s, cs)
    }

    /// Formats milliseconds into `(MM:SS, .hh)` (e.g. `("01:23", ".45")`).
    pub fn format_ms(ms: u64) -> (String, String) {
        let total_secs = ms / 1000;
        let centis = (ms % 1000) / 10;
        let secs = total_secs % 60;
        let mins = (total_secs / 60) % 60;
        let hrs = total_secs / 3600;

        if hrs > 0 {
            (format!("{:02}:{:02}:{:02}", hrs, mins, secs), format!(".{:02}", centis))
        } else {
            (format!("{:02}:{:02}", mins, secs), format!(".{:02}", centis))
        }
    }

    /// Complete formatted string: e.g. `00:15.42`.
    pub fn format_full(&self) -> String {
        let (main, sub) = Self::format_ms(self.elapsed_ms);
        format!("{}{}", main, sub)
    }
}

/// Helper function to format milliseconds to "MM:SS.hh"
pub fn format_ms(ms: u64) -> String {
    let (main, sub) = Stopwatch::format_ms(ms);
    format!("{}{}", main, sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopwatch_lifecycle() {
        let mut sw = Stopwatch::default();
        assert_eq!(sw.state, StopwatchState::Stopped);
        sw.start();
        assert_eq!(sw.state, StopwatchState::Running);
        sw.advance_millis(1500);
        assert_eq!(sw.elapsed_ms, 1500);
        let (main, sub) = Stopwatch::format_ms(sw.elapsed_ms);
        assert_eq!(main, "00:01");
        assert_eq!(sub, ".50");

        sw.lap();
        sw.advance_millis(2000);
        sw.lap();
        assert_eq!(sw.laps.len(), 2);
        assert_eq!(sw.fastest_lap_index(), Some(0)); // 1500ms < 2000ms
        assert_eq!(sw.slowest_lap_index(), Some(1));
    }
}
