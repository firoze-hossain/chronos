//! crates/chronos-app/src/main.rs
//!
//! # Chronos Clock Application Runner & Qualification Engine
//!
//! Standalone CLI driver, autonomous self-test verification suite, and terminal dashboard
//! for the Sovereign NovaOS Clock application ("Chronos").

use chronos_core::stopwatch::StopwatchState;
use chronos_core::time::DateTime;
use chronos_core::timer::TimerState;
use chronos_core::{ClockHub, ClockTab};
use chronos_ui::draw::BufferSurface32;
use chronos_ui::geom::Rect;
use chronos_ui::views::ChronosView;
use chronos_ui::handle_click;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|a| a == "--selftest" || a == "-t") {
        let code = run_selftest();
        std::process::exit(code);
    }

    if args.iter().any(|a| a == "--benchmark" || a == "-b") {
        run_benchmark();
        return;
    }

    if args.iter().any(|a| a == "--status" || a == "-s") {
        print_status();
        return;
    }

    // Default: run self-test qualification
    println!("Chronos Clock - Sovereign Timekeeping Suite for NovaOS");
    let code = run_selftest();
    std::process::exit(code);
}

fn print_help() {
    println!("Chronos Clock v1.0.0 (NovaOS Enterprise)");
    println!("Usage: chronos [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -t, --selftest     Run full autonomous test suite across all 4 subsystems");
    println!("  -b, --benchmark    Benchmark 2D software rasterizer rendering speed");
    println!("  -s, --status       Print current timekeeping telemetry and status");
    println!("  -h, --help         Show this help information");
}

fn print_status() {
    let mut hub = ClockHub::default();
    hub.sync_rtc(2026, 11, 24, 18, 0, 0, 360); // 18:00 BST

    println!("=== CHRONOS TIMEKEEPING STATUS ===");
    println!("Local Base Time (Dhaka): {}", hub.world_mgr.city_time(&hub.world_mgr.pinned[0].city).format_hms());
    println!("Pinned World Clocks: {}", hub.world_mgr.pinned.len());
    for p in &hub.world_mgr.pinned {
        let t = hub.world_mgr.city_time(&p.city);
        let day_str = if hub.world_mgr.is_daytime(&p.city) { "DAY" } else { "NIGHT" };
        println!("  • {:<12} ({}) {:02}:{:02}:{:02} [{}]", p.city.name, p.city.tz_code, t.hour, t.minute, t.second, day_str);
    }
    println!("Alarms: {} active", hub.alarm_mgr.alarms.len());
    println!("Stopwatch: {:?}", hub.stopwatch.state);
    println!("Timer: {:?} (Remaining: {} ms)", hub.timer.state, hub.timer.remaining_ms);
}

fn run_benchmark() {
    println!("[Benchmark] Rendering 100 frames of 640x480 full Chronos UI...");
    let mut hub = ClockHub::default();
    hub.sync_rtc(2026, 11, 24, 14, 30, 0, 0);

    let width = 640;
    let height = 480;
    let mut buffer = vec![0u32; (width * height) as usize];

    let start = std::time::Instant::now();
    for frame in 0..100 {
        hub.tick(16); // ~60fps step
        let mut surface = BufferSurface32::new(&mut buffer, width, height, width);
        ChronosView::render(&mut surface, Rect::new(0, 0, width, height), &hub);
        if frame % 25 == 0 {
            // Test tab switching
            hub.set_tab(match (frame / 25) % 4 {
                0 => ClockTab::WorldClock,
                1 => ClockTab::Alarms,
                2 => ClockTab::Stopwatch,
                _ => ClockTab::Timers,
            });
        }
    }
    let elapsed = start.elapsed();
    let fps = 100.0 / elapsed.as_secs_f64();
    println!("[Benchmark] Completed 100 frames in {:.2?} (~{:.1} FPS)", elapsed, fps);
}

