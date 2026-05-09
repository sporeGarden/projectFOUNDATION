# Upstream Gaps — NestGate Content-Addressed Publishing

**Date**: 2026-05-07
**From**: projectNUCLEUS (ironGate)
**For**: NestGate team, primalSpring
**Context**: Attempted to use NestGate (running on ironGate:9500) for
content-addressed static site publishing as the first step in replacing
GitHub Pages (TUNNEL_EVOLUTION.md Step 3a).

---

## Test Methodology

Tested NestGate's storage API via JSON-RPC over TCP (port 9500). Attempted
to store and retrieve HTML content, list stored items, and use the
`storage.fetch_external` method for content hashing.

---

## What Worked

| Test | Method | Result |
|------|--------|--------|
| Health check | `health.liveness` | `{"status":"alive"}` |
| Store string value | `storage.store` (key/value) | Stored, retrievable |
| Retrieve string value | `storage.retrieve` | Returns `data` and `value` fields |
| Store binary blob | `storage.store_blob` (key/blob base64) | Stored, returns `size` |
| Retrieve binary blob | `storage.retrieve_blob` | Returns base64 blob |
| Fetch external URL | `storage.fetch_external` | Returns BLAKE3 hash + content |
| List by prefix | `storage.list` (prefix) | Returns matching keys |
| Capabilities listing | `capabilities.list` | Full method list returned |

### Notable: `storage.fetch_external` Returns BLAKE3

The `storage.fetch_external` method successfully fetched `https://primals.eco/`,
returned a BLAKE3 hash (`279ead1a...`), content type, cached flag, and the
full content. This is the building block for content-addressed storage.

---

## What Did Not Work

### `content.put` / `content.get` — Method Not Found

```json
Request:  {"method":"content.put","params":{"path":"test.html","data":"..."},"id":8}
Response: {"error":{"code":-32601,"message":"Method not found"}}

Request:  {"method":"content.get","params":{"path":"test.html"},"id":9}
Response: {"error":{"code":-32601,"message":"Method not found"}}
```

These methods are referenced in `TUNNEL_EVOLUTION.md` but do not exist
in NestGate's dispatch table.

### Blob Store and KV Store Are Separate Namespaces

After storing a blob at key `sporeprint/index.html`:

```json
// storage.store_blob → success (key: "sporeprint/index.html", size: 101)
// storage.retrieve_blob → success (returns the blob)
// storage.exists → {"exists": false}  ← UNEXPECTED
// storage.list(prefix: "sporeprint/") → {"keys": []}  ← UNEXPECTED
```

The blob store and the KV store are separate storage backends. `storage.list`
and `storage.exists` only see KV entries, not blobs. This means there is
no unified way to enumerate all stored content.

### `storage.store_blob` Parameter Name Not Obvious

The parameter is `blob`, not `data`. Using `data` returns:
```
"error":"Validation error: Invalid input: blob (base64 string) required"
```

### `storage.fetch_external` Parameter Name Not Obvious

The parameter is `cache_key`, not `key`. Using `key` returns:
```
"error":"Validation error: Invalid input: cache_key (string) required"
```

---

## Gap NG-1: Content-Addressed Storage (BLAKE3 Hash as Key)

**Severity**: High (blocks content-addressed publishing pipeline)

NestGate can compute BLAKE3 hashes (proven by `storage.fetch_external`), but
there is no method that:
1. Accepts content
2. Automatically computes the BLAKE3 hash
3. Uses the hash as the storage key
4. Returns the hash as the content identifier
5. Deduplicates if the same content is stored again

**What's needed**: A `content.put` (or `storage.store_content_addressed`) method:

```json
Request:  {"method":"content.put", "params":{"data":"<base64>","content_type":"text/html"}}
Response: {"result":{"hash":"279ead1a...","size":1234,"stored":true,"deduplicated":false}}
```

And a corresponding `content.get`:

```json
Request:  {"method":"content.get", "params":{"hash":"279ead1a..."}}
Response: {"result":{"data":"<base64>","content_type":"text/html","size":1234}}
```

**Building block exists**: `storage.fetch_external` already does BLAKE3 hashing
and content storage internally. The content-addressed storage method would
reuse this logic but accept local content instead of fetching from a URL.

---

## Gap NG-2: Collection / Manifest for Versioned Releases

**Severity**: Medium (blocks "publish a site version" workflow)

There is no concept of grouping stored content into a named collection or
release. For sporePrint publishing, we need:

```json
// Create a manifest that maps URL paths to content hashes
Request:  {"method":"content.publish", "params":{
    "collection": "sporeprint-v20260507",
    "manifest": {
        "/": "279ead1a...",
        "/lab/compute-access/": "3f8b2c1e...",
        "/css/main.css": "a1b2c3d4..."
    }
}}

// Set the "latest" pointer
Request:  {"method":"content.promote", "params":{
    "collection": "sporeprint-v20260507",
    "alias": "sporeprint-latest"
}}
```

This enables atomic deployments (publish all files, then flip the pointer)
and rollback (point "latest" back to a previous collection).

---

## Gap NG-3: Unified Namespace for Blobs and KV

**Severity**: Medium (blocks content enumeration and management)

The blob store (`storage.store_blob` / `storage.retrieve_blob`) and the KV
store (`storage.store` / `storage.retrieve`) are separate namespaces.
`storage.list` and `storage.exists` only see KV entries.

**What's needed**: Either:
1. Unify the namespaces so `storage.list` returns both KV and blob entries, or
2. Add `storage.list_blobs` and `storage.blob_exists` methods, or
3. Document the separation clearly so downstream knows which API to use

For content-addressed publishing, the blob store is the right backend
(binary content with content types), but it needs its own list/exists
operations.

---

## Gap NG-4: No Streaming Store for Large Content

**Severity**: Low (sporePrint pages are small, but large datasets matter)

`storage.store_blob` requires the full content as a base64 string in a
single JSON-RPC message. For large files (datasets, plasmidBin binaries),
this is impractical.

`storage.store_stream` and `storage.retrieve_stream` methods exist in the
dispatch table (per `capabilities.list`), but their wire protocol for
streaming over JSON-RPC is unclear.

---

## Positive Findings

| Finding | Significance |
|---------|-------------|
| `storage.fetch_external` computes BLAKE3 | Content-hashing capability exists |
| Blob store works (store + retrieve) | Binary content storage is functional |
| KV store with prefix listing works | Manifest/metadata storage is functional |
| BTSP method-level auth gating | Security model is correct |
| Storage survives restart | Persistence works |

---

## Workaround: Two-Layer Publishing Without Upstream Changes

Until NG-1/NG-2 are resolved, projectNUCLEUS can implement content
publishing using the existing API:

1. Build sporePrint with Zola
2. For each file, compute BLAKE3 hash locally, use hash as key
3. Call `storage.store_blob` with hash-as-key for each file
4. Store a manifest as a KV entry mapping URL paths to hashes
5. petalTongue (once PT-1 is resolved) reads the manifest from KV,
   resolves paths to hashes, retrieves blobs via `storage.retrieve_blob`

This works today but pushes content-addressing logic to the client side
rather than NestGate handling it natively.

---

## Action Summary

| Gap | Severity | Owner | Blocks |
|-----|----------|-------|--------|
| NG-1: Content-addressed storage (BLAKE3 key) | High | NestGate team | Content publishing pipeline |
| NG-2: Collection/manifest for releases | Medium | NestGate team | Atomic site deployment |
| NG-3: Unified blob/KV namespace | Medium | NestGate team | Content enumeration |
| NG-4: Streaming store for large content | Low | NestGate team | Large file publishing |
