# Politique de développement & règles d'environnement (aruaru-tokyo-server)

Le disque de travail est `F:\open-runo`. Cette section considère le `CLAUDE.md` de [`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) comme la référence canonique, copiée et synchronisée vers chaque projet.

## Politique de développement & règles d'environnement (en-tête commun à tous les dépôts, ajouté le 2026-07-15)

### 1. Documentation de référence pour les langages/frameworks récents

Rust lui-même a une longue histoire, mais des frameworks web relativement récents et peu documentés adoptés par cet écosystème, comme [Poem](https://github.com/poem-web/poem), disposent de bien moins de données d'entraînement, d'exemples publiés, de Q&A et d'articles de blog que des combinaisons largement répandues comme Python+FastAPI. Le développement piloté par IA (Claude, etc.) est donc sujet à des erreurs de mémorisation d'API, à l'utilisation d'anciennes versions d'API, et à d'autres schémas d'échec répétitifs déjà survenus plusieurs fois dans ce projet.

En contre-mesure, avant de commencer une tâche, l'IA doit d'abord consulter uniquement la partie pertinente du tableau ci-dessous (pas besoin de tout lire — parcourir 1 ou 2 entrées pertinentes suffit). Cela devrait améliorer le rendement et réduire les reprises liées au développement piloté par IA.

| Technologie | Documentation officielle | GitHub | Notes/Blog |
|---|---|---|---|
| Langage Rust | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem (framework web) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio (runtime asynchrone) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly (wasm32 en général) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **Avertissement important (divulgation honnête)** : cette liste d'URL a été rédigée à partir des données d'entraînement dans une session sans accès à la recherche web, et son existence, sa validité actuelle et son exactitude n'ont pas été vérifiées. L'IA (y compris Claude) doit éviter de prendre cette liste pour argent comptant comme base d'implémentation ou de réponse — vérifiez en y accédant vous-même, ou reconfirmez les sources primaires dans une session avec accès à la recherche web. Tenez toujours compte de la possibilité de liens brisés, de redirections et de changements de version (en particulier les changements d'API cassants). Ajoutez les nouvelles technologies à ce tableau au fur et à mesure.

### 2. Réflexions sur les outils de développement piloté par IA (2026-07-15, enregistré comme observation de l'utilisateur)

Au 2026-07-15, les outils de chat IA généralistes comme ChatGPT peuvent développer de petites applications web, mais les reprises augmentent considérablement une fois qu'un système devient modérément complexe/important, et il existe une limite stricte à la taille de programme gérable à la fois.

Claude Code / Claude Desktop peut lire/écrire des fichiers en spécifiant directement un lecteur local, et peut également lire des dépôts GitHub (pertinent pour un écosystème multi-dépôts comme ce projet), ce qui le rend bien adapté au développement piloté par IA à l'échelle de ce projet. Recommandé comme option lors de la mise en place d'un nouvel environnement de développement piloté par IA.

## Rôle de ce dépôt

La page TOP de `aruaru.tokyo`. Le 2026-07-15, l'implémentation jusqu'alors en PHP a été réécrite en Rust + [Poem](https://github.com/poem-web/poem) (selon l'instruction de l'utilisateur : « aruaru.tokyo devrait être basé sur Rust+Poem »). `audiocafe.tokyo` reste en PHP — une conception intentionnelle où la pile technique diffère selon le domaine.

## Pile technique

- Rust + Poem (un framework web léger basé sur hyper). Aucune dépendance à une base de données, binaire unique.
- Pas de frameworks/ORM lourds (selon la convention de l'écosystème poem-cosmo-tauri).
- Le frontend est du HTML brut assemblé côté serveur sous forme de chaînes de caractères (pas de moteur de template). Le JS se limite à un `<script>` intégré à la page (comportement du bouton shuffle).

## Modules principaux (fichier unique `src/main.rs`)

- `categories()` — données statiques du contenu « aruaru » (moments du quotidien)
- `render_related_sites()` / `RELATED_SITES` — liens vers les pages associées sur audiocafe.tokyo
- `fetch_repo_file()` / `markdown_to_rs()` — récupère le contenu brut GitHub et le convertit en texte façon `.rs` (fonctionnalité readme-to-rs)
- `markdown_to_github_style_html()` — utilise `pulldown-cmark` pour rendre réellement le README etc. en HTML façon GitHub (visible via un onglet de bascule à côté de la conversion `.rs`)
- `fetch_org_repos()` / le gestionnaire `api_repos` (`GET /api/repos`) — récupère à chaque fois la liste complète et actualisée des noms de dépôts de `aon-co-jp` depuis l'API GitHub. **Remarque : `aon-co-jp` est un compte personnel, pas une Organisation, il faut donc utiliser le point de terminaison `/users/{name}/repos`** (confirmé en conditions réelles que `/orgs/` renvoie 404). Soumis à la limitation de débit de l'API GitHub (60/heure/IP) car les appels ne sont pas authentifiés.
- `is_valid_repo_name()` — valide le format du nom de dépôt. Les dépôts récupérés dynamiquement n'étant pas dans la liste statique `GITHUB_REPOS`, cette vérification de format (et non une correspondance avec une liste statique) est utilisée pour les accepter.
- Le gestionnaire `top()` — assemble l'intégralité du HTML de la page TOP (accepte le paramètre `?repo=` via `Query<TopQuery>`)

## Déploiement

Exécute `cargo build --release` directement sur le VPS (ConoHa, AlmaLinux), liant le binaire résultant comme service systemd (`aruaru-tokyo-server.service`) sur `127.0.0.1:4100`. nginx (`/etc/nginx/conf.d/aruaru.tokyo.conf`) termine le TLS sur le port 443 et fait du reverse proxy via `location /` vers ce port.

**Emplacements miroirs `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/`** : ces chemins n'existent à l'origine que du côté `audiocafe.tokyo` (PHP) ; pour les rendre également visibles directement depuis `aruaru.tokyo`, des blocs `location` nginx dédiés font un proxy interne vers `http://127.0.0.1:80/` avec l'en-tête Host réécrit en `audiocafe.tokyo`. Ce binaire lui-même ne traite jamais ces chemins (seuls `/` et `/healthz` sont enregistrés dans `Route`).

## Projets associés

- [open-runo](https://github.com/aon-co-jp/open-runo) — le cœur de l'écosystème open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — la source de la convention d'implémentation Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — passerelle serveur web généraliste
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — couche base de données (ce dépôt en est indépendant)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — source canonique des règles de développement
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — serveur d'authentification OTP et de gestion de sites
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP) — le contenu réel derrière les emplacements miroirs

## Règles opérationnelles

- Ne pas introduire de fonctionnalités dépendantes d'une base de données ou de frameworks lourds dans ce dépôt.
- Toujours utiliser une URL publique pour les constantes de liens externes (une adresse loopback est inaccessible pour le navigateur des visiteurs).
- Ne jamais enregistrer l'adresse IP réelle du VPS dans le code ou la documentation.
- À chaque mise à jour de CLAUDE.md, mettre également à jour ces versions en 10 langues (`CLAUDE-<Langue>.md`) avec le même contenu et les pousser ensemble.

## État actuel

- 2026-07-15 : amorçage et déploiement en production terminés.
- 2026-07-16 : extensions d'intégration GitHub (lien vers l'organisation, liste de dépôts dynamique, rendu README façon GitHub, affichage pleine largeur) terminées.

