# Politica di sviluppo & regole d'ambiente (aruaru-tokyo-server)

L'unità di lavoro è `F:\open-runo`. Questa sezione considera il `CLAUDE.md` di [`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) come canonico, copiato e sincronizzato in ogni progetto.

## Politica di sviluppo & regole d'ambiente (intestazione comune a tutti i repository, aggiunta il 2026-07-15)

### 1. Materiale di riferimento per linguaggi/framework più recenti

Rust stesso ha una lunga storia, ma framework web relativamente nuovi e con poca documentazione adottati da questo ecosistema, come [Poem](https://github.com/poem-web/poem), hanno molti meno dati di addestramento, esempi pubblicati, Q&A e copertura sui blog rispetto a combinazioni ampiamente adottate come Python+FastAPI. Questo rende lo sviluppo guidato dall'IA (Claude, ecc.) incline a ricordare male le API, usare versioni obsolete delle API e altri schemi di errore ripetitivi già verificatisi più volte in questo progetto.

Come contromisura, prima di iniziare un'attività, l'IA dovrebbe prima consultare solo la parte rilevante della tabella seguente (non è necessario leggere tutto — scorrere 1-2 voci rilevanti è sufficiente). Questo dovrebbe migliorare la produttività e ridurre il lavoro ripetuto guidato dall'IA.

| Tecnologia | Documentazione ufficiale | GitHub | Note/Blog |
|---|---|---|---|
| Linguaggio Rust | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem (framework web) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio (runtime asincrono) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly (wasm32 in generale) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **Avviso importante (divulgazione onesta)**: questo elenco di URL è stato scritto dai dati di addestramento in una sessione senza accesso alla ricerca web, e la sua esistenza, validità attuale e accuratezza non sono state verificate. L'IA (Claude incluso) dovrebbe evitare di prendere questo elenco per buono come base per implementazioni o risposte — verificare accedendovi personalmente, oppure riconfermare le fonti primarie in una sessione con ricerca web disponibile. Considerare sempre la possibilità di link non funzionanti, reindirizzamenti e cambi di versione (specialmente cambiamenti API che rompono la compatibilità). Aggiungere nuove tecnologie a questa tabella quando adottate.

### 2. Riflessioni sugli strumenti di sviluppo guidati dall'IA (2026-07-15, registrato come osservazione dell'utente)

Al 2026-07-15, strumenti di chat IA generici come ChatGPT possono sviluppare piccole app web, ma il lavoro ripetuto cresce significativamente quando un sistema diventa moderatamente complesso/grande, e c'è un limite rigido alla dimensione del programma gestibile alla volta.

Claude Code / Claude Desktop può leggere/scrivere file specificando direttamente un'unità locale, e può anche leggere repository GitHub (rilevante per un ecosistema multi-repository come questo progetto), rendendolo adatto allo sviluppo guidato dall'IA su questa scala di progetto. Consigliato come opzione quando si configura un nuovo ambiente di sviluppo guidato dall'IA.

## Ruolo di questo repository

La pagina TOP di `aruaru.tokyo`. Il 2026-07-15, l'implementazione precedentemente in PHP è stata riscritta in Rust + [Poem](https://github.com/poem-web/poem) (su istruzione dell'utente: "aruaru.tokyo dovrebbe basarsi su Rust+Poem"). `audiocafe.tokyo` rimane PHP — un design intenzionale in cui lo stack differisce per dominio.

## Stack tecnologico

- Rust + Poem (un framework web leggero basato su hyper). Nessuna dipendenza da database, binario singolo.
- Nessun framework/ORM pesante (secondo la convenzione dell'ecosistema poem-cosmo-tauri).
- Il frontend è HTML puro assemblato lato server come stringhe (nessun motore di template). Il JS è limitato a uno `<script>` incorporato nella pagina (comportamento del pulsante shuffle).

## Moduli principali (file singolo `src/main.rs`)

- `categories()` — dati statici per il contenuto "aruaru" (momenti quotidiani in cui ci si riconosce)
- `render_related_sites()` / `RELATED_SITES` — link alle pagine correlate su audiocafe.tokyo
- `fetch_repo_file()` / `markdown_to_rs()` — recupera il contenuto grezzo di GitHub e lo converte in testo stile `.rs` (funzionalità readme-to-rs)
- `markdown_to_github_style_html()` — usa `pulldown-cmark` per renderizzare effettivamente README ecc. come HTML in stile GitHub (visibile tramite una scheda di commutazione accanto alla conversione `.rs`)
- `fetch_org_repos()` / il gestore `api_repos` (`GET /api/repos`) — recupera ogni volta l'elenco completo e aggiornato dei nomi dei repository di `aon-co-jp` dall'API GitHub. **Nota: `aon-co-jp` è un account personale, non un'Organizzazione, quindi va usato l'endpoint `/users/{name}/repos`** (confermato su infrastruttura reale che `/orgs/` restituisce 404). Soggetto al rate limiting dell'API GitHub (60/ora/IP) poiché le chiamate non sono autenticate.
- `is_valid_repo_name()` — valida il formato del nome del repository. Poiché i repository recuperati dinamicamente non sono nell'elenco statico `GITHUB_REPOS`, questo controllo di formato (non una corrispondenza con l'elenco statico) viene usato per accettarli.
- Il gestore `top()` — assembla l'intero HTML della pagina TOP (accetta il parametro `?repo=` tramite `Query<TopQuery>`)

