#![forbid(unsafe_code)]

//! Depth measurement and pressure modeling for nested ternary systems.
//!
//! Models how constructs at different layers of abstraction interact, measuring
//! computational pressure, enabling safe transitions between depth levels, and
//! characterizing the deepest (abyssal) zones where normal rules break down.
//! Maps to construct-core's L0→L1→L2 layer transitions.

use std::collections::HashMap;

/// Ternary value: -1, 0, or +1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// Represents depth in a nested hierarchy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Depth {
    pub level: u32,
    pub max_level: u32,
}

impl Depth {
    pub fn new(level: u32, max_level: u32) -> Self {
        Depth {
            level: level.min(max_level),
            max_level,
        }
    }

    pub fn surface() -> Self {
        Depth { level: 0, max_level: u32::MAX }
    }

    pub fn is_surface(&self) -> bool {
        self.level == 0
    }

    pub fn is_max(&self) -> bool {
        self.level == self.max_level
    }

    /// Descend one level. Returns None if already at max.
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

    /// Ascend one level. Returns None if already at surface.
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

    /// Fraction of max depth reached (0.0 at surface, 1.0 at max).
    pub fn fraction(&self) -> f64 {
        if self.max_level == 0 {
            return 1.0;
        }
        self.level as f64 / self.max_level as f64
    }
}

/// Pressure reading with ternary classification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressureReading {
    pub depth: Depth,
    pub raw_pressure: f64,
    pub classification: Ternary,
}

/// Measures computational pressure at various depths.
#[derive(Debug)]
pub struct PressureGauge {
    pub baseline: f64,
    pub sensitivity: f64,
}

impl PressureGauge {
    pub fn new(baseline: f64, sensitivity: f64) -> Self {
        PressureGauge { baseline, sensitivity: sensitivity.max(0.001) }
    }

    /// Measure pressure at a given depth.
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

    /// Check if pressure exceeds a safe threshold.
    pub fn is_safe(&self, reading: &PressureReading, max_safe: f64) -> bool {
        reading.raw_pressure <= max_safe
    }
}

impl Default for PressureGauge {
    fn default() -> Self {
        PressureGauge::new(1.0, 1.0)
    }
}

/// Observes deep layers without direct interaction.
#[derive(Debug)]
pub struct Bathyscope {
    pub max_depth: u32,
    observations: HashMap<u32, Vec<String>>,
}

impl Bathyscope {
    pub fn new(max_depth: u32) -> Self {
        Bathyscope {
            max_depth,
            observations: HashMap::new(),
        }
    }

    /// Record an observation at a specific depth.
    pub fn observe(&mut self, depth: u32, note: &str) -> bool {
        if depth > self.max_depth {
            return false;
        }
        self.observations.entry(depth).or_default().push(note.to_string());
        true
    }

    /// Retrieve all observations at a given depth.
    pub fn at_depth(&self, depth: u32) -> &[String] {
        self.observations.get(&depth).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Total observations across all depths.
    pub fn total_observations(&self) -> usize {
        self.observations.values().map(|v| v.len()).sum()
    }

    /// The deepest level with observations.
    pub fn deepest_observed(&self) -> Option<u32> {
        self.observations.keys().max().copied()
    }
}

/// A targeted perturbation applied at a specific depth.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthCharge {
    pub target_depth: u32,
    pub intensity: f64,
    pub effect: Ternary,
}

impl DepthCharge {
    pub fn new(target_depth: u32, intensity: f64, effect: Ternary) -> Self {
        DepthCharge {
            target_depth,
            intensity: intensity.clamp(0.0, 1.0),
            effect,
        }
    }

    /// Compute the effect at a given depth, attenuated by distance.
    pub fn effect_at(&self, depth: u32) -> f64 {
        let distance = (self.target_depth as i64 - depth as i64).unsigned_abs();
        if distance == 0 {
            self.intensity
        } else {
            self.intensity / (1.0 + distance as f64)
        }
    }

    /// Whether the charge affects a given depth above a threshold.
    pub fn affects(&self, depth: u32, threshold: f64) -> bool {
        self.effect_at(depth) >= threshold
    }
}

/// Characterizes the deepest layers where normal operations may not hold.
#[derive(Debug)]
pub struct AbyssalZone {
    pub start_depth: u32,
    pub max_depth: u32,
    pub anomalies: Vec<String>,
}

impl AbyssalZone {
    pub fn new(start_depth: u32, max_depth: u32) -> Self {
        AbyssalZone {
            start_depth,
            max_depth,
            anomalies: Vec::new(),
        }
    }

    pub fn is_abyssal(&self, depth: u32) -> bool {
        depth >= self.start_depth && depth <= self.max_depth
    }

    pub fn record_anomaly(&mut self, anomaly: &str) {
        self.anomalies.push(anomaly.to_string());
    }

    pub fn anomaly_count(&self) -> usize {
        self.anomalies.len()
    }

    /// Safety factor: 1.0 at start of abyssal zone, decreasing toward 0 at max.
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

/// Safe decompression for transitioning between depth levels.
#[derive(Debug)]
pub struct PressureDecompression {
    pub steps: Vec<Depth>,
}

impl PressureDecompression {
    /// Plan a decompression from deep to surface with intermediate stops.
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
        PressureDecompression { steps }
    }

    /// Plan decompression with a stop at every N levels.
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
        PressureDecompression { steps }
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn total_levels(&self) -> u32 {
        self.steps.last().map(|s| s.level).unwrap_or(0)
    }

    /// Check if a given depth is a decompression stop.
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
        let gauge = PressureGauge::new(1.0, 2.0);
        let d = Depth::new(10, 10);
        let reading = gauge.measure(d, 2.0);
        assert!(reading.raw_pressure > 1.5);
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
        let d = Depth::new(10, 10);
        let plan = PressureDecompression::plan(d);
        assert!(plan.step_count() > 0);
        assert!(plan.steps.last().unwrap().is_surface());
    }

    #[test]
    fn test_decompression_plan_interval() {
        let d = Depth::new(10, 10);
        let plan = PressureDecompression::plan_with_interval(d, 2);
        assert!(plan.step_count() > 0);
    }

    #[test]
    fn test_decompression_is_stop() {
        let d = Depth::new(10, 10);
        let plan = PressureDecompression::plan_with_interval(d, 5);
        let stop = Depth::new(5, 10);
        assert!(plan.is_stop(&stop));
    }
}