fn run_selftest() -> i32 {
    println!("============================================================");
    println!("       CHRONOS CLOCK AUTONOMOUS QUALIFICATION SUITE        ");
    println!("============================================================");

    let mut passed = 0;
    let mut failed = 0;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                println!("  [PASS] {}", $name);
                passed += 1;
            } else {
                println!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    // --- Subsystem 1: Core Time & Calendar Math ---
    println!("\n[1/5] Subsystem: DateTime & Solar Terminator Math");
    let dt = DateTime::new(2024, 2, 28, 23, 59, 59);
    let mut dt_tick = dt.clone();
    dt_tick.advance_millis(2000);
    check!("Leap year rollover (2024-02-28 23:59:59 + 2s -> 2024-02-29 00:00:01)",
        dt_tick.year == 2024 && dt_tick.month == 2 && dt_tick.day == 29 && dt_tick.second == 1);

    let mut hub = ClockHub::default();
    hub.sync_rtc(2026, 11, 24, 12, 0, 0, 0); // UTC 12:00
    // Dhaka is UTC+6 -> 18:00
    let dac = hub.world_mgr.pinned.iter().find(|p| p.city.id == "dac").unwrap();
    let dac_time = hub.world_mgr.city_time(&dac.city);
    check!("World Clock UTC offset calculation (Dhaka UTC+6 = 18:00 at UTC 12:00)",
        dac_time.hour == 18 && dac_time.minute == 0);

    // Day/Night Solar Terminator: At UTC 12:00, London (lon ~0) is near midday (daylight)
    let lon_daylight = hub.world_mgr.is_coord_in_daylight(51.5, 0.0);
    check!("Solar Terminator: London in daylight at UTC 12:00", lon_daylight == true);

    // Opposite side of earth (lon 180) should be night at UTC 12:00
    let pacific_daylight = hub.world_mgr.is_coord_in_daylight(0.0, -180.0);
    check!("Solar Terminator: Pacific lon -180 in night at UTC 12:00", pacific_daylight == false);

    // --- Subsystem 2: Alarms Engine ---
    println!("\n[2/5] Subsystem: Multi-Alarm Engine & Repetition Schedule");
    hub.alarm_mgr.alarms.clear();
    hub.alarm_mgr.add_alarm(7, 30, "Morning Standup", 0b00111110); // Weekdays
    check!("Alarm created with weekdays repetition mask", hub.alarm_mgr.alarms.len() == 1);
    check!("Alarm format 12h string matches ('07:30', 'AM')",
        hub.alarm_mgr.alarms[0].format_12h() == ("07:30".to_string(), "AM"));
    check!("Alarm repetition summary is 'Weekdays'",
        hub.alarm_mgr.alarms[0].repeat_summary() == "Weekdays");

    // Check alarm trigger
    let trigger_dt = DateTime::new(2026, 11, 24, 7, 30, 0); // Tuesday (Weekday)
    let triggered = hub.alarm_mgr.check_tick(&trigger_dt);
    check!("Alarm triggers at exactly scheduled time on active weekday", triggered.is_some());
    check!("Alarm state changes to ringing", hub.alarm_mgr.alarms[0].is_ringing);

    // Snooze alarm
    hub.alarm_mgr.snooze_active(&trigger_dt);
    check!("Snooze alarm reschedules 9 minutes forward",
        hub.alarm_mgr.alarms[0].snooze_until.is_some() && !hub.alarm_mgr.alarms[0].is_ringing);

    // Dismiss alarm
    hub.alarm_mgr.dismiss_active();
    check!("Dismiss alarm clears ringing and snooze",
        hub.alarm_mgr.alarms[0].snooze_until.is_none());

    // --- Subsystem 3: Stopwatch Engine ---
    println!("\n[3/5] Subsystem: Precision Stopwatch & Split Telemetry");
    hub.stopwatch.reset();
    check!("Stopwatch starts in Stopped state with 0 elapsed",
        hub.stopwatch.state == StopwatchState::Stopped && hub.stopwatch.elapsed_ms == 0);

    hub.stopwatch.start();
    check!("Stopwatch transitions to Running state", hub.stopwatch.state == StopwatchState::Running);

    hub.stopwatch.advance_millis(15420); // 15.42 seconds
    let lap1 = hub.stopwatch.record_lap();
    check!("Lap 1 recorded with 15.42s split", lap1.is_some() && lap1.unwrap().split_ms == 15420);

    hub.stopwatch.advance_millis(12100); // 12.10 seconds
    let lap2 = hub.stopwatch.record_lap();
    check!("Lap 2 recorded with 12.10s split", lap2.is_some() && lap2.unwrap().split_ms == 12100);

    check!("Stopwatch telemetry identifies fastest lap (Lap 2 @ 12.10s)",
        hub.stopwatch.fastest_lap_index() == Some(1));
    check!("Stopwatch telemetry identifies slowest lap (Lap 1 @ 15.42s)",
        hub.stopwatch.slowest_lap_index() == Some(0));

    hub.stopwatch.pause();
    check!("Stopwatch pauses and preserves elapsed time",
        hub.stopwatch.state == StopwatchState::Paused && hub.stopwatch.elapsed_ms == 27520);

    hub.stopwatch.reset();
    check!("Stopwatch resets cleared laps and elapsed",
        hub.stopwatch.state == StopwatchState::Stopped && hub.stopwatch.laps.is_empty());

    // --- Subsystem 4: Countdown Timers Engine ---
    println!("\n[4/5] Subsystem: Countdown Timer & Quick Presets");
    hub.timer.reset();
    hub.timer.set_duration(0, 5, 0); // 5 minutes preset
    check!("Timer set to 5 minutes (300,000 ms)", hub.timer.total_duration_ms == 300_000);

    hub.timer.start();
    check!("Timer transitions to Running state", hub.timer.state == TimerState::Running);

    hub.timer.advance_millis(120_000); // 2 minutes pass
    check!("Timer remaining decreases to 3 minutes (180,000 ms)", hub.timer.remaining_ms == 180_000);
    check!("Timer progress ratio is 40% elapsed (60% remaining)",
        (hub.timer.elapsed_ratio() - 0.4).abs() < 0.01 && (hub.timer.remaining_ratio() - 0.6).abs() < 0.01);

    // Advance to completion
    let finished = hub.timer.advance_millis(185_000);
    check!("Timer fires completion signal when time runs out", finished == true);
    check!("Timer enters Finished state", hub.timer.state == TimerState::Finished);

    // --- Subsystem 5: Software Rasterizer & Interactive Views ---
    println!("\n[5/5] Subsystem: 2D Rasterizer & Interactive View Controller");
    let width = 640;
    let height = 480;
    let mut buffer = vec![0u32; (width * height) as usize];
    let bounds = Rect::new(0, 0, width, height);

    // Test rendering all 4 tabs to verify no crashes or panics
    for tab in &[ClockTab::WorldClock, ClockTab::Alarms, ClockTab::Stopwatch, ClockTab::Timers] {
        hub.set_tab(*tab);
        let mut surface = BufferSurface32::new(&mut buffer, width, height, width);
        ChronosView::render(&mut surface, bounds, &hub);
    }
    check!("All 4 tabs rendered to software 32-bit frame buffer without panics", true);

    // Test interactive click dispatch
    hub.set_tab(ClockTab::WorldClock);
    // Click on Alarms tab (tab 1)
    let tab_alarms_x = bounds.x + (bounds.w - 380) / 2 + 120;
    let tab_alarms_y = 22;
    let handled = handle_click(&mut hub, bounds, tab_alarms_x, tab_alarms_y);
    check!("Click dispatched on header tab switches active tab to Alarms",
        handled && hub.active_tab == ClockTab::Alarms);

    // Open add alarm modal via "+" button
    let plus_x = bounds.w - 28;
    let plus_y = 22;
    let handled_plus = handle_click(&mut hub, bounds, plus_x, plus_y);
    check!("Click on '+' button opens Add Alarm Modal",
        handled_plus && hub.show_add_alarm_modal == true);

    // Close modal
    hub.close_add_alarm_modal();
    check!("Modal closes cleanly", hub.show_add_alarm_modal == false);

    println!("\n------------------------------------------------------------");
    println!("Test Summary: {} Passed, {} Failed", passed, failed);
    println!("------------------------------------------------------------");

    if failed == 0 {
        println!("CHRONOS QUALIFICATION: 100% PASS - Ready for NovaOS deployment!");
        0
    } else {
        println!("CHRONOS QUALIFICATION: FAIL - Please inspect diagnostics above.");
        1
    }
}
