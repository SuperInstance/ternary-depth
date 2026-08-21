#![forbid(unsafe_code)]

//! Depth measurement and pressure modeling for nested ternary systems.
//!
//! Models how constructs at different layers of abstraction interact, measuring
//! computational pressure, enabling safe transitions between depth levels, and
//! characterizing the deepest (abyssal) zones where normal rules break down.
//! Maps to construct-core's L0→L1→L2 layer transitions.

use std::collections::HashMap;

/// A three-valued sign used to classify pressure readings.
///
/// `Neg`/`Pos`/`Zero` map to under-pressure / over-pressure / nominal
/// respectively (see [`PressureGauge::measure`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ternary {
    /// Under-pressure (below the nominal band).
    Neg = -1,
    /// Nominal pressure (within the band).
    Zero = 0,
    /// Over-pressure (above the nominal band).
    Pos = 1,
}

impl Ternary {
    /// Convert an `i8` into a [`Ternary`]. Returns `None` for anything other
    /// than `-1`, `0`, or `1`.
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    /// Convert this [`Ternary`] back into its `i8` representation (`-1`, `0`,
    /// or `1`).
    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// Represents a position in a nested hierarchy as a `level`/`max_level` pair.
///
/// `level` is always `<= max_level`; [`Depth::new`] clamps an out-of-range
/// level down to `max_level` rather than panicking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Depth {
    /// Current level (0 == surface).
    pub level: u32,
    /// Maximum reachable level.
    pub max_level: u32,
}

impl Depth {
    /// Create a depth, clamping `level` down to `max_level` if it exceeds it.
    pub fn new(level: u32, max_level: u32) -> Self {
        Depth {
            level: level.min(max_level),
            max_level,
        }
    }

    /// The surface of an (effectively) unbounded hierarchy: level 0 with the
    /// largest representable `max_level`. Useful when no explicit bound exists.
    pub fn surface() -> Self {
        Depth {
            level: 0,
            max_level: u32::MAX,
        }
    }

    /// True when at level 0.
    pub fn is_surface(&self) -> bool {
        self.level == 0
    }

    /// True when at `max_level` (cannot descend further).
    pub fn is_max(&self) -> bool {
        self.level == self.max_level
    }

    /// Descend one level. Returns `None` if already at `max_level`.
    pub fn descend(&self) -> Option<Depth> {
        if self.level < self.max_level {
            Some(Depth {
                level: self.level + 1,
                max_level: self.max_level,
            })
        } else {
            None
        }
    }

    /// Ascend one level. Returns `None` if already at the surface.
    pub fn ascend(&self) -> Option<Depth> {
        if self.level > 0 {
            Some(Depth {
                level: self.level - 1,
                max_level: self.max_level,
            })
        } else {
            None
        }
    }

    /// Fraction of `max_level` reached: `0.0` at the surface and `1.0` at
    /// `max_level`. A degenerate depth with `max_level == 0` is defined to have
    /// fraction `1.0`.
    pub fn fraction(&self) -> f64 {
        if self.max_level == 0 {
            return 1.0;
        }
        self.level as f64 / self.max_level as f64
    }
}

/// A pressure measurement: the raw value and its [`Ternary`] classification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressureReading {
    /// The depth at which the reading was taken.
    pub depth: Depth,
    /// The computed pressure `P = baseline + load * fraction * sensitivity`.
    pub raw_pressure: f64,
    /// Whether `raw_pressure` is above (`Pos`), below (`Neg`) or within
    /// (`Zero`) the nominal band around the baseline.
    pub classification: Ternary,
}

/// Measures computational pressure as a function of [`Depth`] and load.
///
/// The model is `P(d, L) = baseline + L * fraction(d) * sensitivity`, where
/// `fraction(d)` is [`Depth::fraction`]. A reading is classified as
/// over-pressure when `P > 1.5 * baseline`, under-pressure when
/// `P < 0.5 * baseline`, and nominal otherwise.
#[derive(Debug)]
pub struct PressureGauge {
    /// Baseline (surface) pressure `P0`.
    pub baseline: f64,
    /// Load-to-pressure gain `S`. Construction clamps this to a small positive
    /// floor (`0.001`) so a degenerate zero sensitivity cannot silently erase
    /// all depth-dependent load.
    pub sensitivity: f64,
}

