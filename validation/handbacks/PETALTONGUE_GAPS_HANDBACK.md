# Upstream Gaps — petalTongue Web Mode for Static Site Serving

**Date**: 2026-05-07
**From**: projectNUCLEUS (ironGate)
**For**: petalTongue team, primalSpring
**Context**: Attempted to use petalTongue `web` mode (v1.6.6) to serve
sporePrint static site content as the first step in replacing GitHub Pages
(TUNNEL_EVOLUTION.md Step 3a).

---

## Test Methodology

Started `petaltongue web --bind 127.0.0.1:9901` from plasmidBin
(x86_64-unknown-linux-musl). Tested against sporePrint-style URL paths.

### What Worked

- `/` — returns 200 with bundled `web/index.html` (dashboard SPA)
- `/health` — returns `{"status":"ok"}`
- `/api/status` — returns version, mode, pure_rust flag
- `/api/primals` — returns primal list from DataService
- `/api/snapshot` — returns full topology snapshot
- `/api/events` — SSE stream with `text/event-stream` content type
- `--bind` flag works correctly (respects custom address)
- Graceful startup/shutdown

### What Did Not Work

- `/test.html` — 404 (no arbitrary path routing)
- `/lab/compute-access/` — 404 (no arbitrary path routing)
- Any URL outside the 6 hardcoded routes returns 404
- No way to specify an external docroot or content source
- No NestGate integration for content resolution

---

## Gap PT-1: `web` Mode Needs Catch-All Static File Route

**Source**: `src/web_mode.rs` lines 53-61
**Severity**: High (blocks Step 3a — GitHub Pages replacement)

The router has 6 fixed routes and a `/static` nest for bundled assets:

```rust
let app = Router::new()
    .route("/", get(index_handler))
    .route("/health", get(health_handler))
    .route("/api/status", get(status_handler))
    .route("/api/primals", get(primals_handler))
    .route("/api/snapshot", get(snapshot_handler))
    .route("/api/events", get(events_sse_handler))
    .nest_service("/static", ServeDir::new(WEB_STATIC_DIR))
    .with_state(data_service);
```

`WEB_STATIC_DIR` is hardcoded to `"web/static"` (line 14), which is relative
to CWD and only contains bundled dashboard assets.

**What's needed**: A catch-all route (`/{path...}`) that serves files from
a configurable docroot or NestGate backend. This is the axum `ServeDir`
pattern with fallback:

```rust
// Conceptual — what Step 3a needs
.fallback_service(
    ServeDir::new(&config.docroot)
        .append_index_html_on_directories(true)
)
```

The index handler (`index_handler`) uses `include_str!("../web/index.html")` —
compiled into the binary. This needs to coexist with external content serving.

---

## Gap PT-2: No NestGate Backend Integration

**Source**: `src/web_mode.rs` (entire file)
**Severity**: High (blocks content-addressed serving)

The web mode has zero integration with NestGate. There is no:
- NestGate connection/client
- Content hash resolution (URL path -> BLAKE3 hash -> blob)
- Collection/manifest concept (which set of files is "the current site")
- Cache layer between petalTongue and NestGate

The sovereignty spec (TUNNEL_EVOLUTION.md) envisions:

```toml
[web]
listen = "127.0.0.1:9901"
backend = "nestgate"
root_collection = "sporeprint-latest"
cache_ttl_secs = 3600
```

This config schema does not exist. petalTongue's `web` CLI accepts only
`--bind`, `--scenario`, and `--workers`.

**What's needed**: Either:
1. A NestGate storage client in petalTongue that resolves URL paths to
   content via `storage.retrieve` RPC calls, or
2. A simpler filesystem-backed mode where an external publish script
   writes files to a docroot that petalTongue serves directly

Option 2 is simpler and could be a stepping stone: serve from disk first,
graduate to NestGate later.

---

## Gap PT-3: `petaltongue_web.toml` Config Schema Missing

**Source**: `src/web_mode.rs` line 33 (`run` function signature)
**Severity**: Medium