## Deployment

Esegue `cargo build --release` direttamente sul VPS (ConoHa, AlmaLinux), collegando il binario risultante come servizio systemd (`aruaru-tokyo-server.service`) su `127.0.0.1:4100`. nginx (`/etc/nginx/conf.d/aruaru.tokyo.conf`) termina il TLS sulla porta 443 e fa da reverse proxy tramite `location /` verso questa porta.

**Location speculari `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/`**: questi percorsi esistono originariamente solo sul lato `audiocafe.tokyo` (PHP); per renderli visualizzabili direttamente anche da `aruaru.tokyo`, blocchi `location` nginx dedicati fanno da proxy interno verso `http://127.0.0.1:80/` con l'header Host riscritto in `audiocafe.tokyo`. Questo binario stesso non gestisce mai questi percorsi (solo `/` e `/healthz` sono registrati in `Route`).

## Progetti correlati

- [open-runo](https://github.com/aon-co-jp/open-runo) — il nucleo dell'ecosistema open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — l'origine della convenzione di implementazione Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — gateway server web generico
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — livello database (questo repo ne è indipendente)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — fonte canonica delle regole di sviluppo
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — server di autenticazione OTP e gestione siti
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP) — il contenuto reale dietro le location speculari

## Regole operative

- Non introdurre funzionalità dipendenti da database o framework pesanti in questo repo.
- Usare sempre un URL pubblico per le costanti di link esterni (un indirizzo loopback è irraggiungibile per i browser dei visitatori).
- Non registrare mai l'indirizzo IP reale del VPS nel codice o nella documentazione.
- Ogni volta che CLAUDE.md viene aggiornato, aggiornare anche queste versioni in 10 lingue (`CLAUDE-<Lingua>.md`) con lo stesso contenuto e fare push insieme.

## Stato attuale

- 2026-07-15 Bootstrap e deployment in produzione completati.
- 2026-07-16 Estensioni di integrazione GitHub (link all'organizzazione, elenco dinamico dei repository, rendering README stile GitHub, visualizzazione a larghezza piena) completate.

## HANDOFF (registro di lavoro recente, dal più recente)

- **2026-07-16**: Estesa l'integrazione GitHub. Aggiunto un link alla pagina principale dell'organizzazione GitHub, recupero dinamico dell'elenco più recente dei repository (`GET /api/repos`), link diretto al repository selezionato, rendering README stile GitHub tramite `pulldown-cmark` (commutabile con la conversione `.rs` esistente), e visualizzazione a larghezza piena. Scoperto durante l'implementazione: `aon-co-jp` è un account personale, non un'Organizzazione, quindi `/users/{name}/repos` è l'endpoint corretto. Anche la validazione dei repository è passata da una corrispondenza con elenco statico a un controllo di formato.
- **2026-07-15**: riscrittura da PHP a Rust+Poem e deployment in produzione completati.

## Ruolo del livello application server (open-runo / poem-cosmo-tauri, aggiunto il 2026-07-16)

`open-web-server` è stato aggiunto come opzione per il "motore di distribuzione (vhost)", ma finché `open-web-server` non funziona ancora come un server web ibrido Apache+Nginx completo, il ruolo di livello di compatibilità in stile Tomcat è svolto da `open-runo` o `poem-cosmo-tauri`.

Questi, insieme a `open-raid-z` e a una VersionlessAPI, mirano a funzionare come framework/middleware che supporta pienamente lo sviluppo web mission-critical, no-stop 24 ore su 24, 365 giorni all'anno — per usi in cui non è mai tollerabile perdere dati su Internet, come acquisti di oggetti in giochi online 3D, finanza online, trading di titoli online e pagamenti con carta di credito online — costruendo un sistema di comunicazione all'avanguardia "a quattro livelli, quadruplice ridondanza" che combina `aruaru-db` (compatibile ACID e ZFS) con una configurazione DUAL DATABASE accanto a PostgreSQL, su un design del database che rende semplici i cambiamenti di specifica mantenendo al contempo un funzionamento senza versione insieme al controllo versione e alla gestione Git.
