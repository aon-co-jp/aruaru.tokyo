# PORTING.md — File portabili

Elenco di pattern implementativi riutilizzabili così come sono (o con modifiche minime) in altri progetti.

## `markdown_to_rs()` (src/main.rs)

Una funzione che converte ogni riga di Markdown in stile commento rustdoc prefissato con `//!`. Utilizzabile genericamente per README.md/CLAUDE.md/PORTING.md. Copiabile così com'è in altri repository che adottano il concetto readme-to-rs.

```rust
fn markdown_to_rs(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| if line.is_empty() { "//!".to_string() } else { format!("//! {line}") })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
```

## `fetch_repo_file()` (src/main.rs)

Un helper che recupera il contenuto grezzo di GitHub senza autenticazione (con fallback main→master). Riutilizzabile per altre funzionalità di integrazione GitHub semplicemente sostituendo il nome dell'organizzazione e il valore di timeout.

## Pattern nginx "priorità conf.d"

`/etc/nginx/conf.d/*.conf` viene incluso in `nginx.conf` prima di `/etc/nginx/sites-enabled/*.conf`, quindi posizionare una configurazione con lo stesso `server_name` in `conf.d/` le permette di avere la precedenza sulle configurazioni auto-generate da strumenti UI (es. aruaru-easyweb) in `sites-enabled/`. Tecnica riutilizzabile in caso di conflitti simili su un altro dominio (per i dettagli vedere `CLAUDE.md`).

## API GitHub: differenza tra endpoint per Organization e per account personale

`GET /orgs/{name}/repos` funziona solo per le Organization GitHub; chiamarlo per un account personale (User) restituisce 404 (confermato su infrastruttura reale). Per ottenere l'elenco completo dei repository di un account personale, usare invece `GET /users/{name}/repos`. `aon-co-jp` è un account personale, non un'Organization.

```rust
// ✗ Non funziona per un account personale (restituisce 404)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ Corretto per un account personale
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

Quando si porta una funzionalità di integrazione GitHub in altri progetti, verificare preventivamente tramite `GET /users/{name}` se il campo `"type"` è `"Organization"` o `"User"`, oppure considerare un fallback che provi entrambi gli endpoint.

## `markdown_to_github_style_html()` + `pulldown-cmark` (src/main.rs)

Una funzionalità che renderizza effettivamente un README esterno in stile GitHub per la visualizzazione. Un pattern per abbinarlo alla visualizzazione di conversione `.rs` (`markdown_to_rs`) e renderli commutabili tramite schede.

```rust
fn markdown_to_github_style_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}
```

Preparare un CSS in stile `.markdown-body` nella destinazione di visualizzazione (sottolineatura dei titoli, blocchi di codice, bordi delle tabelle, ecc.) avvicina l'aspetto a quello di GitHub.

## Uscire da un contenitore `main` stretto (centratura basata su vw)

Quando l'intera pagina deve rimanere stretta per leggibilità (es. `max-width: 780px`), ma una sezione specifica (come la visualizzazione di un README) deve espandersi a tutta la larghezza dello schermo:

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

Questo crea una sezione ampia centrata rispetto al viewport, indipendentemente dal `max-width` del contenitore genitore.

## Validare le liste recuperate dinamicamente tramite controllo di formato (non tramite corrispondenza con whitelist)

Quando si desidera accettare sia un elenco codificato in modo fisso all'avvio del server (es. `GITHUB_REPOS`) sia un elenco recuperato dinamicamente da un'API a runtime, validare l'input tramite "corrispondenza con l'elenco statico" rifiuterà i nuovi valori esistenti solo nell'elenco dinamico. Validare invece tramite "il formato è corretto" (solo alfanumerico/trattino/underscore/punto, limiti di lunghezza, ecc.).

## Il pattern "location speculare" (proxy interno tramite riscrittura dell'header Host)

Quando si vuole fare da proxy internamente solo per percorsi specifici sotto un dominio verso il contenuto reale di un altro dominio:

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

Riutilizzabile ogni volta che più domini coesistono sullo stesso VPS e si desidera che il contenuto sia visualizzabile da un altro dominio senza duplicarlo.
