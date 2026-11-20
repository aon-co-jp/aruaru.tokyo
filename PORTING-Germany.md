# PORTING.md — Übertragbare Dateien

Eine Liste von Implementierungsmustern, die unverändert (oder mit geringfügigen Änderungen) in andere Projekte übernommen werden können.

## `markdown_to_rs()` (src/main.rs)

Eine Funktion, die jede Zeile von Markdown in `//!`-präfixierten Rustdoc-Kommentarstil umwandelt. Universell für README.md/CLAUDE.md/PORTING.md nutzbar. Kann unverändert in andere Repos kopiert werden, die das readme-to-rs-Konzept übernehmen.

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

Ein Helfer, der GitHub-Rohinhalte ohne Authentifizierung abruft (mit main→master-Fallback). Kann für andere GitHub-Integrationsfunktionen wiederverwendet werden, indem einfach Organisationsname und Timeout-Wert ausgetauscht werden.

## nginx „conf.d-Vorrang"-Muster

`/etc/nginx/conf.d/*.conf` wird in `nginx.conf` vor `/etc/nginx/sites-enabled/*.conf` eingebunden, sodass eine Konfiguration mit demselben `server_name` in `conf.d/` Vorrang vor Konfigurationen hat, die von UI-Tools (z. B. aruaru-easyweb) automatisch in `sites-enabled/` generiert werden. Eine wiederverwendbare Technik bei ähnlichen Konflikten auf einer anderen Domain (Details siehe `CLAUDE.md`).

## GitHub-API: Unterschied zwischen Organisations- und persönlichen Konto-Endpunkten

`GET /orgs/{name}/repos` funktioniert nur für GitHub-Organisationen; der Aufruf für ein persönliches Konto (User) liefert 404 zurück (in echter Infrastruktur bestätigt). Um die vollständige Repo-Liste eines persönlichen Kontos abzurufen, verwenden Sie stattdessen `GET /users/{name}/repos`. `aon-co-jp` ist ein persönliches Konto, keine Organisation.

```rust
// ✗ Funktioniert nicht für ein persönliches Konto (liefert 404)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ Korrekt für ein persönliches Konto
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

Beim Portieren von GitHub-Integrationsfunktionen in andere Projekte entweder vorab über `GET /users/{name}` prüfen (ob das `"type"`-Feld `"Organization"` oder `"User"` ist), oder einen Fallback erwägen, der beide Endpunkte versucht.

## `markdown_to_github_style_html()` + `pulldown-cmark` (src/main.rs)

Eine Funktion, die ein externes README tatsächlich GitHub-artig zur Anzeige rendert. Ein Muster, um dies mit der `.rs`-Konvertierungsansicht (`markdown_to_rs`) zu kombinieren und über Tabs umschaltbar zu machen.

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

Die Vorbereitung von `.markdown-body`-artigem CSS am Anzeigeort (Überschriften-Unterstreichung, Codeblöcke, Tabellenrahmen usw.) bringt das Erscheinungsbild näher an GitHub heran.

## Ausbrechen aus einem engen `main`-Container (vw-basierte Zentrierung)

Wenn die gesamte Seite aus Gründen der Lesbarkeit schmal bleiben soll (z. B. `max-width: 780px`), aber ein bestimmter Abschnitt (wie eine README-Anzeige) sich über die volle Bildschirmbreite erstrecken soll:

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

Dies erzeugt einen breiten, am Viewport zentrierten Abschnitt unabhängig vom `max-width` des übergeordneten Containers.

## Dynamisch abgerufene Listen per Formatprüfung validieren (keine Whitelist-Abgleiche)

Wenn sowohl eine beim Serverstart fest codierte Liste (z. B. `GITHUB_REPOS`) als auch eine zur Laufzeit dynamisch von einer API abgerufene Liste akzeptiert werden soll, führt eine Validierung per „Abgleich mit der statischen Liste" dazu, dass neue Werte, die nur in der dynamischen Liste existieren, abgelehnt werden. Stattdessen per „ist das Format korrekt" validieren (nur alphanumerisch/Bindestrich/Unterstrich/Punkt, Längenbeschränkungen usw.).

## Das „Spiegel-Location"-Muster (interner Proxy via Host-Header-Umschreibung)

Wenn nur bestimmte Pfade unter einer Domain intern zum tatsächlichen Inhalt einer anderen Domain geproxyt werden sollen:

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

Wiederverwendbar, wenn mehrere Domains auf demselben VPS koexistieren und Inhalte von einer anderen Domain aus einsehbar sein sollen, ohne sie zu duplizieren.
