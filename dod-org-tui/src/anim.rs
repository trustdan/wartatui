//! Animation clock and easing helpers.
//!
//! Everything cinematic reads from a single wall-clock elapsed time so motion
//! stays consistent regardless of frame rate.

use std::time::Instant;

/// How long the boot cascade runs, in seconds.
pub const BOOT_SECS: f32 = 1.6;
/// How long the classification banner flickers before going solid.
pub const BANNER_FLICKER_SECS: f32 = 0.45;

/// Monotonic animation clock started at app launch.
pub struct Clock {
    start: Instant,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Seconds elapsed since launch.
    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }
}

pub fn clamp01(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// Smooth deceleration — fast start, gentle landing.
pub fn ease_out_cubic(t: f32) -> f32 {
    let p = 1.0 - clamp01(t);
    1.0 - p * p * p
}

/// Even-paced ease in/out — visible motion across the whole span.
pub fn smoothstep(t: f32) -> f32 {
    let t = clamp01(t);
    t * t * (3.0 - 2.0 * t)
}

/// A 0..1 triangle/sine pulse for breathing effects.
pub fn pulse(elapsed: f32, speed: f32) -> f32 {
    (elapsed * speed).sin() * 0.5 + 0.5
}

/// Linear interpolation.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * clamp01(t)
}
