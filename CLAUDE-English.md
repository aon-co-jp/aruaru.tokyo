# Development Policy & Environment Rules (aruaru-tokyo-server)

Working drive is `F:\open-runo`. This section treats [`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)'s `CLAUDE.md` as canonical and is copied/synced to each project.

## Development Policy & Environment Rules (Shared Header Across All Repos, added 2026-07-15)

### 1. Reference material for newer languages/frameworks

Rust itself has a long history, but relatively new, lower-information web frameworks adopted by this ecosystem such as [Poem](https://github.com/poem-web/poem) have far less training data, published examples, Q&A, and blog coverage than widely-adopted combinations like Python+FastAPI. This means AI-driven development (Claude, etc.) is prone to misremembering APIs, using outdated API versions, and other rework/whack-a-mole failure patterns that have actually occurred repeatedly in this project.

As a countermeasure, before starting a task, the AI should first look up only the portion relevant to that task from the table below (no need to read everything — skimming 1–2 relevant entries is enough). This is expected to improve throughput and reduce AI-driven rework.

| Technology | Official Docs | GitHub | Notes/Blog |
|---|---|---|---|
| Rust language | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem (web framework) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio (async runtime) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly (wasm32 general) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **Important honest disclosure**: this URL list was written from training data in a session without web search access, and its existence, current validity, and accuracy have not been verified. AI (including Claude) should avoid taking this list at face value as a basis for implementation or answers — verify by actually accessing it yourself, or re-confirm primary sources in a session with web search available. Always account for the possibility of broken links, redirects, and version changes (especially breaking API changes). Append new technologies to this table as they're adopted.

### 2. Thoughts on AI-driven development tools (2026-07-15, recorded as the user's own observation)

As of 2026-07-15, general-purpose AI chat tools like ChatGPT can develop small web apps, but rework grows significantly once a system becomes moderately complex/large, and there's a hard ceiling on the program size handleable at once.

Claude Code / Claude Desktop can read/write files by directly specifying a local drive, and can also read GitHub repositories (relevant for a multi-repo ecosystem like this project), making it well-suited to AI-driven development at this project's scale. Recommended as an option when setting up a new AI-driven development environment.

## Role of this repository

The TOP page of `aruaru.tokyo`. On 2026-07-15, the previously-PHP implementation was rewritten in Rust + [Poem](https://github.com/poem-web/poem) (per user instruction: "aruaru.tokyo should be Rust+Poem based"). `audiocafe.tokyo` remains PHP — an intentional design where the stack differs per domain.

## Tech stack

- Rust + Poem (a lightweight, hyper-based web framework). No DB dependency, single binary.
- No heavy frameworks/ORMs (per poem-cosmo-tauri ecosystem convention).
- Frontend is plain HTML assembled server-side as strings (no template engine). JS is limited to an in-page `<script>` (shuffle button behavior).

## Key modules (single file `src/main.rs`)

- `categories()` — static data for the "aruaru" (relatable moments) content
- `render_related_sites()` / `RELATED_SITES` — links to related pages on audiocafe.tokyo
- `fetch_repo_file()` / `markdown_to_rs()` — fetches GitHub raw content and converts it to `.rs`-style text (the readme-to-rs feature)
- `markdown_to_github_style_html()` — uses `pulldown-cmark` to actually render README etc. as GitHub-style HTML (viewable via a toggle tab alongside the `.rs` conversion)
- `fetch_org_repos()` / the `api_repos` handler (`GET /api/repos`) — fetches the latest full list of `aon-co-jp`'s repo names from the GitHub API every time. **Note: `aon-co-jp` is a personal account, not an Organization, so the `/users/{name}/repos` endpoint must be used** (confirmed on real infrastructure that `/orgs/` returns 404). Subject to GitHub API rate limiting (60/hour/IP) since calls are unauthenticated.
- `is_valid_repo_name()` — validates repo name format. Since dynamically-fetched repos aren't in the static `GITHUB_REPOS` list, this format check (not a static-list match) is used to accept them.
- The `top()` handler — assembles the entire TOP page HTML (accepts the `?repo=` parameter via `Query<TopQuery>`)

## Deployment

Runs `cargo build --release` directly on the VPS (ConoHa, AlmaLinux), binding the resulting binary as a systemd service (`aruaru-tokyo-server.service`) on `127.0.0.1:4100`. nginx (`/etc/nginx/conf.d/aruaru.tokyo.conf`) terminates TLS on 443 and reverse-proxies via `location /` to this port.

**`/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` mirror locations**: these paths originally exist only on the `audiocafe.tokyo` side (PHP); to make them directly viewable from `aruaru.tokyo` as well, dedicated nginx location blocks internally proxy to `http://127.0.0.1:80/` with the Host header overwritten to `audiocafe.tokyo`. This binary itself never handles these paths at all (only `/` and `/healthz` are registered in `Route`).

## Related projects

- [open-runo](https://github.com/aon-co-jp/open-runo) — the core Rust→WASM/tokio+hyper open-runo ecosystem
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — the source of the Poem/Tauri implementation convention
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — general-purpose web server gateway
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB layer (this repo itself is DB-independent)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — canonical source for development rules
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP auth / site-management server
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP) — the real content behind the mirror locations

## Operating rules

- Don't bring DB-dependent features or heavy frameworks into this repo.
- Always use a public URL for external link constants (a loopback address is unreachable for viewers' browsers).
- Never record the VPS's real IP address in code or docs.
- Whenever CLAUDE.md is updated, also update these 10-language versions (`CLAUDE-<Language>.md`) with the same content and push them together.

## Current state

- 2026-07-15 Bootstrap and production deployment complete.
- 2026-07-16 GitHub-integration extensions (org link, dynamic repo list, GitHub-style README rendering, full-width display) complete.

## HANDOFF (recent work log, newest first)

- **2026-07-16**: Extended GitHub integration. Added a link to the GitHub organization top page, dynamic fetching of the latest repo list (`GET /api/repos`), a direct link to the selected repo, GitHub-style README rendering via `pulldown-cmark` (toggleable against the existing `.rs` conversion), and full-width display. Found during implementation: `aon-co-jp` is a personal account, not an Organization, so `/users/{name}/repos` is the correct endpoint. Also switched repo validation from a static-list match to a format check.
- **2026-07-15**: Rewrite from PHP to Rust+Poem and production deployment complete.

## Role of the application server layer (open-runo / poem-cosmo-tauri, added 2026-07-16)

`open-web-server` was added as an option for the "delivery engine (vhost)," but while `open-web-server` is not yet functioning as an Apache+Nginx hybrid web server, the Tomcat-like compatibility layer role is played by `open-runo` or `poem-cosmo-tauri`.

These, together with `open-raid-z` and a VersionlessAPI, aim to function as a framework/middleware that fully backs mission-critical, 24/7/365 non-stop web development — for uses that must never lose data on the internet, such as 3D online game item purchases, online finance, online securities trading, and online credit card payments — by building a cutting-edge "quadruple-layer, quadruple-redundant" communication system combining `aruaru-db` (ACID- and ZFS-compatible) with a DUAL DATABASE configuration alongside PostgreSQL, on top of a database design that makes spec changes easy while maintaining versionless operation alongside version control and Git management.
