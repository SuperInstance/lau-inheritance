# lau-inheritance

> An artifact is never just an object. It carries the hands that made it, the wisdom accumulated across generations, and the patience of every maker who refined it.

**lau-inheritance** models the intergenerational transfer of artifacts, knowledge, and skill in the PLATO ecosystem. Makers create artifacts, pass them to apprentices, and with each inheritance the artifact accumulates wisdom — while losing a little condition.

---

## What This Does

| Component | Purpose |
|---|---|
| **`Maker`** | A craftsperson with specialties, a signature temperament, apprentices, and a list of created artifacts. |
| **`Artifact`** | An object with a generation counter, accumulated wisdom, condition tracking, and a lineage traceable to its original maker. |
| **`InheritanceChain`** | The central registry — tracks makers, artifacts, apprenticeship relationships, lineage, and aggregate statistics. |
| **`MakerSignature`** | A 4-axis temperament profile (patience, precision, playfulness, conservation_priority). |

The crate models a simple but powerful loop:

```
Master creates artifact
    → Apprentice inherits it (generation +1, condition ×0.95)
        → Adds wisdom during use
            → Passes to next apprentice
                → After 3+ generations: becomes a masterwork
```

---

## Key Idea

**Every inheritance is a trade**: the artifact loses 5% condition but gains a piece of wisdom. After enough generations (≥3), with enough care (condition > 0.8), an artifact becomes a **masterwork** — a marker of craft lineage that has persisted through multiple hands.

The `MakerSignature` captures the temperament of each maker as four clamped scalars:
- **Patience** — willingness to iterate slowly
- **Precision** — attention to detail
- **Playfulness** — openness to experimentation
- **Conservation priority** — tendency to preserve over innovate

These signatures are metadata for now but designed to influence future crafting mechanics.

---

## Install

```toml
[dependencies]
lau-inheritance = "0.1"
```

Or:

```bash
cargo add lau-inheritance
```

### Dependencies

- `serde` 1.x (with `derive`) — serialisation
- `serde_json` 1.x (dev-only, for round-trip tests)

No async, no database, no filesystem.

---

## Quick Start

```rust
use lau_inheritance::*;

// 1. Set up the chain with pre-built makers
let mut chain = InheritanceChain::new();
chain.register_maker(old_craftsman());
chain.register_maker(young_apprentice());

// 2. Old craftsman creates an artifact
let chisel = chain.create_artifact(
    &MakerId("old-craftsman".to_string()),
    ArtifactKind::Tool,
    "Master Chisel",
);

// 3. Add wisdom
chain.add_wisdom(&chisel, "A sharp tool is a safe tool.".to_string());

// 4. Apprentice inherits it
let inherited = chain.inherit(
    &chisel,
    &MakerId("young-apprentice".to_string()),
    "Apprentice Chisel",
).unwrap();

// 5. Use it (condition decays slightly each use)
chain.use_artifact(&inherited);

// 6. Establish apprenticeship
chain.apprentice_to(
    &MakerId("young-apprentice".to_string()),
    &MakerId("old-craftsman".to_string()),
);

// 7. Query the chain
let lineage = chain.lineage(&inherited);
let stats = chain.chain_stats();
println!("Generation depth: {}", chain.generation_depth(&inherited));
println!("Masterworks: {}", chain.masterwork_count());
println!("Apprentice tree: {:?}", chain.apprentice_tree(&MakerId("old-craftsman".to_string())));
```

---

## API Reference

### Newtype IDs

- **`ArtifactId(String)`** — Artifact identifier. `Hash`, `Eq`, `Serialize`, `Deserialize`.
- **`MakerId(String)`** — Maker identifier. Same traits.

### `ArtifactKind`

| Variant | Represents |
|---|---|
| `Tool` | A physical tool |
| `Blueprint` | A design document |
| `Agent` | An agent configuration |
| `Technique` | A method or process |
| `Song` | A musical piece |
| `Recipe` | A formula or recipe |
| `Story` | A narrative |

### `MakerSignature`

```rust
pub struct MakerSignature {
    pub patience: f64,              // 0.0–1.0
    pub precision: f64,             // 0.0–1.0
    pub playfulness: f64,           // 0.0–1.0
    pub conservation_priority: f64, // 0.0–1.0
}
```

`new(...)` clamps all values to [0.0, 1.0].

### `Maker`

```rust
pub struct Maker {
    pub id: MakerId,
    pub name: String,
    pub specialties: Vec<String>,
    pub apprentices: Vec<MakerId>,
    pub artifacts_created: Vec<ArtifactId>,
    pub generations: u32,
    pub signature: MakerSignature,
}
```

| Method | Description |
|---|---|
| `apprentice_under(mentor)` | Increment generation counter. |
| `create_artifact(kind, name)` | Generate an `ArtifactId` and track it. |

### `Artifact`

```rust
pub struct Artifact {
    pub id: ArtifactId,
    pub name: String,
    pub kind: ArtifactKind,
    pub original_maker: MakerId,
    pub current_holder: Option<MakerId>,
    pub generation: u32,
    pub inherited_wisdom: Vec<String>,
    pub skill_requirements: Vec<String>,
    pub condition: f64,        // 0.0–1.0
    pub created_tick: u64,
    pub times_used: u32,
}
```

| Method | Description |
|---|---|
| `inherit_from(other, name, maker)` | Create an inherited copy: generation +1, condition ×0.95, wisdom carried forward + new entry. |
| `use_artifact()` | Increment `times_used`, condition −0.01 (clamped to 0). |
| `is_masterwork()` | `true` if generation ≥3 AND condition > 0.8. |
| `add_wisdom(s)` | Append a wisdom string. |

