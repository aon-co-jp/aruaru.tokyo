# Entwicklungsrichtlinien & Umgebungsregeln (aruaru-tokyo-server)

Das Arbeitslaufwerk ist `F:\open-runo`. Dieser Abschnitt betrachtet die `CLAUDE.md` von [`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) als maßgeblich und wird in jedes Projekt kopiert/synchronisiert.

## Entwicklungsrichtlinien & Umgebungsregeln (gemeinsamer Kopf für alle Repos, hinzugefügt am 2026-07-15)

### 1. Referenzmaterial für neuere Sprachen/Frameworks

Rust selbst hat eine lange Geschichte, aber relativ neue, informationsarme Web-Frameworks, die von diesem Ökosystem übernommen wurden, wie z. B. [Poem](https://github.com/poem-web/poem), haben weit weniger Trainingsdaten, veröffentlichte Beispiele, Q&A und Blog-Berichterstattung als weit verbreitete Kombinationen wie Python+FastAPI. Dies führt dazu, dass KI-gestützte Entwicklung (Claude usw.) anfällig für Fehlerinnerungen bei APIs, Verwendung veralteter API-Versionen und andere sich wiederholende Fehlermuster ist, die in diesem Projekt tatsächlich mehrfach aufgetreten sind.

Als Gegenmaßnahme sollte die KI vor Beginn einer Aufgabe zunächst nur den relevanten Teil der folgenden Tabelle nachschlagen (es muss nicht alles gelesen werden — das Überfliegen von 1–2 relevanten Einträgen reicht aus). Dies soll den Durchsatz verbessern und KI-gestützte Nacharbeit reduzieren.

| Technologie | Offizielle Doku | GitHub | Hinweise/Blog |
|---|---|---|---|
| Rust-Sprache | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem (Web-Framework) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio (asynchrone Laufzeitumgebung) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly (wasm32 allgemein) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **Wichtiger ehrlicher Hinweis**: Diese URL-Liste wurde aus Trainingsdaten in einer Sitzung ohne Websuche-Zugriff erstellt, und ihre Existenz, aktuelle Gültigkeit und Genauigkeit wurden nicht überprüft. KI (einschließlich Claude) sollte vermeiden, diese Liste unkritisch als Grundlage für Implementierung oder Antworten zu verwenden — überprüfen Sie sie selbst, oder bestätigen Sie Primärquellen erneut in einer Sitzung mit Websuche-Zugriff. Berücksichtigen Sie immer die Möglichkeit von defekten Links, Weiterleitungen und Versionsänderungen (insbesondere breaking API-Änderungen). Neue Technologien bei Bedarf zu dieser Tabelle hinzufügen.

### 2. Gedanken zu KI-gestützten Entwicklungstools (2026-07-15, als Beobachtung des Nutzers festgehalten)

Stand 2026-07-15 können allgemeine KI-Chat-Tools wie ChatGPT kleine Webanwendungen entwickeln, aber der Nacharbeitsaufwand steigt erheblich, sobald ein System mäßig komplex/groß wird, und es gibt eine harte Obergrenze für die auf einmal handhabbare Programmgröße.

Claude Code / Claude Desktop kann Dateien durch direkte Angabe eines lokalen Laufwerks lesen/schreiben und auch GitHub-Repositorys lesen (relevant für ein Multi-Repo-Ökosystem wie dieses Projekt), was es für KI-gestützte Entwicklung in diesem Projektumfang gut geeignet macht. Empfohlen als Option bei der Einrichtung einer neuen KI-gestützten Entwicklungsumgebung.

## Rolle dieses Repositorys

Die TOP-Seite von `aruaru.tokyo`. Am 2026-07-15 wurde die zuvor in PHP implementierte Version nach Rust + [Poem](https://github.com/poem-web/poem) umgeschrieben (gemäß Nutzeranweisung: „aruaru.tokyo sollte auf Rust+Poem basieren"). `audiocafe.tokyo` bleibt PHP — ein bewusstes Design, bei dem sich der Stack je Domain unterscheidet.

## Tech-Stack

- Rust + Poem (ein leichtgewichtiges, auf hyper basierendes Web-Framework). Keine DB-Abhängigkeit, Single-Binary.
- Keine schweren Frameworks/ORMs (gemäß poem-cosmo-tauri-Ökosystem-Konvention).
- Frontend ist serverseitig als Strings zusammengesetztes reines HTML (keine Template-Engine). JS beschränkt sich auf ein Inline-`<script>` (Shuffle-Button-Verhalten).

## Kernmodule (einzelne Datei `src/main.rs`)

- `categories()` — statische Daten für den „Aruaru"-Inhalt (alltägliche, nachvollziehbare Momente)
- `render_related_sites()` / `RELATED_SITES` — Links zu verwandten Seiten auf audiocafe.tokyo
- `fetch_repo_file()` / `markdown_to_rs()` — ruft GitHub-Rohinhalte ab und konvertiert sie in `.rs`-artigen Text (die readme-to-rs-Funktion)
- `markdown_to_github_style_html()` — nutzt `pulldown-cmark`, um README usw. tatsächlich als GitHub-artiges HTML zu rendern (über einen Umschalt-Tab neben der `.rs`-Konvertierung einsehbar)
- `fetch_org_repos()` / der `api_repos`-Handler (`GET /api/repos`) — ruft bei jedem Aufruf die aktuelle vollständige Liste der Repo-Namen von `aon-co-jp` über die GitHub-API ab. **Hinweis: `aon-co-jp` ist ein persönliches Konto, keine Organisation, daher muss der `/users/{name}/repos`-Endpunkt verwendet werden** (in der realen Infrastruktur bestätigt, dass `/orgs/` 404 zurückgibt). Unterliegt der GitHub-API-Ratenbegrenzung (60/Stunde/IP), da die Aufrufe unauthentifiziert sind.
- `is_valid_repo_name()` — validiert das Format des Repo-Namens. Da dynamisch abgerufene Repos nicht in der statischen `GITHUB_REPOS`-Liste enthalten sind, wird diese Formatprüfung (kein statischer Listenabgleich) verwendet, um sie zu akzeptieren.
- Der `top()`-Handler — stellt das gesamte HTML der TOP-Seite zusammen (akzeptiert den `?repo=`-Parameter über `Query<TopQuery>`)

## Deployment

Führt `cargo build --release` direkt auf dem VPS (ConoHa, AlmaLinux) aus und bindet die resultierende Binärdatei als systemd-Dienst (`aruaru-tokyo-server.service`) an `127.0.0.1:4100`. nginx (`/etc/nginx/conf.d/aruaru.tokyo.conf`) terminiert TLS auf Port 443 und leitet über `location /` per Reverse-Proxy an diesen Port weiter.

**Spiegel-Locations `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/`**: Diese Pfade existieren ursprünglich nur auf der `audiocafe.tokyo`-Seite (PHP); um sie auch direkt von `aruaru.tokyo` aus einsehbar zu machen, proxyen dedizierte nginx-Location-Blöcke intern zu `http://127.0.0.1:80/` mit auf `audiocafe.tokyo` überschriebenem Host-Header. Diese Binärdatei selbst verarbeitet diese Pfade nie (nur `/` und `/healthz` sind in `Route` registriert).

## Verwandte Projekte

- [open-runo](https://github.com/aon-co-jp/open-runo) — der Kern des open-runo-Ökosystems (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Ursprung der Poem/Tauri-Implementierungskonvention
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — universelles Webserver-Gateway
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — Datenbankschicht (dieses Repo ist DB-unabhängig)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — kanonische Quelle der Entwicklungsregeln
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP-Auth- und Site-Management-Server
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP) — der tatsächliche Inhalt hinter den Spiegel-Locations

## Betriebsregeln

- Keine DB-abhängigen Funktionen oder schweren Frameworks in dieses Repo einbringen.
- Für externe Link-Konstanten immer eine öffentliche URL verwenden (eine Loopback-Adresse ist für Besucher-Browser unerreichbar).
- Niemals die echte IP-Adresse des VPS in Code oder Dokumentation festhalten.
- Bei jeder Aktualisierung von CLAUDE.md auch diese 10-Sprachen-Versionen (`CLAUDE-<Sprache>.md`) mit demselben Inhalt aktualisieren und zusammen pushen.

## Aktueller Stand

- 2026-07-15 Bootstrap und Produktions-Deployment abgeschlossen.
- 2026-07-16 GitHub-Integrationserweiterungen (Organisations-Link, dynamische Repo-Liste, GitHub-artiges README-Rendering, Vollbreiten-Anzeige) abgeschlossen.

## HANDOFF (aktuelles Arbeitsprotokoll, neueste zuerst)

- **2026-07-16**: GitHub-Integration erweitert. Link zur GitHub-Organisations-Startseite, dynamisches Abrufen der aktuellen Repo-Liste (`GET /api/repos`), direkter Link zum ausgewählten Repo, GitHub-artiges README-Rendering via `pulldown-cmark` (umschaltbar gegen die bestehende `.rs`-Konvertierung) und Vollbreiten-Anzeige hinzugefügt. Während der Implementierung festgestellt: `aon-co-jp` ist ein persönliches Konto, keine Organisation, daher ist `/users/{name}/repos` der korrekte Endpunkt. Repo-Validierung auch von statischem Listenabgleich auf Formatprüfung umgestellt.
- **2026-07-15**: Umschreibung von PHP nach Rust+Poem und Produktions-Deployment abgeschlossen.

## Rolle der Anwendungsserver-Schicht (open-runo / poem-cosmo-tauri, hinzugefügt am 2026-07-16)

`open-web-server` wurde als Option für die „Auslieferungs-Engine (vhost)" hinzugefügt, aber solange `open-web-server` noch nicht als vollständiger Apache+Nginx-Hybrid-Webserver funktioniert, wird die Tomcat-artige Kompatibilitätsschicht von `open-runo` oder `poem-cosmo-tauri` übernommen.

Diese zielen zusammen mit `open-raid-z` und einer VersionlessAPI darauf ab, als Framework/Middleware zu fungieren, die geschäftskritische, rund um die Uhr 24/7/365 nonstop laufende Webentwicklung vollständig unterstützt — für Anwendungsfälle, bei denen im Internet niemals Daten verloren gehen dürfen, wie 3D-Online-Spiel-Item-Käufe, Online-Finanzen, Online-Wertpapierhandel und Online-Kreditkartenzahlungen — indem ein hochmodernes „vierschichtiges, vierfach redundantes" Kommunikationssystem aufgebaut wird, das `aruaru-db` (ACID- und ZFS-kompatibel) mit einer DUAL-DATABASE-Konfiguration neben PostgreSQL kombiniert, auf Basis eines Datenbankdesigns, das Spezifikationsänderungen erleichtert, während versionsloser Betrieb neben Versionskontrolle und Git-Management aufrechterhalten wird.