The `run()` function takes `bind`, `scenario`, `workers`, and `data_service`
as arguments. There is no config file loading for web-specific settings.
The `--scenario` flag loads a JSON scenario file for the DataService
(dashboard visualization data), not web serving config.

**What's needed**: A web-mode-specific config file or CLI flags for:
- `docroot` — filesystem path to serve static files from
- `backend` — `"filesystem"` or `"nestgate"` content resolution
- `nestgate_url` — NestGate connection for content-addressed mode
- `collection` — which NestGate collection to serve
- `cache_ttl` — how long to cache resolved content
- `index_file` — default file for directories (default: `index.html`)
- `mime_types` — whether to auto-detect MIME types

---

## Gap PT-4: Deploy Mode Alignment

**Source**: `deploy.sh` vs `deploy_gate.sh` vs TUNNEL_EVOLUTION.md
**Severity**: Medium

Three different deployment contexts start petalTongue in different modes:

| Context | Command | Port | Mode |
|---------|---------|------|------|
| `deploy.sh` (NUCLEUS) | `petaltongue server --port 9900` | 9900 | JSON-RPC IPC |
| `deploy_gate.sh` (plasmidBin) | `petaltongue web --bind 0.0.0.0:$PORT` | varies | HTTP dashboard |
| TUNNEL_EVOLUTION.md spec | `petaltongue web` on 9901 | 9901 | HTTP content server |

These are three different use cases, but they conflict on port allocation and
mode selection. The NUCLEUS composition needs both the IPC surface (for
primal-to-primal communication) and the web surface (for HTTP content serving).

**What's needed**: Either:
1. `web` mode also starts the IPC server (like `live` mode combines GUI + IPC), or
2. Two petalTongue instances: one in `server` mode (9900, IPC) and one in
   `web` mode (9901, HTTP), or
3. A single mode that serves both HTTP and IPC on different ports

---

## Gap PT-5: `--workers` Flag Not Applied

**Source**: `src/web_mode.rs` line 36 (`workers` parameter)
**Severity**: Low

The `workers` argument is accepted and logged but not used. The axum server
uses tokio's default thread pool. For production static site serving under
load, this should either configure the tokio runtime or be removed from the
CLI to avoid misleading configuration.

---

## Positive Findings

| Finding | Significance |
|---------|-------------|
| Axum router is clean and extensible | Adding `ServeDir` fallback is mechanical |
| SSE streaming works | Live dashboard updates are production-ready |
| DataService abstraction is solid | Could be extended with content resolution |
| `tower-http::ServeDir` already imported | Static file serving capability is one line away |
| Health/status endpoints follow JSON-RPC patterns | Consistent with other primals |

---

## Recommendation

**Shortest path to Step 3a**: Add a `--docroot <path>` CLI flag to `web` mode
that adds a `ServeDir` fallback to the router. This is approximately 10 lines
of Rust:

```rust
if let Some(docroot) = docroot {
    app = app.fallback_service(
        ServeDir::new(docroot).append_index_html_on_directories(true)
    );
}
```

This enables filesystem-backed static site serving immediately. NestGate
backend integration (Gap PT-2) can come later as a separate evolution.

The `deploy/publish_sporeprint.sh` script on the projectNUCLEUS side would
run `zola build` and copy the output to petalTongue's docroot. No NestGate
needed for the first iteration.

---

## Action Summary

| Gap | Severity | Owner | Blocks |
|-----|----------|-------|--------|
| PT-1: Catch-all static route | High | petalTongue team | Step 3a (GitHub Pages replacement) |
| PT-2: NestGate backend | High | petalTongue + NestGate teams | Content-addressed serving |
| PT-3: Web config schema | Medium | petalTongue team | Production web deployment |
| PT-4: Deploy mode alignment | Medium | petalTongue + projectNUCLEUS | Dual IPC+HTTP deployment |
| PT-5: Workers flag unused | Low | petalTongue team | Production tuning |