impl PressureGauge {
    /// Construct a gauge. `sensitivity` is clamped to at least `0.001` to keep
    /// the depth-dependent term non-degenerate.
    pub fn new(baseline: f64, sensitivity: f64) -> Self {
        PressureGauge {
            baseline,
            sensitivity: sensitivity.max(0.001),
        }
    }

    /// Measure pressure at `depth` under `load`, returning the raw value and
    /// its [`Ternary`] classification.
    pub fn measure(&self, depth: Depth, load: f64) -> PressureReading {
        let raw = self.baseline + load * depth.fraction() * self.sensitivity;
        let classification = if raw > self.baseline * 1.5 {
            Ternary::Pos
        } else if raw < self.baseline * 0.5 {
            Ternary::Neg
        } else {
            Ternary::Zero
        };
        PressureReading {
            depth,
            raw_pressure: raw,
            classification,
        }
    }

    /// True when `reading.raw_pressure` is at or below `max_safe`.
    pub fn is_safe(&self, reading: &PressureReading, max_safe: f64) -> bool {
        reading.raw_pressure <= max_safe
    }
}

impl Default for PressureGauge {
    /// A gauge with baseline `1.0` and sensitivity `1.0`.
    fn default() -> Self {
        PressureGauge::new(1.0, 1.0)
    }
}

/// A non-invasive log of observations keyed by depth, like a bathyscope
/// lowered into deep layers without disturbing them.
#[derive(Debug)]
pub struct Bathyscope {
    /// Deepest level this scope will accept observations for.
    pub max_depth: u32,
    observations: HashMap<u32, Vec<String>>,
}

impl Bathyscope {
    /// Create an empty scope accepting observations up to `max_depth`.
    pub fn new(max_depth: u32) -> Self {
        Bathyscope {
            max_depth,
            observations: HashMap::new(),
        }
    }

    /// Record an observation at `depth`. Returns `false` (and stores nothing)
    /// if `depth` exceeds [`max_depth`](Self::max_depth).
    pub fn observe(&mut self, depth: u32, note: &str) -> bool {
        if depth > self.max_depth {
            return false;
        }
        self.observations
            .entry(depth)
            .or_default()
            .push(note.to_string());
        true
    }

    /// All observation notes recorded at `depth` (empty slice if none).
    pub fn at_depth(&self, depth: u32) -> &[String] {
        self.observations
            .get(&depth)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Total number of observations across every depth.
    pub fn total_observations(&self) -> usize {
        self.observations.values().map(|v| v.len()).sum()
    }

    /// The deepest level that has at least one observation, or `None` if empty.
    pub fn deepest_observed(&self) -> Option<u32> {
        self.observations.keys().max().copied()
    }
}

/// A targeted perturbation delivered at a specific depth, attenuating with
/// distance as `intensity / (1 + |depth - target_depth|)`.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthCharge {
    /// Depth at which the charge is centered (full intensity).
    pub target_depth: u32,
    /// Charge strength, clamped to `[0, 1]` on construction.
    pub intensity: f64,
    /// Sign of the perturbation applied by this charge.
    pub effect: Ternary,
}

impl DepthCharge {
    /// Create a charge centered at `target_depth`. `intensity` is clamped to
    /// `[0, 1]`.
    pub fn new(target_depth: u32, intensity: f64, effect: Ternary) -> Self {
        DepthCharge {
            target_depth,
            intensity: intensity.clamp(0.0, 1.0),
            effect,
        }
    }

    /// Compute the attenuated effect at `depth`: `intensity` at the target,
    /// falling off as `intensity / (1 + distance)` away from it.
    pub fn effect_at(&self, depth: u32) -> f64 {
        let distance = (self.target_depth as i64 - depth as i64).unsigned_abs();
        if distance == 0 {
            self.intensity
        } else {
            self.intensity / (1.0 + distance as f64)
        }
    }

    /// True when [`effect_at`](Self::effect_at)`(depth)` reaches `threshold`.
    pub fn affects(&self, depth: u32, threshold: f64) -> bool {
        self.effect_at(depth) >= threshold
    }
}

