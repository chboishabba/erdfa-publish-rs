# erdfa-publish

Semantic UI components as CBOR shards. Define structure in Rust, render anywhere.

## Concept

Instead of writing HTML/JS, you describe **what** your UI contains — headings, tables, trees, maps, code blocks — as typed Rust structs. These get serialized as CBOR shards with content-addressed IDs. Any renderer (browser, screen reader, CLI, embedded display) loads the shards and presents them according to its own a11y layer and CSS.

```
Rust program → Component structs → CBOR shards → loader → renderer
                                                          ├─ visual CSS
                                                          ├─ screen reader
                                                          ├─ CLI table
                                                          └─ braille display
```

## Install

Add to `Cargo.toml`:

```toml
[dependencies]
erdfa-publish = { path = "../erdfa-publish" }
```

## Quick start

```rust
use erdfa_publish::*;

// 1. Create semantic components
let heading = Component::Heading { level: 1, text: "Results".into() };
let table = Component::Table {
    headers: vec!["Name".into(), "Value".into()],
    rows: vec![vec!["alpha".into(), "0.73".into()]],
};

// 2. Wrap as shards (auto-generates CID from content hash)
let s1 = Shard::new("result-heading", heading);
let s2 = Shard::new("result-table", table).with_tags(vec!["data".into()]);

// 3. Build manifest
let mut set = ShardSet::new("my-results");
set.add(&s1);
set.add(&s2);

// 4. Export
let cbor = set.to_cbor();              // DA51-tagged CBOR manifest
let shard_bytes = s1.to_cbor();        // individual shard as CBOR

// 5. Or bundle everything as a tar archive
let mut tar = std::fs::File::create("output.tar").unwrap();
set.to_tar(&[s1, s2], &mut tar).unwrap();
```

## Component types

| Type | Fields | Semantic meaning |
|------|--------|-----------------|
| `Heading` | `level`, `text` | Section header (1–6) |
| `Paragraph` | `text` | Block of prose |
| `Code` | `language`, `source` | Source code with syntax hint |
| `Table` | `headers`, `rows` | Tabular data |
| `Tree` | `label`, `children` | Recursive hierarchy |
| `List` | `ordered`, `items` | Ordered or unordered list |
| `Link` | `href`, `label` | Navigation reference |
| `Image` | `alt`, `cid` | Image by content address |
| `KeyValue` | `pairs` | Metadata / properties |
| `MapEntity` | `name`, `kind`, `x`, `y`, `meta` | Positioned entity on a map |
| `Group` | `role`, `children` | Container with semantic role |

## CBOR format

Every shard and manifest is wrapped in CBOR tag 55889 (`0xDA51`):

```
DA51 tag → {
  "id": "result-table",
  "cid": "bafk205260a6c670b02f...",
  "component": { "type": "Table", "headers": [...], "rows": [...] },
  "tags": ["data"]
}
```

Manifests:

```
DA51 tag → {
  "name": "my-results",
  "shards": [
    { "id": "result-heading", "cid": "bafk...", "tags": [] },
    { "id": "result-table", "cid": "bafk...", "tags": ["data"] }
  ]
}
```

## Tar archive layout

```
output.tar
├── result-heading.cbor    # DA51-tagged shard
├── result-table.cbor      # DA51-tagged shard
└── manifest.cbor          # DA51-tagged ShardSet
```

## Rendering

Shards are semantic, not visual. A loader fetches shards by CID, reads the `type` field, and delegates to the active a11y layer:

- **Visual**: CSS grid, syntax highlighting, interactive maps
- **Screen reader**: ARIA roles derived from component type
- **CLI**: ASCII tables, indented trees, plain text
- **Minimal**: progressive loading — show N/total progress, activate when complete

The `Group` component with a `role` field maps directly to ARIA landmarks (`navigation`, `main`, `complementary`, etc.).

## URLs

```rust
let shard = Shard::new("my-data", component);
shard.ipfs_url()                    // https://ipfs.io/ipfs/bafk...
shard.paste_url("http://host:8090") // http://host:8090/raw/my-data
```
