# aruaru-tokyo-server

The TOP page for [aruaru.tokyo](https://aruaru.tokyo/). Built in Rust + [Poem](https://github.com/poem-web/poem), no DB dependency, single binary.

A sister site to `audiocafe.tokyo` (PHP) on a different domain and stack, implemented per the poem-cosmo-tauri ecosystem convention (use hyper/Poem directly, no heavy frameworks or DB dependency).

## Features

- "Aruaru" (everyday relatable-moments) content across 5 genre categories, plus a random-pick display
- A quick link to [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- A link to the GitHub (aon-co-jp) account top page
- A "🔄 Fetch latest repo list" button that dynamically refreshes the repo choices from the GitHub API
- A direct link to the selected repository's GitHub page
- View README.md / CLAUDE.md / PORTING.md either as GitHub-style rendered HTML or as rustdoc-comment (`//!`) style `.rs` text, toggleable via tabs (an implementation of the readme-to-rs concept). The display area spans nearly the full viewport width (94vw, up to 1400px).

## Build & run

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

Listens on `0.0.0.0:4100` if `ARUARU_TOKYO_BIND` is unset.

## Production setup (reference)

On the VPS it runs as a systemd service bound to `127.0.0.1:4100`, with nginx terminating TLS on 443 and reverse-proxying to it. The `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` paths are mirrored to the real content on `audiocafe.tokyo` (PHP) via dedicated location blocks in the same nginx vhost (see [CLAUDE.md](CLAUDE.md) for details).

## Related projects

- [open-runo](https://github.com/aon-co-jp/open-runo) — the core Rust→WASM/tokio+hyper open-runo ecosystem
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — the source of the Poem/Tauri implementation convention
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — general-purpose web server gateway
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB layer (this repo is DB-independent)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — canonical source of development rules
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP auth / site-management server (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)

## Online-learning sites and the "AI-driven development research" article (2026-10-08)
Just before the "Claude / aruaru introduction" links on the top page, added five online-learning sites (Progate "AI-driven development", "HTML & CSS", "Python intro", paiza Learning Python, python.jp)
and, as part of the research, the article on the benefits of porting to Rust (title, URL, link) (`ONLINE_LEARNING_HTML` / `AI_DRIVEN_RESEARCH_HTML`). All five links were checked to be alive before publishing.
**Moving servers**: production lives in `/root/url/aruaru.tokyo`: `cargo build --release` then `systemctl restart aruaru.tokyo.service`. The full procedure is "VPS migration guide" in `PORTING.md` of `audiocafe-tokyo`.

## JSON handling policy: use `parse_secure_sonic` (fast type + secure) and `sonic-rs` everywhere they can be used (2026-10-14, user instruction)
The source of truth is [`RS-JSON`](https://github.com/aon-co-jp/RS-JSON) (`README.md`, `PORTING.md`).

**Policy**: wherever JSON is read, use `parse_secure_sonic` from `rust-json` (RS-JSON, the `fast` feature; the result is `sonic_rs::Value`) **everywhere it can be used**.
Use `sonic-rs` directly where possible too (e.g. reading into typed structs). Where it cannot be used, use `parse_secure` (result: `serde_json::Value`).
Untrusted input from outside (HTTP bodies, files over the network, uploads) **must always be read with size / depth / duplicate-key limits**.

**Measured (7 MB, 40,000 records)**: `serde_json` 200 ms / `parse_secure` 217 ms / **`parse_secure_sonic` 83 ms (about 2.4x faster, with the defences)** / raw `sonic-rs` 33 ms (about 6x, no defences).
Reading into `serde_json::Value` gives almost no speed-up even with SIMD (188 ms). **The speed comes from the value type that packs the result into contiguous memory.**

**Where it can / cannot be used**
- ✅ Read-heavy code that parses large or frequent JSON (cache JSON, config, API responses, logs, indexes).
- ✅ Entrances for outside input (the goal there is defence rather than speed; `parse_secure` is fine).
- ⚠️ Existing code that passes `serde_json::Value` between functions needs its type changed to `sonic_rs::Value` (methods from `JsonValueTrait` / `JsonContainerTrait`). Reading into `serde_json::Value` without that change is not faster.
- ❌ WASM (`rust_json::light`, zero dependencies) and the JSON handling on the PHP / JavaScript side (Rust components cannot be used there).
- Note: the `sonic-rs` dependency uses `unsafe` internally (our own code has none). For small JSON (a few KB) the difference is not noticeable; do speed-motivated rewrites where measurements show a benefit.

**Adoption**: in `Cargo.toml`, `rust-json = { path = "../RS-JSON", features = ["fast"] }`. Example in `RS-JSON/README.md`.

**Status in this repository**: Not yet applied (policy only). Code will be switched over place by place, starting where measurements show large or frequent JSON reads.
