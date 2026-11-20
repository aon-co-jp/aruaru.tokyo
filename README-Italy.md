# aruaru-tokyo-server

La pagina TOP di [aruaru.tokyo](https://aruaru.tokyo/). Scritta in Rust + [Poem](https://github.com/poem-web/poem), senza dipendenza da database, binario singolo.

Un sito gemello di `audiocafe.tokyo` (PHP) su un dominio e uno stack diversi, implementato secondo la convenzione dell'ecosistema poem-cosmo-tauri (usare direttamente hyper/Poem, senza framework pesanti né dipendenza da database).

## Funzionalità

- Contenuti "aruaru" (momenti quotidiani in cui tutti si riconoscono) suddivisi in 5 categorie, più visualizzazione casuale
- Un link rapido a [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- Un link alla pagina principale dell'account GitHub (aon-co-jp)
- Un pulsante "🔄 Recupera l'elenco più recente dei repository" che aggiorna dinamicamente le scelte di repository tramite l'API GitHub
- Un link diretto alla pagina GitHub del repository selezionato
- Visualizzazione di README.md / CLAUDE.md / PORTING.md sia come HTML renderizzato in stile GitHub sia come testo `.rs` in stile commento rustdoc (`//!`), commutabile tramite schede (un'implementazione del concetto readme-to-rs). L'area di visualizzazione occupa quasi tutta la larghezza dello schermo (94vw, fino a 1400px).

## Compilazione e avvio

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

Se `ARUARU_TOKYO_BIND` non è impostato, resta in ascolto su `0.0.0.0:4100`.

## Configurazione di produzione (riferimento)

Sul VPS viene eseguito come servizio systemd collegato a `127.0.0.1:4100`, con nginx che termina il TLS sulla porta 443 e funge da reverse proxy. I percorsi `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` vengono rispecchiati verso i contenuti reali su `audiocafe.tokyo` (PHP) tramite blocchi `location` dedicati nello stesso vhost nginx (per i dettagli vedere [CLAUDE.md](CLAUDE.md)).

## Progetti correlati

- [open-runo](https://github.com/aon-co-jp/open-runo) — il nucleo dell'ecosistema open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — l'origine della convenzione di implementazione Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — gateway server web generico
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — livello database (questo repo ne è indipendente)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — fonte canonica delle regole di sviluppo
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — server di autenticazione OTP e gestione siti (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)