### `InheritanceChain`

| Method | Description |
|---|---|
| `new()` / `default()` | Empty chain. |
| `register_maker(maker)` | Add a maker to the registry. |
| `create_artifact(maker_id, kind, name)` | Maker creates an artifact (generation 0, condition 1.0). |
| `inherit(artifact_id, new_maker_id, new_name)` | Create inherited copy. Returns `None` if artifact doesn't exist. |
| `add_wisdom(artifact_id, wisdom)` | Append wisdom to an artifact. |
| `use_artifact(artifact_id)` | Use an artifact (decay condition). |
| `lineage(artifact_id)` | Trace back through generations, sorted by generation. |
| `apprentice_to(apprentice_id, mentor_id)` | Register apprenticeship (adds to mentor's list, increments apprentice's generation). |
| `maker_artifacts(maker_id)` | All artifacts where maker is original_maker OR current_holder. |
| `apprentice_tree(maker_id)` | Recursive descent of all apprentices (DFS). |
| `generation_depth(artifact_id)` | The generation field of an artifact. |
| `most_inherited()` | Artifact with the highest generation. |
| `masterwork_count()` | Number of masterwork artifacts. |
| `chain_stats()` | `InheritanceStats` aggregate. |

### `InheritanceStats`

```rust
pub struct InheritanceStats {
    pub total_makers: usize,
    pub total_artifacts: usize,
    pub max_generation: u32,
    pub total_wisdom_entries: usize,
    pub avg_condition: f64,
    pub masterwork_count: usize,
}
```

### Pre-built Makers

| Function | Name | Generations | Signature |
|---|---|---|---|
| `old_craftsman()` | "Old Craftsman" | 3 | patience=0.95, precision=0.9, playfulness=0.4, conservation=0.8 |
| `young_apprentice()` | "Young Apprentice" | 1 | patience=0.5, precision=0.6, playfulness=0.9, conservation=0.3 |

Specialties: Old Craftsman → Woodworking, Metal Forging, Story-weaving. Young Apprentice → Curiosity, Experimentation.

---

## How It Works

### Inheritance Flow

```
Original (gen 0, cond 1.0)
    │
    ▼ inherit()
Inherited (gen 1, cond 0.95)
    │  + wisdom: "Passed to {name} at generation 1"
    │  + carries forward all parent wisdom
    ▼ inherit()
Deep Inherited (gen 2, cond 0.9025)
    │  + more wisdom
    ▼ inherit()
Masterwork Candidate (gen 3, cond 0.857...)
    │  condition > 0.8 → is_masterwork() = true ✓
```

### Condition Decay

- **On inheritance:** `new_condition = old_condition × 0.95`
- **On use:** `condition = max(condition − 0.01, 0.0)`
- **Minimum condition:** 0.0 (clamped, never negative)

After 5 inheritances from perfect condition: $0.95^5 \approx 0.774$ — too worn to be a masterwork. This means masterworks require either careful maintenance or relatively few inheritance hops.

### Wisdom Accumulation

Each inheritance appends one automatic wisdom entry. Makers can add more via `add_wisdom()`. The wisdom list grows monotonically — it's never trimmed or lost.

### Lineage Tracing

The `lineage()` method finds all artifacts sharing the same `original_maker` and `kind`, sorted by generation, up to and including the target artifact. This is an approximation — it doesn't follow a strict parent-child chain but rather groups by (maker, kind).

### Apprenticeship Tree

`apprentice_tree()` does a DFS from a maker, following the `apprentices` list recursively. It excludes the root maker but includes all descendants.

### ID Generation

Artifact IDs are `{name}-{hex-timestamp}` where the hex is the nanosecond timestamp formatted as a 20-character hex string.

---

## The Math

### Condition Decay on Inheritance

$$c_{n+1} = c_n \times 0.95$$

After $k$ inheritances:

$$c_k = c_0 \times 0.95^k$$

For a masterwork: $c_k > 0.8$ and $k \ge 3$.

Solving for maximum inheritances from perfect condition:

$$0.95^k > 0.8 \implies k < \frac{\ln 0.8}{\ln 0.95} \approx 4.35$$

So at most **4 inheritances** before condition drops below the masterwork threshold (starting from condition 1.0).

### Condition Decay from Use

Each use: $c \leftarrow \max(c - 0.01, 0)$.

A brand-new artifact (condition 1.0) can be used 100 times before becoming unusable. An inherited artifact at condition 0.85 has 85 uses left.

### Masterwork Criterion

$$\text{is\_masterwork} = (\text{generation} \ge 3) \land (\text{condition} > 0.8)$$

### Average Condition

$$\bar{c} = \frac{1}{|A|} \sum_{a \in A} c_a$$

where $A$ is the set of all artifacts.

---

## Testing

**42 tests** covering:

- Maker creation, apprenticeship, artifact creation, signature clamping
- Artifact use decay, inheritance, wisdom accumulation, masterwork detection
- Condition floor (never below 0)
- All 7 `ArtifactKind` variants
- `InheritanceChain` CRUD: register, create, inherit (existing and nonexistent), wisdom, use
- Lineage tracing (single, deep chain)
- Maker artifacts (original + held)
- Apprentice tree (with and without apprentices)
- Generation depth, most inherited, masterwork count
- Chain statistics (empty and populated)
- Newtype ID equality and hashing
- Serde round-trips for `Maker`, `Artifact`, and `InheritanceChain`
- Pre-built maker signature verification
- Edge cases: double inheritance, condition decay, missing artifacts

```bash
cargo test
```

---

## License

MIT
