//! The desktop shell's geometry (see ADR-011).
//!
//! ST-IA is a fixed-size desktop application: one window size for the whole
//! session, decided once at startup, never driven by what the content
//! happens to need. The layout is designed for the shell rather than the
//! shell chasing the layout.
//!
//! The only variable is a safety fallback: a monitor whose usable area is
//! too small for the target size gets a proportionally reduced shell so the
//! app stays usable. That decision is made once, before any window is shown.
//!
//! Kept free of Tauri types so the rules are testable as plain arithmetic.

/// Target shell size. Every screen is composed for exactly this viewport.
pub const SHELL_WIDTH: f64 = 900.0;
pub const SHELL_HEIGHT: f64 = 640.0;

/// Floor for the reduced shell on a small monitor. Below this the layout
/// stops being usable, and the internal panels' own overflow takes over
/// instead of shrinking further.
pub const MIN_WIDTH: f64 = 640.0;
pub const MIN_HEIGHT: f64 = 460.0;

/// Fraction of the monitor's usable area the shell may occupy before it is
/// reduced. `work_area` already excludes the macOS menu bar and Dock, so
/// this is purely breathing room: a window that fills its screen edge to
/// edge reads as a mis-sized window, not as a deliberate one.
pub const MAX_WORK_AREA_FRACTION: f64 = 0.92;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// The shell size for a session, given the usable area of the monitor it
/// will open on.
///
/// Returns the target size whenever it fits within `MAX_WORK_AREA_FRACTION`
/// of the area — the overwhelmingly common case, and the one every layout is
/// designed against. Only a genuinely small screen produces anything else.
///
/// Both axes are reduced independently: a short-but-wide monitor keeps the
/// full width and loses only height, rather than being shrunk uniformly into
/// a smaller square.
pub fn shell_size(work_area: Size) -> Size {
    let available = Size {
        width: usable(work_area.width),
        height: usable(work_area.height),
    };
    Size {
        width: axis(SHELL_WIDTH, MIN_WIDTH, available.width),
        height: axis(SHELL_HEIGHT, MIN_HEIGHT, available.height),
    }
}

/// The portion of one work-area dimension the shell may use. A non-finite or
/// non-positive value (no monitor info, a runtime that reports nonsense)
/// yields 0, which `axis` then resolves to the target size — better to open
/// at the designed size than at something derived from garbage.
fn usable(work_area_dimension: f64) -> f64 {
    if work_area_dimension.is_finite() && work_area_dimension > 0.0 {
        work_area_dimension * MAX_WORK_AREA_FRACTION
    } else {
        0.0
    }
}

/// One axis of the shell: the target, unless the screen cannot accommodate
/// it, in which case as much as it can — but never below the floor, and
/// never derived from an unusable measurement.
fn axis(target: f64, min: f64, available: f64) -> f64 {
    if available <= 0.0 {
        return target;
    }
    if available >= target {
        target
    } else {
        available.max(min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAPTOP: Size = Size {
        width: 1512.0,
        height: 916.0,
    };

    #[test]
    fn an_ordinary_screen_gets_exactly_the_target_size() {
        // The case every layout is designed for; it must not be approximate.
        assert_eq!(
            shell_size(LAPTOP),
            Size {
                width: SHELL_WIDTH,
                height: SHELL_HEIGHT
            }
        );
    }

    #[test]
    fn a_large_screen_still_gets_the_target_size() {
        // The shell is fixed, not proportional: a 5K display does not get a
        // bigger window.
        let ultrawide = Size {
            width: 3440.0,
            height: 1410.0,
        };
        assert_eq!(
            shell_size(ultrawide),
            Size {
                width: SHELL_WIDTH,
                height: SHELL_HEIGHT
            }
        );
    }

    #[test]
    fn a_short_screen_loses_height_only() {
        // 92% of 700 = 644, below the 640 target? No — 644 > 640, so the
        // target still fits. Use something genuinely short.
        let short = Size {
            width: 1512.0,
            height: 620.0,
        };
        let result = shell_size(short);
        assert_eq!(
            result.width, SHELL_WIDTH,
            "width must not shrink with height"
        );
        assert!(result.height < SHELL_HEIGHT);
        assert!(result.height <= 620.0 * MAX_WORK_AREA_FRACTION);
    }

    #[test]
    fn a_narrow_screen_loses_width_only() {
        let narrow = Size {
            width: 800.0,
            height: 1000.0,
        };
        let result = shell_size(narrow);
        assert_eq!(
            result.height, SHELL_HEIGHT,
            "height must not shrink with width"
        );
        assert!(result.width < SHELL_WIDTH);
        assert!(result.width <= 800.0 * MAX_WORK_AREA_FRACTION);
    }

    #[test]
    fn the_shell_never_exceeds_the_allowed_fraction_of_the_screen() {
        for area in [
            Size {
                width: 1024.0,
                height: 640.0,
            },
            Size {
                width: 800.0,
                height: 600.0,
            },
            Size {
                width: 1280.0,
                height: 800.0,
            },
        ] {
            let result = shell_size(area);
            // Either it is the target size (which fits), or it is capped.
            assert!(result.width <= (area.width * MAX_WORK_AREA_FRACTION).max(MIN_WIDTH));
            assert!(result.height <= (area.height * MAX_WORK_AREA_FRACTION).max(MIN_HEIGHT));
        }
    }

    #[test]
    fn a_very_small_screen_stops_at_the_floor_rather_than_becoming_unusable() {
        // Past this point the internal panels' own overflow takes over; the
        // shell does not keep shrinking toward nothing.
        let tiny = Size {
            width: 400.0,
            height: 300.0,
        };
        let result = shell_size(tiny);
        assert_eq!(result.width, MIN_WIDTH);
        assert_eq!(result.height, MIN_HEIGHT);
    }

    #[test]
    fn an_unusable_work_area_falls_back_to_the_target_not_to_garbage() {
        // No monitor info, or a runtime reporting nonsense: opening at the
        // designed size is the safe answer.
        for bad in [0.0, -100.0, f64::NAN, f64::INFINITY] {
            let result = shell_size(Size {
                width: bad,
                height: bad,
            });
            assert!(result.width.is_finite() && result.width > 0.0);
            assert!(result.height.is_finite() && result.height > 0.0);
        }
        assert_eq!(
            shell_size(Size {
                width: 0.0,
                height: 0.0
            }),
            Size {
                width: SHELL_WIDTH,
                height: SHELL_HEIGHT
            }
        );
    }

    #[test]
    fn infinite_available_space_is_treated_as_unknown_not_as_unlimited() {
        // `usable` rejects non-finite input, so this lands on the target
        // rather than propagating infinity into a native resize call.
        let result = shell_size(Size {
            width: f64::INFINITY,
            height: f64::INFINITY,
        });
        assert_eq!(result.width, SHELL_WIDTH);
        assert_eq!(result.height, SHELL_HEIGHT);
    }

    #[test]
    fn the_result_is_deterministic_for_the_same_screen() {
        // The shell is decided once per session; the same input must always
        // produce the same window.
        assert_eq!(shell_size(LAPTOP), shell_size(LAPTOP));
    }
}
