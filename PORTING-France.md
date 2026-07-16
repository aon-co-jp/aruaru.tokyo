# PORTING.md — Fichiers transposables

Liste de patrons d'implémentation réutilisables tels quels (ou avec des modifications mineures) dans d'autres projets.

## `markdown_to_rs()` (src/main.rs)

Fonction convertissant chaque ligne de Markdown en style commentaire rustdoc préfixé par `//!`. Utilisable de façon générique pour README.md/CLAUDE.md/PORTING.md. Copiable tel quel dans d'autres dépôts adoptant le concept readme-to-rs.

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

Un helper qui récupère le contenu brut de GitHub sans authentification (avec repli main→master). Réutilisable pour d'autres fonctionnalités d'intégration GitHub en changeant simplement le nom de l'organisation et la valeur du délai d'attente.

## Motif nginx « priorité conf.d »

`/etc/nginx/conf.d/*.conf` est inclus dans `nginx.conf` avant `/etc/nginx/sites-enabled/*.conf`, donc placer une configuration avec le même `server_name` dans `conf.d/` lui permet de prendre le pas sur les configurations auto-générées par des outils UI (par ex. aruaru-easyweb) dans `sites-enabled/`. Technique réutilisable en cas de conflit similaire sur un autre domaine (voir `CLAUDE.md` pour les détails).

## API GitHub : différence entre les points de terminaison Organization et compte personnel

`GET /orgs/{name}/repos` ne fonctionne que pour les Organizations GitHub ; l'appeler pour un compte personnel (User) renvoie 404 (confirmé en conditions réelles). Pour obtenir la liste complète des dépôts d'un compte personnel, utilisez plutôt `GET /users/{name}/repos`. `aon-co-jp` est un compte personnel, pas une Organization.

```rust
// ✗ Ne fonctionne pas pour un compte personnel (renvoie 404)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ Correct pour un compte personnel
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

Lors du portage de fonctionnalités d'intégration GitHub vers d'autres projets, vérifiez au préalable via `GET /users/{name}` si le champ `"type"` vaut `"Organization"` ou `"User"`, ou envisagez un repli essayant les deux points de terminaison.

## `markdown_to_github_style_html()` + `pulldown-cmark` (src/main.rs)

Une fonctionnalité qui rend réellement un README externe façon GitHub pour l'affichage. Un patron pour l'associer à la vue de conversion `.rs` (`markdown_to_rs`) et permettre de basculer entre les deux via des onglets.

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

Préparer un CSS façon `.markdown-body` à destination (soulignement des titres, blocs de code, bordures de tableau, etc.) rapproche l'apparence de celle de GitHub.

## Sortir d'un conteneur `main` étroit (centrage basé sur vw)

Lorsque la page entière doit rester étroite (par ex. `max-width: 780px`) pour la lisibilité, mais qu'une section spécifique (comme l'affichage d'un README) doit s'étendre à toute la largeur de l'écran :

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

Cela crée une section large centrée par rapport au viewport, indépendamment du `max-width` du conteneur parent.

## Valider les listes récupérées dynamiquement par vérification de format (pas par correspondance avec une liste blanche)

Lorsqu'on souhaite accepter à la fois une liste codée en dur au démarrage du serveur (par ex. `GITHUB_REPOS`) et une liste récupérée dynamiquement depuis une API à l'exécution, valider l'entrée par « correspondance avec la liste statique » rejettera les nouvelles valeurs qui n'existent que dans la liste dynamique. Validez plutôt par « le format est-il correct » (alphanumérique/tiret/underscore/point uniquement, limites de longueur, etc.).

## Le motif « emplacement miroir » (proxy interne via réécriture de l'en-tête Host)

Lorsqu'on veut proxyer en interne uniquement des chemins spécifiques d'un domaine vers le contenu réel d'un autre domaine :

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

Réutilisable chaque fois que plusieurs domaines cohabitent sur le même VPS et que l'on souhaite rendre du contenu visible depuis un autre domaine sans le dupliquer.
