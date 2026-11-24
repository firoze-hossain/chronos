//! crates/chronos-core/src/world.rs
//!
//! Global City database, World Clock calculations, and Solar Terminator geometry.

use crate::time::DateTime;

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub id: &'static str,
    pub name: &'static str,
    pub country: &'static str,
    pub lat: f32, // -90.0 to +90.0
    pub lon: f32, // -180.0 to +180.0
    pub tz_code: &'static str,
    pub utc_offset_minutes: i32,
    pub sunrise_hour: u8,
    pub sunrise_min: u8,
    pub sunset_hour: u8,
    pub sunset_min: u8,
}

pub const ALL_CITIES: [City; 24] = [
    City { id: "dac", name: "Dhaka", country: "Bangladesh", lat: 23.8103, lon: 90.4125, tz_code: "BST", utc_offset_minutes: 360, sunrise_hour: 6, sunrise_min: 16, sunset_hour: 17, sunset_min: 11 },
    City { id: "nyc", name: "New York", country: "United States", lat: 40.7128, lon: -74.0060, tz_code: "EST", utc_offset_minutes: -300, sunrise_hour: 6, sunrise_min: 52, sunset_hour: 16, sunset_min: 35 },
    City { id: "lon", name: "London", country: "United Kingdom", lat: 51.5074, lon: -0.1278, tz_code: "GMT", utc_offset_minutes: 0, sunrise_hour: 7, sunrise_min: 34, sunset_hour: 16, sunset_min: 2 },
    City { id: "tyo", name: "Tokyo", country: "Japan", lat: 35.6762, lon: 139.6503, tz_code: "JST", utc_offset_minutes: 540, sunrise_hour: 6, sunrise_min: 25, sunset_hour: 16, sunset_min: 31 },
    City { id: "syd", name: "Sydney", country: "Australia", lat: -33.8688, lon: 151.2093, tz_code: "AEST", utc_offset_minutes: 660, sunrise_hour: 5, sunrise_min: 40, sunset_hour: 19, sunset_min: 45 },
    City { id: "par", name: "Paris", country: "France", lat: 48.8566, lon: 2.3522, tz_code: "CET", utc_offset_minutes: 60, sunrise_hour: 8, sunrise_min: 10, sunset_hour: 17, sunset_min: 4 },
    City { id: "dxb", name: "Dubai", country: "United Arab Emirates", lat: 25.2048, lon: 55.2708, tz_code: "GST", utc_offset_minutes: 240, sunrise_hour: 6, sunrise_min: 42, sunset_hour: 17, sunset_min: 33 },
    City { id: "sin", name: "Singapore", country: "Singapore", lat: 1.3521, lon: 103.8198, tz_code: "SGT", utc_offset_minutes: 480, sunrise_hour: 6, sunrise_min: 51, sunset_hour: 18, sunset_min: 54 },
    City { id: "sfo", name: "San Francisco", country: "United States", lat: 37.7749, lon: -122.4194, tz_code: "PST", utc_offset_minutes: -480, sunrise_hour: 7, sunrise_min: 2, sunset_hour: 16, sunset_min: 52 },
    City { id: "cai", name: "Cairo", country: "Egypt", lat: 30.0444, lon: 31.2357, tz_code: "EET", utc_offset_minutes: 120, sunrise_hour: 6, sunrise_min: 27, sunset_hour: 16, sunset_min: 58 },
    City { id: "ber", name: "Berlin", country: "Germany", lat: 52.5200, lon: 13.4050, tz_code: "CET", utc_offset_minutes: 60, sunrise_hour: 7, sunrise_min: 48, sunset_hour: 16, sunset_min: 6 },
    City { id: "del", name: "New Delhi", country: "India", lat: 28.6139, lon: 77.2090, tz_code: "IST", utc_offset_minutes: 330, sunrise_hour: 6, sunrise_min: 52, sunset_hour: 17, sunset_min: 25 },
    City { id: "hkg", name: "Hong Kong", country: "Hong Kong", lat: 22.3193, lon: 114.1694, tz_code: "HKT", utc_offset_minutes: 480, sunrise_hour: 6, sunrise_min: 41, sunset_hour: 17, sunset_min: 39 },
    City { id: "sao", name: "Sao Paulo", country: "Brazil", lat: -23.5505, lon: -46.6333, tz_code: "BRT", utc_offset_minutes: -180, sunrise_hour: 5, sunrise_min: 15, sunset_hour: 18, sunset_min: 42 },
    City { id: "tor", name: "Toronto", country: "Canada", lat: 43.6532, lon: -79.3832, tz_code: "EST", utc_offset_minutes: -300, sunrise_hour: 7, sunrise_min: 22, sunset_hour: 16, sunset_min: 48 },
    City { id: "rom", name: "Rome", country: "Italy", lat: 41.9028, lon: 12.4964, tz_code: "CET", utc_offset_minutes: 60, sunrise_hour: 7, sunrise_min: 12, sunset_hour: 16, sunset_min: 44 },
    City { id: "bkk", name: "Bangkok", country: "Thailand", lat: 13.7563, lon: 100.5018, tz_code: "ICT", utc_offset_minutes: 420, sunrise_hour: 6, sunrise_min: 21, sunset_hour: 17, sunset_min: 48 },
    City { id: "hnl", name: "Honolulu", country: "United States", lat: 21.3069, lon: -157.8583, tz_code: "HST", utc_offset_minutes: -600, sunrise_hour: 6, sunrise_min: 49, sunset_hour: 17, sunset_min: 49 },
    City { id: "sel", name: "Seoul", country: "South Korea", lat: 37.5665, lon: 126.9780, tz_code: "KST", utc_offset_minutes: 540, sunrise_hour: 7, sunrise_min: 20, sunset_hour: 17, sunset_min: 16 },
    City { id: "jnb", name: "Johannesburg", country: "South Africa", lat: -26.2041, lon: 28.0473, tz_code: "SAST", utc_offset_minutes: 120, sunrise_hour: 5, sunrise_min: 10, sunset_hour: 18, sunset_min: 45 },
    City { id: "khi", name: "Karachi", country: "Pakistan", lat: 24.8607, lon: 67.0011, tz_code: "PKT", utc_offset_minutes: 300, sunrise_hour: 6, sunrise_min: 58, sunset_hour: 17, sunset_min: 42 },
    City { id: "rek", name: "Reykjavik", country: "Iceland", lat: 64.1466, lon: -21.9426, tz_code: "GMT", utc_offset_minutes: 0, sunrise_hour: 10, sunrise_min: 15, sunset_hour: 16, sunset_min: 0 },
    City { id: "akl", name: "Auckland", country: "New Zealand", lat: -36.8485, lon: 174.7633, tz_code: "NZDT", utc_offset_minutes: 780, sunrise_hour: 6, sunrise_min: 2, sunset_hour: 20, sunset_min: 18 },
    City { id: "mex", name: "Mexico City", country: "Mexico", lat: 19.4326, lon: -99.1332, tz_code: "CST", utc_offset_minutes: -360, sunrise_hour: 6, sunrise_min: 52, sunset_hour: 18, sunset_min: 0 },
];

