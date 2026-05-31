# lau-inheritance

The chisel's memory. Wisdom passed down through generations of builders, where each maker inherits techniques, tools, and hard-won lessons from those who came before.

Inspired by "The Voyage" — Marcus stealing the chisel, discovering that the tool carries the knowledge of everyone who ever held it.

## The concept in 60 seconds

Inheritance in PLATO isn't class-based OOP — it's *craft-based*. A maker creates artifacts (tools, techniques, patterns). When a new maker inherits those artifacts, they receive:

- **The artifact itself** (what was built)
- **The provenance** (how and why it was built)
- **The mastery level** (how good the original maker was)
- **The lessons** (what the maker learned while building it)

Mastery compounds across generations — each maker can improve on what they inherited, and the delta is tracked.

## Quick start

```rust
use lau_inheritance::{Maker, Artifact, ArtifactKind, InheritanceChain};

let master = Maker::new("original_builder").with_mastery(0.9);
let artifact = Artifact::new("hermes_construct")
    .with_kind(ArtifactKind::Tool)
    .with_lesson("The deadlock was in port.rs:102. Use AtomicBool.");

let apprentice = Maker::new("ensign_7").with_mastery(0.4);

// Inherit
let inherited = apprentice.inherit(&artifact, &master);
assert!(inherited.mastery() > 0.4); // inherited knowledge boosts mastery

// Track the full chain
let chain = InheritanceChain::from_artifact(&inherited);
for generation in chain.generations() {
    println!("{} (mastery: {:.2})", generation.maker, generation.mastery);
}
```

## Contributing

[Open an issue](https://github.com/SuperInstance/lau-inheritance/issues) or PR.
