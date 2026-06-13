# Ternary Depth

Depth measurement and **pressure modeling for nested ternary systems** — measuring computational pressure at various abstraction layers, enabling safe transitions between depth levels, and characterizing the deepest zones (abyssal) where normal rules may not hold.

## Why It Matters

Nested systems — whether call stacks, nested ternary agent hierarchies, or layered abstraction architectures — exhibit **pressure gradients**. Deeper layers bear more load: more context, more accumulated state, more interaction complexity. Without a formal model of depth and pressure, two failure modes emerge:

1. **Decompression sickness**: Jumping from deep to surface too quickly causes state corruption (the software equivalent of nitrogen bubbles in blood)
2. **Abyssal anomalies**: At extreme depths, normal assumptions break — invariants may not hold, recursion limits are hit, stack overflows occur

This crate provides the mathematical framework to reason about these phenomena. The `Depth` type tracks position in a nested hierarchy, `PressureGauge` measures computational load, and `PressureDecompression` plans safe multi-step transitions.

## How It Works

### Depth Model

A `Depth` is a level/max_level pair. Operations form a monoid:

$$\text{descend}(d) = \begin{cases} d + 1 & d < d_{\max} \\ \text{None} & \text{otherwise} \end{cases}$$

$$\text{ascend}(d) = \begin{cases} d - 1 & d > 0 \\ \text{None} & \text{otherwise} \end{cases}$$

Depth fraction: $\phi(d) = d / d_{\max}$, with $\phi \in [0, 1]$.

### Pressure Model

The pressure at depth $d$ with load $L$ is:

$$P(d, L) = P_0 + L \cdot \phi(d) \cdot S$$

where $P_0$ is baseline pressure and $S$ is sensitivity. The ternary classification:

$$\text{class}(P) = \begin{cases} +1 & P > 1.5 \cdot P_0 \;\text{(over-pressure)} \\ -1 & P < 0.5 \cdot P_0 \;\text{(under-pressure)} \\ 0 & \text{otherwise (nominal)} \end{cases}$$

### Depth Charge (Perturbation)

A depth charge delivers a targeted perturbation at a specific depth, attenuating with distance:

$$E(d) = \frac{I}{1 + |d - d_{\text{target}}|}$$

where $I \in [0, 1]$ is the charge intensity. The $1/(1+d)$ attenuation ensures perturbations are local — a charge at depth 5 has negligible effect at depth 0.

### Decompression Planning

Safe ascent requires **staged decompression** — pausing at intermediate depths to allow state to stabilize. The default plan stops every 3 levels:

$$\text{stops} = \{d : d \bmod 3 = 0 \;\text{or} \; d = 0\}$$

This mirrors diving decompression tables, where ascending too fast causes dissolved gases to form bubbles.

### Abyssal Zone

The abyssal zone is $[d_{\text{start}}, d_{\max}]$ where anomalies are recorded and safety degrades linearly:

$$S_{\text{abyssal}}(d) = 1 - \frac{d - d_{\text{start}}}{d_{\max} - d_{\text{start}}}$$

At $d_{\text{start}}$, safety is 1.0. At $d_{\max}$, safety is 0.0.

### Complexity

| Operation | Time |
|-----------|------|
| `Depth::descend() / ascend()` | O(1) |
| `PressureGauge::measure(depth, load)` | O(1) |
| `DepthCharge::effect_at(depth)` | O(1) |
| `PressureDecompression::plan(from)` | O(D) — D = depth levels |
| `AbyssalZone::safety_factor(depth)` | O(1) |
| `Bathyscope::observe(depth, note)` | O(1) amortized |

## Quick Start

```rust
use ternary_depth::*;

// Depth navigation
let surface = Depth::surface();
let deep = Depth::new(10, 10);
assert!((deep.fraction() - 1.0).abs() < 0.01);

// Pressure measurement
let gauge = PressureGauge::new(1.0, 2.0);
let reading = gauge.measure(Depth::new(5, 10), 1.5);
assert!(reading.raw_pressure > 1.0);

// Decompression plan
let plan = PressureDecompression::plan(Depth::new(10, 10));
// Stops at levels 9, 6, 3, 0 (every 3rd level)

// Depth charges — targeted perturbations
let charge = DepthCharge::new(5, 0.8, Ternary::Pos);
assert!((charge.effect_at(5) - 0.8).abs() < 0.01);
assert!(charge.effect_at(0) < charge.effect_at(5)); // attenuates

// Abyssal zone
let mut abyss = AbyssalZone::new(8, 12);
abyss.record_anomaly("infinite recursion at depth 9");
assert!((abyss.safety_factor(10) - 0.5).abs() < 0.01);
```

## API

| Type | Description |
|------|-------------|
| `Depth` | Level tracker with descend/ascend |
| `PressureGauge` | Measure + classify computational pressure |
| `PressureReading` | Raw pressure + ternary classification |
| `DepthCharge` | Targeted perturbation with distance attenuation |
| `Bathyscope` | Non-invasive deep-layer observer |
| `AbyssalZone` | Anomaly tracking for extreme depths |
| `PressureDecompression` | Staged ascent planning |

## Architecture Notes

The depth system models the **γ + η = C** conservation principle through pressure invariants:

- **γ (structure)**: the depth hierarchy — the fixed topology of layers from surface to abyssal
- **η (dynamics)**: perturbations — depth charges, loads, and state transitions that stress the hierarchy
- **C (conservation)**: the pressure invariant — total computational pressure is conserved across the hierarchy, and decompression ensures it is redistributed safely

The abyssal zone represents the **breakdown of C** — the region where conservation laws fail. Anomalies are η-events that violate the expected γ structure. The safety factor quantifies how close the system is to losing its conservation guarantees.

## References

- Dijkstra, E.W. (1968). *Go To Statement Considered Harmful*. CACM — Structured depth in software.
| Bohm, C. & Jacopini, G. (1966). *Flow Diagrams, Turing Machines and Languages with Only Two Formation Rules*. — Structured programming and nesting depth.
| Boyes, R. (1908). *Decompression Sickness*. — The diving analogy for staged transitions.
| Abadi, M. & Lamport, L. (1991). *The Existence of Refinement Mappings*. — Layered abstractions and invariants.

## License: MIT