#[derive(Clone, Debug, PartialEq)]
pub struct PinnedCity {
    pub city: City,
}

#[derive(Clone, Debug)]
pub struct WorldClockManager {
    pub base_utc: DateTime,
    pub pinned: Vec<PinnedCity>,
    pub selected_idx: usize,
}

impl Default for WorldClockManager {
    fn default() -> Self {
        let mut mgr = WorldClockManager {
            base_utc: DateTime::default(),
            pinned: Vec::new(),
            selected_idx: 0,
        };
        // Default pinned cities: Dhaka (primary local), London, New York, Tokyo
        mgr.pin_city("dac");
        mgr.pin_city("lon");
        mgr.pin_city("nyc");
        mgr.pin_city("tyo");
        mgr
    }
}

impl WorldClockManager {
    pub fn pin_city(&mut self, id: &str) -> bool {
        if self.pinned.iter().any(|p| p.city.id == id) {
            return false; // Already pinned
        }
        if let Some(c) = ALL_CITIES.iter().find(|c| c.id == id) {
            self.pinned.push(PinnedCity { city: c.clone() });
            true
        } else {
            false
        }
    }

    pub fn unpin_city(&mut self, index: usize) -> bool {
        if index < self.pinned.len() {
            self.pinned.remove(index);
            if self.selected_idx >= self.pinned.len() && !self.pinned.is_empty() {
                self.selected_idx = self.pinned.len() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn city_time(&self, city: &City) -> DateTime {
        self.base_utc.with_offset(city.utc_offset_minutes)
    }

    pub fn relative_offset_str(&self, city: &City, local_offset_mins: i32) -> String {
        let diff_mins = city.utc_offset_minutes - local_offset_mins;
        let diff_hours = diff_mins / 60;
        let diff_rem_mins = (diff_mins % 60).abs();

        let day_rel = if diff_mins >= 720 {
            "Tomorrow"
        } else if diff_mins <= -720 {
            "Yesterday"
        } else {
            "Today"
        };

        if diff_rem_mins == 0 {
            if diff_hours >= 0 {
                format!("{}, +{} HRS", day_rel, diff_hours)
            } else {
                format!("{}, {} HRS", day_rel, diff_hours)
            }
        } else {
            let sign = if diff_mins >= 0 { "+" } else { "-" };
            format!("{}, {}{}:{:02} HRS", day_rel, sign, diff_hours.abs(), diff_rem_mins)
        }
    }

    /// Determines if a city is currently in daytime based on its sunrise and sunset.
    pub fn is_daytime(&self, city: &City) -> bool {
        let ct = self.city_time(city);
        let cur_mins = ct.hour as u32 * 60 + ct.minute as u32;
        let sr_mins = city.sunrise_hour as u32 * 60 + city.sunrise_min as u32;
        let ss_mins = city.sunset_hour as u32 * 60 + city.sunset_min as u32;
        cur_mins >= sr_mins && cur_mins < ss_mins
    }

    /// Evaluates if a given geographical coordinate (lat, lon) on Earth is in daylight
    /// based on solar declination and subsolar longitude for the current UTC time.
    ///
    /// Mathematical basis:
    ///   Declination approx: dec = -23.44 * cos(2 * pi * (day_of_year + 10) / 365)
    ///   Subsolar longitude: lon_sun = -(utc_hour + utc_min/60 - 12) * 15 degrees
    ///   Solar altitude angle: sin(alt) = sin(lat)*sin(dec) + cos(lat)*cos(dec)*cos(lon - lon_sun)
    ///   If sin(alt) >= 0, it is daytime; otherwise night.
    pub fn is_coord_in_daylight(&self, lat_deg: f32, lon_deg: f32) -> bool {
        let day_of_year = (self.base_utc.month as f32 - 1.0) * 30.5 + self.base_utc.day as f32;
        let rad = core::f32::consts::PI / 180.0;
        let dec_deg = -23.44 * ((2.0 * core::f32::consts::PI * (day_of_year + 10.0) / 365.25).cos());
        let dec_rad = dec_deg * rad;

        let utc_hours = self.base_utc.hour as f32 + self.base_utc.minute as f32 / 60.0 + self.base_utc.second as f32 / 3600.0;
        let lon_sun_deg = -(utc_hours - 12.0) * 15.0;

        let lat_rad = lat_deg * rad;
        let delta_lon_rad = (lon_deg - lon_sun_deg) * rad;

        let sin_alt = lat_rad.sin() * dec_rad.sin() + lat_rad.cos() * dec_rad.cos() * delta_lon_rad.cos();
        sin_alt >= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_clock_manager() {
        let mut wm = WorldClockManager::default();
        assert_eq!(wm.pinned.len(), 4);
        assert_eq!(wm.pinned[0].city.name, "Dhaka");
        assert_eq!(wm.pinned[1].city.name, "London");

        // Set UTC time to 12:00 (Dhaka is UTC+6 = 18:00)
        wm.base_utc = DateTime::new(2026, 11, 24, 12, 0, 0);
        let dac_time = wm.city_time(&wm.pinned[0].city);
        assert_eq!(dac_time.hour, 18);
        assert_eq!(dac_time.minute, 0);

        let nyc = ALL_CITIES.iter().find(|c| c.id == "nyc").unwrap();
        let nyc_time = wm.city_time(nyc);
        assert_eq!(nyc_time.hour, 7); // UTC 12:00 - 5 hours = 7:00 AM
    }
}
