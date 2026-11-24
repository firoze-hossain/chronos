# Chronos Clock 🕰️

**Chronos** is the sovereign, pure-Rust default Clock application for **NovaOS**. Synthesizing the refined visual elegance of **macOS Clock** (equirectangular solar terminator world map, vector analog clock dial cards, and clean typography) with the powerful utilitarian controls of **Ubuntu Clock** (Quick Start preset grid, dynamic countdown rings, and sub-millisecond lap split telemetry).

---

## 🌟 Key Features

### 1. 🌍 World Clock & Solar Geometry
- **Real-Time Solar Terminator**: Mathematical equirectangular projection of Earth's solar day/night boundary using solar declination:
  $$\sin(\text{alt}) = \sin(\text{lat})\sin(\text{dec}) + \cos(\text{lat})\cos(\text{dec})\cos(\text{lon} - \text{lon}_{\text{sun}}) \ge 0$$
- **Global Hubs**: 24 built-in global metropolitan centers (Dhaka, London, New York, Tokyo, Sydney, Paris, Dubai, Singapore, San Francisco, etc.).
- **Live Analog Dial Cards**: Vector rendered analog clock dials with moving hour, minute, and second sweep hands, styled with daylight vs night face dials.
- **Solar Telemetry**: Relative UTC offsets ("Today, +6 HRS") and sunrise/sunset times.
- **City Search & Pinning Modal**: Dynamic city picker dialog with substring search and pin/unpin toggles.

### 2. ⏰ Multi-Alarm Schedule
- **Repeat Bitmasks**: Flexible repetition bitmasks supporting one-time, daily, weekday (`Mon..Fri`), weekend (`Sat..Sun`), or custom schedules.
- **Animated Toggle Switches**: Smooth visual switches with instant status changes.
- **Snooze Engine**: Configurable 9-minute snooze logic with automatic reschedule.
- **Add Alarm Modal**: Stepper controls for hours, minutes, AM/PM, repeat day pills, custom label input, and alarm sound selection.

### 3. ⏱️ Precision Centisecond Stopwatch
- **High-Frequency Timer**: 10-millisecond centisecond digital readout (`HH:MM:SS.cs`).
- **Telemetry Analysis**:
  - Fastest lap split highlighted in **vibrant green**.
  - Slowest lap split highlighted in **alert red**.
- **Interactive Controls**: Start, Stop, Lap, and Reset.

### 4. ⏳ Countdown Timers & Quick Presets
- **Ubuntu Quick Start Grid**: Instant one-click presets for `1m`, `2m`, `3m`, `5m`, `10m`, `15m`, `30m`, and `1h`.
- **Custom Steppers**: Precision `+`/`-` steppers for hours, minutes, and seconds.
- **Dynamic Progress Bar**: Visual progress indicator tracking remaining countdown time.
- **Expiration Alerts**: Audible and visual overlay banner when countdown expires.

---

## 🏗️ Workspace Architecture

```
chronos/
├── Cargo.toml
├── crates/
│   ├── chronos-core/       # Pure domain logic, DateTime math, World Clock, Alarms, Stopwatch, Timer
│   │   ├── src/
│   │   │   ├── time.rs     # Leap year math, UTC offsets, formatting
│   │   │   ├── world.rs    # 24 cities, solar declination, day/night geometry
│   │   │   ├── alarm.rs    # Repetition bitmasks, snooze, alert trigger checks
│   │   │   ├── stopwatch.rs# Centisecond timer, lap split telemetry
│   │   │   ├── timer.rs    # Countdown timer, quick presets, progress ratios
│   │   │   └── lib.rs      # ClockHub coordinator
│   │   └── Cargo.toml
│   ├── chronos-ui/         # 2D software rasterizer & segmented views
│   │   ├── src/
│   │   │   ├── geom.rs     # Rect, Point, Color ARGB
│   │   │   ├── draw.rs     # Surface trait, vector analog clock dial
│   │   │   ├── font.rs     # Clean typography & large digital fonts
│   │   │   ├── map.rs      # Equirectangular day/night world map
│   │   │   ├── views.rs    # 4 tab views, modal dialogs, alert banners
│   │   │   └── lib.rs      # Hit testing & click dispatch
│   │   └── Cargo.toml
│   └── chronos-app/        # Standalone CLI driver & qualification runner
│       ├── src/main.rs     # Autonomous self-test suite (--selftest, --benchmark, --status)
│       └── Cargo.toml
└── novaos/
    ├── ffi.rs              # Ring-3 system call bindings
    ├── chronos.rs          # #![no_std] ELF32 entry point
    └── build.sh            # Compiles CLOCK.ELF and CLOCK.PKG into NovaOS fixtures
```

---

## 🚀 Running & Verification

```bash
# Run unit tests
cargo test

# Run full autonomous qualification suite
cargo run -p chronos-app -- --selftest

# Check rasterization benchmark
cargo run -p chronos-app -- --benchmark

# Compile for NovaOS ring-3
./novaos/build.sh
```
