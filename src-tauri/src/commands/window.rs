use crate::domain::window_fit::{self, Point, Size};
use tauri::{LogicalPosition, LogicalSize, Window};

/// Resizes the calling window to fit `width`/`height` of real content,
/// bounded by its current monitor's usable area (see ADR-011 and
/// `domain::window_fit`).
///
/// Called from the frontend every time its measured content size changes by
/// more than a small threshold — see `src/features/startup/useFitWindow.ts`,
/// which is also what keeps this from being spammed on every animation
/// frame. This command does no debouncing of its own; it trusts its caller
/// and only applies the sizing policy.
#[tauri::command]
pub fn fit_window(window: Window, width: f64, height: f64) -> Result<(), String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or(window.primary_monitor().map_err(|e| e.to_string())?);

    // No monitor info at all is a real possibility in a headless/CI
    // environment. Resizing to the floor-clamped request rather than failing
    // outright means the window still ends up in a sane state.
    let Some(monitor) = monitor else {
        let fallback = Size {
            width: width.max(window_fit::MIN_WIDTH),
            height: height.max(window_fit::MIN_HEIGHT),
        };
        return window
            .set_size(LogicalSize::new(fallback.width, fallback.height))
            .map_err(|e| e.to_string());
    };

    let scale = monitor.scale_factor();
    let work_area = monitor.work_area();
    let area_size = Size {
        width: work_area.size.width as f64 / scale,
        height: work_area.size.height as f64 / scale,
    };
    let area_origin = Point {
        x: work_area.position.x as f64 / scale,
        y: work_area.position.y as f64 / scale,
    };

    let target = window_fit::fit(Size { width, height }, area_size);
    window
        .set_size(LogicalSize::new(target.width, target.height))
        .map_err(|e| e.to_string())?;

    // Only reposition if the resize would otherwise leave part of the window
    // outside the monitor's usable area (e.g. it grew toward an edge, or the
    // window was moved to a smaller external display since it was last
    // placed). A window that already fits is left exactly where the user put
    // it — this is a safety net, not a recentring on every content change.
    if let Ok(outer) = window.outer_position() {
        let current = Point {
            x: outer.x as f64 / scale,
            y: outer.y as f64 / scale,
        };
        let clamped = window_fit::clamp_position(current, target, area_origin, area_size);
        if clamped != current {
            let _ = window.set_position(LogicalPosition::new(clamped.x, clamped.y));
        }
    }

    Ok(())
}
