use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::io::Write;

/// DA51 CBOR tag (0xDA51 = 55889)
const DASL_TAG: u64 = 55889;

// ── Semantic components ─────────────────────────────────────────

/// A semantic UI component. Renderers choose presentation per a11y layer.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Component {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    Code { language: String, source: String },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Tree { label: String, children: Vec<Component> },
    List { ordered: bool, items: Vec<String> },
    Link { href: String, label: String },
    Image { alt: String, cid: String },
    KeyValue { pairs: Vec<(String, String)> },
    MapEntity { name: String, kind: String, x: f64, y: f64, meta: Vec<(String, String)> },
    Group { role: String, children: Vec<Component> },
}

// ── Shard ───────────────────────────────────────────────────────

/// One CBOR shard: a semantic component with identity.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shard {
    pub id: String,
    pub cid: String,
    pub component: Component,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Shard {
    pub fn new(id: impl Into<String>, component: Component) -> Self {
        let id = id.into();
        let json = serde_json::to_vec(&component).unwrap_or_default();
        let cid = format!("bafk{}", &hex::encode(Sha256::digest(&json))[..32]);
        Self { id, cid, component, tags: Vec::new() }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Encode as DA51-tagged CBOR bytes.
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let val = ciborium::Value::serialized(self).unwrap();
        let tagged = ciborium::Value::Tag(DASL_TAG, Box::new(val));
        ciborium::into_writer(&tagged, &mut buf).unwrap();
        buf
    }

    pub fn ipfs_url(&self) -> String { format!("https://ipfs.io/ipfs/{}", self.cid) }
    pub fn paste_url(&self, base: &str) -> String { format!("{}/raw/{}", base, self.id) }
}

// ── ShardSet (manifest) ─────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShardSet {
    pub name: String,
    pub shards: Vec<ShardRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShardRef {
    pub id: String,
    pub cid: String,
    pub tags: Vec<String>,
}

impl ShardSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), shards: Vec::new() }
    }

    pub fn add(&mut self, shard: &Shard) {
        self.shards.push(ShardRef {
            id: shard.id.clone(),
            cid: shard.cid.clone(),
            tags: shard.tags.clone(),
        });
    }

    /// Manifest as DA51-tagged CBOR.
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let val = ciborium::Value::serialized(self).unwrap();
        let tagged = ciborium::Value::Tag(DASL_TAG, Box::new(val));
        ciborium::into_writer(&tagged, &mut buf).unwrap();
        buf
    }

    /// Write all shards + manifest as a tar archive.
    pub fn to_tar<W: Write>(&self, shards: &[Shard], mut w: W) -> std::io::Result<()> {
        for shard in shards {
            let data = shard.to_cbor();
            tar_entry(&mut w, &format!("{}.cbor", shard.id), &data)?;
        }
        let manifest = self.to_cbor();
        tar_entry(&mut w, "manifest.cbor", &manifest)?;
        // Two 512-byte zero blocks = tar EOF
        w.write_all(&[0u8; 1024])?;
        Ok(())
    }
}

fn tar_entry<W: Write>(w: &mut W, name: &str, data: &[u8]) -> std::io::Result<()> {
    let mut header = [0u8; 512];
    let n = name.as_bytes();
    header[..n.len().min(100)].copy_from_slice(&n[..n.len().min(100)]);
    // mode
    header[100..107].copy_from_slice(b"0000644");
    // size in octal
    let size_str = format!("{:011o}", data.len());
    header[124..135].copy_from_slice(size_str.as_bytes());
    // typeflag = regular file
    header[156] = b'0';
    // magic
    header[257..263].copy_from_slice(b"ustar\0");
    // checksum
    header[148..156].copy_from_slice(b"        ");
    let cksum: u32 = header.iter().map(|&b| b as u32).sum();
    let ck_str = format!("{:06o}\0 ", cksum);
    header[148..156].copy_from_slice(ck_str.as_bytes());
    w.write_all(&header)?;
    w.write_all(data)?;
    // Pad to 512-byte boundary
    let pad = (512 - data.len() % 512) % 512;
    if pad > 0 { w.write_all(&vec![0u8; pad])?; }
    Ok(())
}
