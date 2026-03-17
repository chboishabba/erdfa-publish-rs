# Integrating erdfa-publish into shem-hamephorash-72

## Goal

Export the Cl(15,0,0) Boustrophedon analysis as semantic CBOR shards instead of (or alongside) terminal output. Each section of the run becomes a typed shard that any renderer can present.

## Setup

Add to `Cargo.toml`:

```toml
[dependencies]
erdfa-publish = { path = "../erdfa-publish" }
```

Add to `main.rs`:

```rust
use erdfa_publish::*;
```

## What to export

Map the existing `println!` sections to components:

| Current output | Component type | Shard ID pattern |
|---|---|---|
| Verse group header | `Heading { level: 2 }` | `group-{book}-{ch}-{v}` |
| Triplet lines | `Table` with columns: index, name, norm, basin, grades | `triplets-{book}-{ch}` |
| Group product summary | `KeyValue` with norm, basin, clock, grades | `product-{book}-{ch}` |
| Grade spectrum | `Table` with grade, count | `grade-spectrum` |
| Basin distribution | `Table` with basin, AZ class, count | `basin-dist` |
| Accumulator blades | `Table` with blade, bits, value, SSP | `accumulator` |
| Cross-book resonance | `Table` with name, locations, basin, grades | `resonance` |
| ALIFE evolution log | `List` of generation summaries | `alife-log` |
| Location map | `Group { role: "map" }` containing `MapEntity` children | `location-map` |
| Actor simulation steps | `Tree` with step → agent → attractor | `actor-sim` |
| Classifier results | `Table` with mark, location, true/pred labels | `classifier-results` |

## Minimal pattern

At the top of `main()`:

```rust
let mut shards: Vec<Shard> = Vec::new();
let mut manifest = ShardSet::new("shem-72-run");
```

Create shards alongside existing output:

```rust
let s = Shard::new("grade-spectrum", Component::Table {
    headers: vec!["Grade".into(), "Count".into()],
    rows: gv.iter().map(|(g, c)| vec![format!("g{}", g), c.to_string()]).collect(),
}).with_tags(vec!["cl15".into(), "spectrum".into()]);
manifest.add(&s);
shards.push(s);
```

For map entities:

```rust
let s = Shard::new(format!("entity-{}", name), Component::MapEntity {
    name, kind, x: x as f64, y: y as f64,
    meta: vec![("blade".into(), format!("0b{:015b}", blade))],
});
manifest.add(&s);
shards.push(s);
```

## Export at end of main()

```rust
let mut tar = std::fs::File::create("shem-72-shards.tar").unwrap();
manifest.to_tar(&shards, &mut tar).unwrap();
eprintln!("Exported {} shards → shem-72-shards.tar", shards.len());
```

## Tags

`cl15`, `basin`, `alife`, `map`, `classifier`, `egyptian`, `spectrum`

## Notes

- Keep existing `println!` — shards are additive
- CIDs are deterministic from content
- Tar output can be piped to `ipfs add` directly