## HANDOFF (journal de travail récent, le plus récent en premier)

- **2026-07-16** : Extension de l'intégration GitHub. Ajout d'un lien vers la page d'accueil de l'organisation GitHub, récupération dynamique de la dernière liste de dépôts (`GET /api/repos`), lien direct vers le dépôt sélectionné, rendu README façon GitHub via `pulldown-cmark` (basculable avec la conversion `.rs` existante), et affichage pleine largeur. Découvert pendant l'implémentation : `aon-co-jp` est un compte personnel, pas une Organisation, donc `/users/{name}/repos` est le bon point de terminaison. La validation des dépôts est aussi passée d'une correspondance avec une liste statique à une vérification de format.
- **2026-07-15** : réécriture de PHP vers Rust+Poem et déploiement en production terminés.

## Rôle de la couche serveur d'applications (open-runo / poem-cosmo-tauri, ajouté le 2026-07-16)

`open-web-server` a été ajouté comme option pour le « moteur de diffusion (vhost) », mais tant que `open-web-server` ne fonctionne pas encore comme un serveur web hybride Apache+Nginx complet, le rôle de couche de compatibilité façon Tomcat est joué par `open-runo` ou `poem-cosmo-tauri`.

Ceux-ci, combinés à `open-raid-z` et à une API sans version (VersionlessAPI), visent à fonctionner comme un framework/middleware soutenant pleinement le développement web critique, non-stop 24h/24 et 365 jours par an — pour des usages où aucune perte de données n'est tolérable sur Internet, comme les achats d'objets dans les jeux en ligne 3D, la finance en ligne, le courtage en ligne et les paiements par carte de crédit en ligne — en construisant un système de communication de pointe « à quatre couches, quadruple redondance » combinant `aruaru-db` (compatible ACID et ZFS) avec une configuration DUAL DATABASE aux côtés de PostgreSQL, sur une conception de base de données facilitant les changements de spécifications tout en maintenant un fonctionnement sans version aux côtés du contrôle de version et de la gestion Git.