/// Characterizes the deepest layers (`[start_depth, max_depth]`) where normal
/// invariants may fail, accumulating a log of anomalies that occur there.
#[derive(Debug)]
pub struct AbyssalZone {
    /// Shallowest depth considered abyssal.
    pub start_depth: u32,
    /// Deepest depth considered abyssal.
    pub max_depth: u32,
    /// Recorded anomaly descriptions.
    pub anomalies: Vec<String>,
}

impl AbyssalZone {
    /// Define an abyssal zone spanning `[start_depth, max_depth]`.
    pub fn new(start_depth: u32, max_depth: u32) -> Self {
        AbyssalZone {
            start_depth,
            max_depth,
            anomalies: Vec::new(),
        }
    }

    /// True when `depth` lies within `[start_depth, max_depth]`.
    pub fn is_abyssal(&self, depth: u32) -> bool {
        depth >= self.start_depth && depth <= self.max_depth
    }

    /// Append an anomaly description to the log.
    pub fn record_anomaly(&mut self, anomaly: &str) {
        self.anomalies.push(anomaly.to_string());
    }

    /// Number of anomalies recorded so far.
    pub fn anomaly_count(&self) -> usize {
        self.anomalies.len()
    }

    /// Safety factor: `1.0` at `start_depth`, decreasing linearly to `0.0` at
    /// `max_depth`. Depths outside the abyssal zone report `1.0`. A degenerate
    /// zero-width zone reports `0.0` at its single depth.
    pub fn safety_factor(&self, depth: u32) -> f64 {
        if !self.is_abyssal(depth) {
            return 1.0;
        }
        let range = (self.max_depth - self.start_depth) as f64;
        if range == 0.0 {
            return 0.0;
        }
        let progress = (depth - self.start_depth) as f64 / range;
        1.0 - progress
    }
}

/// A staged decompression schedule from a deep [`Depth`] back to the surface,
/// pausing at intermediate stops to let pressure equalize.
#[derive(Debug)]
pub struct PressureDecompression {
    /// The depth from which decompression begins.
    pub from: Depth,
    /// Ordered (deepest-first) list of intermediate stops, always terminating at
    /// the surface.
    pub steps: Vec<Depth>,
}

impl PressureDecompression {
    /// Plan a decompression from `from` back to the surface, stopping every 3rd
    /// level (and always at the surface).
    pub fn plan(from: Depth) -> Self {
        let mut steps = Vec::new();
        let mut current = from;
        while let Some(above) = current.ascend() {
            // Stop every 3 levels, or at surface
            if above.level % 3 == 0 || above.is_surface() {
                steps.push(above);
            }
            current = above;
        }
        PressureDecompression { from, steps }
    }

    /// Plan decompression with a stop at every `interval` levels (clamped to a
    /// minimum of 1), plus a final stop at the surface.
    pub fn plan_with_interval(from: Depth, interval: u32) -> Self {
        let interval = interval.max(1);
        let mut steps = Vec::new();
        let mut current = from;
        while let Some(above) = current.ascend() {
            if above.level % interval == 0 || above.is_surface() {
                steps.push(above);
            }
            current = above;
        }
        PressureDecompression { from, steps }
    }

    /// Number of intermediate stops in the schedule.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Total number of depth levels traversed by the plan — i.e. the starting
    /// depth, since every plan terminates at the surface (level 0).
    pub fn total_levels(&self) -> u32 {
        self.from.level
    }

