# aruaru-tokyo-server

Die TOP-Seite von [aruaru.tokyo](https://aruaru.tokyo/). Geschrieben in Rust + [Poem](https://github.com/poem-web/poem), ohne Datenbankabhängigkeit, als Single-Binary.

Eine Schwesterseite von `audiocafe.tokyo` (PHP) auf einer anderen Domain und mit anderem Stack, implementiert gemäß der poem-cosmo-tauri-Ökosystem-Konvention (hyper/Poem direkt verwenden, keine schweren Frameworks oder DB-Abhängigkeit).

## Funktionen

- „Aruaru"-Inhalte (Alltagsmomente, denen jeder zustimmen kann) in 5 Genre-Kategorien, plus Zufallsanzeige
- Ein Schnelllink zu [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- Ein Link zur GitHub-Profilseite (aon-co-jp)
- Ein „🔄 Neueste Repo-Liste abrufen"-Button, der die Repo-Auswahl dynamisch über die GitHub-API aktualisiert
- Ein direkter Link zur GitHub-Seite des ausgewählten Repositorys
- README.md / CLAUDE.md / PORTING.md werden entweder als GitHub-artig gerendertes HTML oder als rustdoc-Kommentar-Stil (`//!`) `.rs`-Text angezeigt, umschaltbar über Tabs (eine Umsetzung des readme-to-rs-Konzepts). Der Anzeigebereich nutzt fast die gesamte Bildschirmbreite (94vw, bis zu 1400px).

## Bauen & Starten

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

Lauscht auf `0.0.0.0:4100`, wenn `ARUARU_TOKYO_BIND` nicht gesetzt ist.

## Produktionsaufbau (Referenz)

Auf dem VPS läuft es als systemd-Dienst gebunden an `127.0.0.1:4100`, wobei nginx TLS auf Port 443 terminiert und als Reverse-Proxy fungiert. Die Pfade `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` werden über eigene Location-Blöcke im selben nginx-vhost auf die tatsächlichen Inhalte bei `audiocafe.tokyo` (PHP) gespiegelt (Details siehe [CLAUDE.md](CLAUDE.md)).

## Verwandte Projekte

- [open-runo](https://github.com/aon-co-jp/open-runo) — der Kern des open-runo-Ökosystems (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Ursprung der Poem/Tauri-Implementierungskonvention
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — universelles Webserver-Gateway
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — Datenbankschicht (dieses Repo ist DB-unabhängig)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — kanonische Quelle der Entwicklungsregeln
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP-Auth- und Site-Management-Server (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)
