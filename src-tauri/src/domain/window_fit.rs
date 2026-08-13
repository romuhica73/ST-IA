//! Pure sizing policy for the main window (see ADR-011).
//!
//! Kept free of any Tauri type so the rules — respect the screen, respect a
//! floor, never invent a size from garbage input — can be tested as plain
//! arithmetic. `commands::window::fit_window` is the only caller, and it does
//! nothing but gather real numbers (content size, monitor work area) and feed
//! them through here.

/// A width/height pair in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// A top-left position in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Absolute floor. Small enough to stay out of the way on any state, large
/// enough that the header, a file card and one action button are never
/// squeezed. Not tuned to any single screen — it is the last line of
/// defence, not the primary sizing mechanism (that is the real content
/// measurement passed in every time).
pub const MIN_WIDTH: f64 = 480.0;
pub const MIN_HEIGHT: f64 = 400.0;

/// Ceiling on how much of the monitor's usable area (already excluding the
/// menu bar and Dock — see `Monitor::work_area`) the window may claim. 90%
/// leaves visible breathing room around the window on every side, which is
/// what makes a resize read as "the app sized itself" rather than "the app
/// took over the screen".
pub const MAX_WORK_AREA_FRACTION: f64 = 0.90;

/// Clamps one dimension: `min` never wins over `max` (a screen smaller than
/// our floor must still not be exceeded), and a non-finite or non-positive
/// desired value — the "no negative/NaN sizes" guarantee — falls back to the
/// floor rather than propagating garbage to a native resize call.
fn clamp_dimension(desired: f64, min: f64, max: f64) -> f64 {
    let max = max.max(0.0);
    let min = min.min(max);
    let desired = if desired.is_finite() && desired > 0.0 {
        desired
    } else {
        min
    };
    desired.clamp(min, max)
}

/// The window size policy: desired content size, bounded to `[min, max]`.
pub fn clamp_size(desired: Size, min: Size, max: Size) -> Size {
    Size {
        width: clamp_dimension(desired.width, min.width, max.width),
        height: clamp_dimension(desired.height, min.height, max.height),
    }
}

/// `work_area * fraction`, clamping the fraction itself so a bad config value
/// can never invert into "more than the whole screen".
pub fn capped_work_area(work_area: Size, fraction: f64) -> Size {
    let fraction = fraction.clamp(0.0, 1.0);
    Size {
        width: work_area.width * fraction,
        height: work_area.height * fraction,
    }
}

/// The whole policy in one call: given what the content actually needs and
/// the monitor's usable area, decide the window size.
pub fn fit(desired: Size, work_area: Size) -> Size {
    let max = capped_work_area(work_area, MAX_WORK_AREA_FRACTION);
    let min = Size {
        width: MIN_WIDTH,
        height: MIN_HEIGHT,
    };
    clamp_size(desired, min, max)
}