    /// Check if a given depth's level is one of the scheduled stops. Only the
    /// `level` is compared; `max_level` is ignored.
    pub fn is_stop(&self, depth: &Depth) -> bool {
        self.steps.iter().any(|s| s.level == depth.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_new_clamps() {
        let d = Depth::new(100, 10);
        assert_eq!(d.level, 10);
        assert_eq!(d.max_level, 10);
    }

    #[test]
    fn test_ternary_round_trip() {
        for v in [-1_i8, 0, 1] {
            let t = Ternary::from_i8(v).unwrap();
            assert_eq!(t.to_i8(), v);
        }
        assert!(Ternary::from_i8(2).is_none());
        assert!(Ternary::from_i8(-2).is_none());
        assert_eq!(Ternary::Neg.to_i8(), -1);
        assert_eq!(Ternary::Pos.to_i8(), 1);
    }

    #[test]
    fn test_depth_fraction_boundaries() {
        assert_eq!(Depth::new(0, 10).fraction(), 0.0);
        assert_eq!(Depth::new(10, 10).fraction(), 1.0);
        // Degenerate zero max_level is defined as fully deep.
        assert_eq!(Depth::new(0, 0).fraction(), 1.0);
    }

    #[test]
    fn test_depth_descend_at_surface() {
        let s = Depth::surface();
        let d = s.descend().unwrap();
        assert_eq!(d.level, 1);
        assert!(!d.is_surface());
    }

    #[test]
    fn test_depth_descend() {
        let d = Depth::new(5, 10);
        assert_eq!(d.descend().unwrap().level, 6);
    }

    #[test]
    fn test_depth_descend_at_max() {
        let d = Depth::new(10, 10);
        assert!(d.descend().is_none());
    }

    #[test]
    fn test_depth_ascend() {
        let d = Depth::new(5, 10);
        assert_eq!(d.ascend().unwrap().level, 4);
    }

    #[test]
    fn test_depth_ascend_at_surface() {
        let d = Depth::surface();
        assert!(d.ascend().is_none());
    }

    #[test]
    fn test_depth_fraction() {
        let d = Depth::new(5, 10);
        assert!((d.fraction() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_depth_is_surface_and_max() {
        let d = Depth::new(0, 10);
        assert!(d.is_surface());
        assert!(!d.is_max());
        let d2 = Depth::new(10, 10);
        assert!(!d2.is_surface());
        assert!(d2.is_max());
    }

    #[test]
    fn test_pressure_gauge_measure_normal() {
        let gauge = PressureGauge::new(1.0, 1.0);
        let d = Depth::new(5, 10);
        let reading = gauge.measure(d, 1.0);
        assert!((reading.raw_pressure - 1.5).abs() < 0.01);
        assert_eq!(reading.classification, Ternary::Zero);
    }

    #[test]
    fn test_pressure_gauge_measure_high() {
        // baseline=1, sensitivity=2, fraction(10/10)=1.0, load=2
        // => raw = 1 + 2 * 1.0 * 2 = 5.0, classified Pos (> 1.5*1 = 1.5)
        let gauge = PressureGauge::new(1.0, 2.0);
        let d = Depth::new(10, 10);
        let reading = gauge.measure(d, 2.0);
        assert!((reading.raw_pressure - 5.0).abs() < 1e-12);
        assert_eq!(reading.classification, Ternary::Pos);
    }

    #[test]
    fn test_pressure_gauge_measure_under() {
        // negative load pushes below baseline*0.5 => Neg
        // raw = 1.0 + (-4.0) * 1.0 * 1.0 = -3.0 < 0.5
        let gauge = PressureGauge::default();
        let reading = gauge.measure(Depth::new(10, 10), -4.0);
        assert!((reading.raw_pressure - (-3.0)).abs() < 1e-12);
        assert_eq!(reading.classification, Ternary::Neg);
    }

    #[test]
    fn test_pressure_gauge_is_safe() {
        let gauge = PressureGauge::new(1.0, 1.0);
        let reading = gauge.measure(Depth::new(0, 10), 0.5);
        assert!(gauge.is_safe(&reading, 10.0));
    }

    #[test]
    fn test_bathyscope_observe() {
        let mut b = Bathyscope::new(10);
        assert!(b.observe(5, "layer 5 structure"));
        assert_eq!(b.at_depth(5).len(), 1);
        // Unobserved depth returns an empty slice (no panic).
        assert!(b.at_depth(7).is_empty());
        assert!(b.at_depth(10).is_empty());
    }

    #[test]
    fn test_bathyscope_reject_too_deep() {
        let mut b = Bathyscope::new(5);
        assert!(!b.observe(10, "too deep"));
    }

    #[test]
    fn test_bathyscope_total_and_deepest() {
        let mut b = Bathyscope::new(10);
        b.observe(3, "a");
        b.observe(7, "b");
        b.observe(7, "c");
        assert_eq!(b.total_observations(), 3);
        assert_eq!(b.deepest_observed(), Some(7));
    }

    #[test]
    fn test_depth_charge_effect_at_target() {
        let dc = DepthCharge::new(5, 0.8, Ternary::Pos);
        assert!((dc.effect_at(5) - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_depth_charge_effect_attenuates() {
        let dc = DepthCharge::new(5, 1.0, Ternary::Neg);
        let at_0 = dc.effect_at(0);
        let at_5 = dc.effect_at(5);
        assert!(at_0 < at_5);
        // Exact attenuation: distance 5 => 1.0 / (1 + 5) = 1/6.
        assert!((dc.effect_at(0) - 1.0 / 6.0).abs() < 1e-12);
        // Symmetric in distance.
        assert!((dc.effect_at(10) - 1.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_depth_charge_affects() {
        let dc = DepthCharge::new(5, 1.0, Ternary::Pos);
        assert!(dc.affects(5, 0.5));
        assert!(!dc.affects(100, 0.5));
    }

    #[test]
    fn test_depth_charge_intensity_clamped() {
        let dc = DepthCharge::new(0, 5.0, Ternary::Zero);
        assert!((dc.intensity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_abyssal_zone_is_abyssal() {
        let az = AbyssalZone::new(8, 12);
        assert!(az.is_abyssal(8));
        assert!(az.is_abyssal(10));
        assert!(az.is_abyssal(12));
        assert!(!az.is_abyssal(7));
        assert!(!az.is_abyssal(13));
    }

    #[test]
    fn test_abyssal_zone_anomalies() {
        let mut az = AbyssalZone::new(8, 12);
        az.record_anomaly("null pointer at depth 9");
        az.record_anomaly("infinite loop at depth 11");
        assert_eq!(az.anomaly_count(), 2);
    }

    #[test]
    fn test_abyssal_zone_safety_factor() {
        let az = AbyssalZone::new(8, 12);
        assert!((az.safety_factor(8) - 1.0).abs() < f64::EPSILON);
        assert!((az.safety_factor(10) - 0.5).abs() < 0.01);
        assert!((az.safety_factor(12) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_abyssal_zone_safety_above_zone() {
        let az = AbyssalZone::new(8, 12);
        assert!((az.safety_factor(5) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decompression_plan() {
        // From level 10, ascending one level at a time, pushing whenever the
        // level is a multiple of 3 (or the surface). Stops: 9, 6, 3, 0.
        let d = Depth::new(10, 10);
        let plan = PressureDecompression::plan(d);
        let levels: Vec<u32> = plan.steps.iter().map(|s| s.level).collect();
        assert_eq!(levels, vec![9, 6, 3, 0]);
        assert_eq!(plan.step_count(), 4);
        assert!(plan.steps.last().unwrap().is_surface());
        assert_eq!(plan.total_levels(), 10);
        assert_eq!(plan.from, d);
    }

    #[test]
    fn test_decompression_plan_from_surface() {
        // Already at surface: no ascents possible, no stops.
        let plan = PressureDecompression::plan(Depth::new(0, 10));
        assert_eq!(plan.step_count(), 0);
        assert!(plan.steps.is_empty());
        assert_eq!(plan.total_levels(), 0);
    }

    #[test]
    fn test_decompression_plan_interval() {
        // interval=2 from level 10 => stops at 8,6,4,2,0.
        let d = Depth::new(10, 10);
        let plan = PressureDecompression::plan_with_interval(d, 2);
        let levels: Vec<u32> = plan.steps.iter().map(|s| s.level).collect();
        assert_eq!(levels, vec![8, 6, 4, 2, 0]);
        assert_eq!(plan.step_count(), 5);
        assert_eq!(plan.total_levels(), 10);
    }

    #[test]
    fn test_decompression_plan_interval_clamps_zero() {
        // interval=0 is clamped to 1 => stop at every level.
        let plan = PressureDecompression::plan_with_interval(Depth::new(3, 3), 0);
        let levels: Vec<u32> = plan.steps.iter().map(|s| s.level).collect();
        assert_eq!(levels, vec![2, 1, 0]);
    }

    #[test]
    fn test_decompression_is_stop() {
        let d = Depth::new(10, 10);
        let plan = PressureDecompression::plan_with_interval(d, 5);
        let stop = Depth::new(5, 10);
        assert!(plan.is_stop(&stop));
        // 4 is not a multiple of 5 and not the surface.
        assert!(!plan.is_stop(&Depth::new(4, 10)));
        // The surface is always a stop.
        assert!(plan.is_stop(&Depth::new(0, 10)));
    }
}
