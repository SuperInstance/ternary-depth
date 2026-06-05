# ternary-depth

**Pressure modeling for nested ternary systems. How deep is too deep?**

Every system has layers. Functions call functions. Modules nest inside modules. At each level of depth, the rules change — what works at the surface may fail in the abyss. This crate models that: a `Depth` type that tracks where you are in a hierarchy, a `Pressure` model that measures how much computational stress accumulates, and a `Decompression` procedure for safely returning to the surface.

Think of it like scuba diving. You can't just go deep and surface instantly — you need decompression stops. Same with nested computation. The deeper you go, the more care you need coming back.

## What's Inside

- **`Depth`** — position in a nested hierarchy with `descend()`, `ascend()`, surface/max detection
- **`Pressure`** — computational stress at a given depth. Increases with nesting, decreases at rest
- **`DepthZone`** — classify depth: `Surface`, `Shallow`, `Deep`, `Abyssal` — each with different rules
- **`decompress(depth, pressure)`** — safe ascent plan: how many stops, how long at each level
- **`DepthMap`** — track ternary values at each depth level, query cross-layer patterns
- **`collapse_depth(values_at_depth)`** — collapse a multi-layer ternary stack to a single value

## Quick Example

```rust
use ternary_depth::*;

let mut d = Depth::new(0, 10);
let mut pressure = Pressure::new();

// Descend into nested computation
for _ in 0..5 {
    d = d.descend().unwrap();
    pressure.accumulate(1.0);
}

// How deep are we?
assert_eq!(d.zone(), DepthZone::Deep);

// Safe ascent — can't surface instantly from here
let plan = decompress(d, pressure);
assert!(plan.stops() > 0);

// Track values at each depth
let mut map = DepthMap::new();
map.set(d, Ternary::Pos);
```

## The Insight

**Not all depth is equal.** Surface operations are cheap and safe. Abyssal operations are expensive and dangerous. This crate makes that explicit — you can *measure* the pressure, *plan* the decompression, and *classify* the zone. In ternary systems, the 0 state at depth behaves differently than 0 at the surface — it's a trap that's harder to escape from.

**Use cases:**
- **Compiler design** — track depth of nested IR transformations
- **Recursive algorithms** — measure and limit recursion depth safely
- **Organizational modeling** — how deep is your hierarchy? Is it too deep?
- **Security** — deep nesting as an attack surface (stack overflow, confusion attacks)
- **Game engines** — nested coordinate systems with pressure-based LOD transitions

## Install

```bash
cargo add ternary-depth
```

## License

MIT
