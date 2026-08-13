//! Pins the boot splash's timeline and containment.
//!
//! The intro is user-validated and deliberately frozen: six seconds, the
//! same duration whether or not motion is reduced, rendered as a layer over
//! the application rather than in a window of its own. All of that lives in
//! one stylesheet, where it is easy to "just shorten" or "just tidy" without
//! realising which parts were decisions.
//!
//! File-content assertions are the established pattern here (see
//! `csp_policy.rs`, which reads this same stylesheet).

use std::fs;

fn css() -> String {
    fs::read_to_string("../src/features/boot/bootSplash.css").expect("read bootSplash.css")
}

#[test]
fn the_cycle_runs_for_six_seconds() {
    assert!(
        css().contains("--cycle-duration: 6000ms;"),
        "the validated intro is six seconds long"
    );
}

#[test]
fn the_cycle_fades_in_over_one_second_and_out_over_two() {
    // 1s and 4s of 6s, as keyframe percentages.
    let css = css();
    assert!(css.contains("16.667%"), "fade-in ends at 1s");
    assert!(css.contains("66.667%"), "fade-out starts at 4s");
}

#[test]
fn the_layer_ends_fully_transparent() {
    // The interface is already mounted and laid out behind it, so the end of
    // the intro is a transparent layer being removed — never a reveal, and
    // never a swap between two windows.
    let css = css();
    let cycle = &css[css
        .find("@keyframes boot-splash-cycle")
        .expect("cycle keyframes")..];
    let body = &cycle[..cycle.find("}\n}").expect("end of keyframes")];
    let last = &body[body.rfind("100%").expect("a 100% keyframe")..];
    assert!(
        last.contains("opacity: 0"),
        "the layer must fade to nothing"
    );
}

#[test]
fn reduced_motion_keeps_the_same_duration() {
    // Reduced motion asks for less movement, not for a hurried product. The
    // duration is declared once and never overridden, which is what makes
    // the two variants structurally identical.
    assert_eq!(
        css().matches("--cycle-duration:").count(),
        1,
        "the cycle duration must not be redefined for reduced motion"
    );
}

#[test]
fn reduced_motion_disables_every_decorative_animation() {
    let css = css();
    for selector in [".wave__bar", ".lines__line", ".boot-splash__wordmark"] {
        let needle = format!("[data-motion=\"reduce\"] {selector}");
        let rule = &css[css
            .find(&needle)
            .unwrap_or_else(|| panic!("no reduced-motion rule for {selector}"))..];
        let body = &rule[..rule.find('}').expect("rule body")];
        assert!(
            body.contains("animation: none"),
            "{selector} must not animate under reduced motion"
        );
    }
}

#[test]
fn the_layer_covers_the_webview_and_nothing_more() {
    // On macOS the webview is exactly the content area: the title bar and
    // its traffic lights are native chrome outside it, and stay visible for
    // the whole intro. Covering the webview is therefore correct by
    // construction — but only if the layer is positioned, not full-screen.
    let css = css();
    let layer = &css[css.find(".boot-splash {").expect(".boot-splash rule")..];
    let body = &layer[..layer.find('}').expect("rule body")];
    assert!(body.contains("position: absolute"));
    assert!(body.contains("inset: 0"));
    assert!(
        body.contains("background: var(--bg)"),
        "the layer must be opaque, or the app is visible behind it"
    );
}

#[test]
fn the_layer_uses_the_application_theme_tokens() {
    // It shares the document with the app now, so a theme change while the
    // intro is on screen applies to it too — no separate palette to drift.
    let css = css();
    assert!(css.contains("var(--bg)"));
    assert!(css.contains("var(--fg)"));
    assert!(
        !css.contains("--splash-bg"),
        "the standalone splash palette should be gone"
    );
}