/// Keeps a window's top-left corner inside `area`, shifting it back in only
/// as far as needed. If `size` itself is larger than `area` (degenerate —
/// `fit` should already prevent this, but position clamping must not assume
/// it), the window is pinned to the area's origin rather than left partially
/// off-screen in one direction to satisfy the other.
pub fn clamp_position(pos: Point, size: Size, area_origin: Point, area_size: Size) -> Point {
    let max_x = (area_origin.x + area_size.width - size.width).max(area_origin.x);
    let max_y = (area_origin.y + area_size.height - size.height).max(area_origin.y);
    Point {
        x: pos.x.clamp(area_origin.x, max_x),
        y: pos.y.clamp(area_origin.y, max_y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK_AREA: Size = Size {
        width: 1920.0,
        height: 1080.0,
    };

    /// The width every screen is visually tuned for (file cards, the
    /// language grid, settings rows). Content in this app is single-column
    /// and never legitimately needs more width than this, so width goes
    /// through the same policy mostly for the safety case — a monitor
    /// narrower than 720px — not because any state asks to grow
    /// horizontally.
    const NATURAL_WIDTH: f64 = 720.0;

    #[test]
    fn desired_smaller_than_available_is_kept_as_is() {
        let result = fit(
            Size {
                width: 720.0,
                height: 480.0,
            },
            WORK_AREA,
        );
        assert_eq!(
            result,
            Size {
                width: 720.0,
                height: 480.0
            }
        );
    }

    #[test]
    fn desired_larger_than_available_is_capped() {
        // 90% of 1080 = 972.
        let result = fit(
            Size {
                width: 720.0,
                height: 2000.0,
            },
            WORK_AREA,
        );
        assert_eq!(result.height, 972.0);
    }

    #[test]
    fn the_floor_is_respected() {
        let result = fit(
            Size {
                width: 100.0,
                height: 100.0,
            },
            WORK_AREA,
        );
        assert_eq!(
            result,
            Size {
                width: MIN_WIDTH,
                height: MIN_HEIGHT
            }
        );
    }

    #[test]
    fn the_ceiling_is_respected() {
        let result = fit(
            Size {
                width: 5000.0,
                height: 5000.0,
            },
            WORK_AREA,
        );
        assert_eq!(
            result,
            Size {
                width: 1920.0 * 0.9,
                height: 1080.0 * 0.9
            }
        );
    }

    #[test]
    fn settings_with_ai_models_expanded_fits_a_typical_laptop_screen() {
        // A realistic tall-content case: header + 4 settings sections + two
        // model cards with technical details open. Comfortably under a
        // 1080p work area's 90% cap (972px), so nothing is capped.
        let desired = Size {
            width: 720.0,
            height: 860.0,
        };
        assert_eq!(fit(desired, WORK_AREA), desired);
    }

    #[test]
    fn success_with_four_files_fits_without_capping() {
        let desired = Size {
            width: 720.0,
            height: 640.0,
        };
        assert_eq!(fit(desired, WORK_AREA), desired);
    }

    #[test]
    fn a_small_screen_never_gets_a_window_bigger_than_itself() {
        // A screen smaller than our own minimum floor: safety over the
        // floor. The window must never exceed what actually exists.
        let tiny_area = Size {
            width: 700.0,
            height: 380.0,
        };
        let result = fit(
            Size {
                width: 720.0,
                height: 900.0,
            },
            tiny_area,
        );
        assert!(result.width <= tiny_area.width);
        assert!(result.height <= tiny_area.height);
    }

    #[test]
    fn nan_or_negative_desired_size_falls_back_to_the_floor_not_to_garbage() {
        for bad in [f64::NAN, f64::NEG_INFINITY, -50.0, 0.0] {
            let result = fit(
                Size {
                    width: bad,
                    height: bad,
                },
                WORK_AREA,
            );
            assert!(result.width.is_finite() && result.width > 0.0);
            assert!(result.height.is_finite() && result.height > 0.0);
            assert_eq!(result.width, MIN_WIDTH);
            assert_eq!(result.height, MIN_HEIGHT);
        }
    }

    #[test]
    fn clamp_dimension_never_panics_when_min_would_exceed_max() {
        // f64::clamp panics if min > max; a screen smaller than our floor is
        // exactly that case, and it must not crash the command.
        let result = clamp_dimension(300.0, 1000.0, 200.0);
        assert_eq!(result, 200.0, "max must win over an unreachable floor");
    }

    #[test]
    fn capped_work_area_never_exceeds_a_fraction_of_one() {
        // A misconfigured fraction above 100% must not hand back more than
        // the whole screen.
        let result = capped_work_area(WORK_AREA, 1.5);
        assert_eq!(result, WORK_AREA);
    }

    #[test]
    fn width_stays_at_its_natural_value_on_an_ordinary_screen() {
        // The product decision: height is content-driven, width stays at the
        // value every screen was visually tuned for.
        let result = fit(
            Size {
                width: NATURAL_WIDTH,
                height: 500.0,
            },
            WORK_AREA,
        );
        assert_eq!(result.width, NATURAL_WIDTH);
    }

    // --- position clamping ---------------------------------------------

    const AREA_ORIGIN: Point = Point { x: 0.0, y: 25.0 }; // macOS menu bar offset
    const AREA_SIZE: Size = Size {
        width: 1920.0,
        height: 1055.0,
    };

    #[test]
    fn a_position_inside_the_area_is_left_alone() {
        let pos = Point { x: 400.0, y: 200.0 };
        let size = Size {
            width: 720.0,
            height: 600.0,
        };
        assert_eq!(clamp_position(pos, size, AREA_ORIGIN, AREA_SIZE), pos);
    }

    #[test]
    fn a_position_past_the_right_edge_is_pulled_back() {
        let size = Size {
            width: 720.0,
            height: 600.0,
        };
        let pos = Point {
            x: 1800.0,
            y: 200.0,
        };
        let clamped = clamp_position(pos, size, AREA_ORIGIN, AREA_SIZE);
        assert_eq!(clamped.x, AREA_SIZE.width - size.width);
        assert!(clamped.x + size.width <= AREA_ORIGIN.x + AREA_SIZE.width);
    }

    #[test]
    fn a_position_above_the_menu_bar_is_pulled_down() {
        let size = Size {
            width: 720.0,
            height: 600.0,
        };
        let pos = Point { x: 400.0, y: 0.0 };
        let clamped = clamp_position(pos, size, AREA_ORIGIN, AREA_SIZE);
        assert_eq!(clamped.y, AREA_ORIGIN.y);
    }

    #[test]
    fn an_oversized_window_is_pinned_to_the_area_origin_rather_than_left_partly_off_screen() {
        let size = Size {
            width: 2200.0,
            height: 1200.0,
        }; // bigger than AREA_SIZE
        let pos = Point { x: 400.0, y: 200.0 };
        let clamped = clamp_position(pos, size, AREA_ORIGIN, AREA_SIZE);
        assert_eq!(clamped, AREA_ORIGIN);
    }
}
