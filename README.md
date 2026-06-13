# Ternary Depth — Layered Abstraction and Pressure Modeling for Nested Systems

**Ternary Depth** models how constructs at different layers of abstraction interact within nested ternary systems. It provides depth tracking (surface → abyssal), computational pressure measurement between layers, safe transition enforcement, and characterization of the **abyssal zone** where normal rules break down — all using ternary values {-1, 0, +1} to classify layer health.

## Why It Matters

Complex systems are nested: functions call functions, modules contain modules, fleets contain nodes contain GPUs. Understanding the depth structure — which layers are healthy, which are under pressure, where communication breaks down — is essential for debugging and optimization. This crate provides the formal vocabulary: `Depth` tracks position in the hierarchy, `Pressure` measures computational stress between layers, and `SafeTransition` enforces that layer changes don't corrupt state. Without depth modeling, nested system failures are opaque — you know something broke but not which layer.

## How It Works

### Depth Tracking

A `Depth` value carries `level` (current depth) and `max_level` (boundary). The surface is level 0; deeper levels have higher numbers. Movement is controlled:

- `descend()` → Depth + 1 (returns None if at max)
- `ascend()` → Depth - 1 (returns None if at surface)

Both are O(1). The max_level prevents infinite recursion — each subsystem declares its maximum nesting depth.

### Layer Classification

Each layer is classified with a ternary health value:
- **+1 (Stable)**: Layer is functioning normally
- **0 (Transitional)**: Layer is between states, undergoing change
- **-1 (Critical)**: Layer is under stress, may fail

### Pressure Model

Computational pressure measures the flow of information between adjacent layers. When upper layers demand more than lower layers can provide, pressure builds. The pressure between layer i and layer i+1 is:

```
P(i, i+1) = demand(i) - supply(i+1)
```

High pressure triggers SafeTransition enforcement: the system must resolve the pressure before allowing further descent.

### Abyssal Zone

The deepest layers (near max_level) form the "abyssal zone" where normal rules break down. In the abyssal zone:
- Pressure is maximum
- Transitions are irreversible (cannot ascend safely)
- Ternary values may oscillate (indicating instability)

The system must detect and flag abyssal conditions before they cause cascading failures.

## Quick Start

```rust
use ternary_depth::{Depth, Ternary};

let surface = Depth::surface();
assert!(surface.is_surface());

let deeper = surface.descend().unwrap();
assert_eq!(deeper.level, 1);

let max_depth = Depth::new(0, 5);
let mut current = max_depth;
for _ in 0..5 {
    current = current.descend().unwrap_or(current);
}
assert!(current.is_max()); // At level 5, can't go deeper
```

```bash
cargo add ternary-depth
```

## API

| Type / Function | Description |
|---|---|
| `Depth` | `{ level, max_level }` with `descend()`, `ascend()`, `is_surface()`, `is_max()` |
| `Ternary` | Layer health: `Neg(-1)`, `Zero(0)`, `Pos(1)` |
| `Pressure` | Computational stress between layers |

## Architecture Notes

In **SuperInstance**, depth models the abstraction hierarchy from fleet → room → agent → GPU → kernel. The γ + η = C conservation law applies at each layer boundary: what flows down as γ (compute demand) must be balanced by η (resource availability) at the receiving layer. The abyssal zone corresponds to the hardware-software boundary where abstractions leak. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Simon, Herbert A. "The Architecture of Complexity," *Proceedings of the American Philosophical Society*, 106(6), 1962 — hierarchical systems.
- Tanenbaum, Andrew. *Computer Networks*, 5th ed., 2010 — layered protocol stacks.
| Ousterhout, John. *A Philosophy of Software Design*, Yaknyam Press, 2018 — deep modules and abstraction.

## License

MIT
