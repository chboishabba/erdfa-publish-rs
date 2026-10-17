# DA51 × SLR proof-linked artifact packaging boundary

Date: 2026-10-17

`erdfa-publish-rs` remains the packaging/content-identity substrate for logical artifacts, shards, manifests, CBOR/tar output and sink publication helpers. Proof-linked provenance is additive metadata; ERDFA does not become the owner of PNF, proof obligations, proof payment, DA51 semantics, or theorem certification.

A proof-linked logical artifact may retain optional:

```text
da51Address
formalDemandRef
formalEvidenceRef
statementDigest
```

alongside the existing logical/content coordinates:

```text
artifactId
artifactRevision
shardId
contentDigest
objectRefs[]
```

Required boundary:

```text
logical artifact/shard identity -> package/serialize -> attach sink refs
```

not:

```text
CID/path -> infer artifact meaning/proof authority
```

Hard firewalls:

```text
CBOR validity != theorem proof
manifest validity != theorem proof
CID != DA51 identity
CID/HF/file URI != shard semantic identity
formalEvidenceRef != ERDFA certification
runtime trace/benchmark != formal proof
publication receipt != SLR payment
```

JSON and CBOR projections of one artifact must preserve the same logical IDs, content digests, selector meaning, and optional proof-provenance joins. A backend statement-digest change requires a new formal demand or explicit equivalence/transport evidence upstream; packaging must not normalize that distinction away.
