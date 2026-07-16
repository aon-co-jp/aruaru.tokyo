# aruaru-tokyo-server

La page TOP de [aruaru.tokyo](https://aruaru.tokyo/). Écrit en Rust + [Poem](https://github.com/poem-web/poem), sans dépendance à une base de données, binaire unique.

Un site jumeau de `audiocafe.tokyo` (PHP) sur un domaine et une pile technique différents, implémenté selon la convention de l'écosystème poem-cosmo-tauri (utiliser hyper/Poem directement, sans framework lourd ni dépendance à une base de données).

## Fonctionnalités

- Contenu « aruaru » (moments du quotidien auxquels tout le monde peut s'identifier) réparti en 5 catégories, avec affichage aléatoire
- Un lien rapide vers [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- Un lien vers la page d'accueil du compte GitHub (aon-co-jp)
- Un bouton « 🔄 Récupérer la dernière liste de dépôts » qui met à jour dynamiquement les choix de dépôts depuis l'API GitHub
- Un lien direct vers la page GitHub du dépôt sélectionné
- Affichage de README.md / CLAUDE.md / PORTING.md soit en HTML rendu façon GitHub, soit en texte `.rs` façon commentaire rustdoc (`//!`), basculable par onglets (implémentation du concept readme-to-rs). La zone d'affichage occupe presque toute la largeur de l'écran (94vw, jusqu'à 1400px).

## Compilation et exécution

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

Écoute sur `0.0.0.0:4100` si `ARUARU_TOKYO_BIND` n'est pas défini.

## Configuration de production (référence)

Sur le VPS, il fonctionne comme un service systemd lié à `127.0.0.1:4100`, avec nginx qui termine le TLS sur le port 443 et fait du reverse proxy. Les chemins `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` sont mis en miroir vers le contenu réel sur `audiocafe.tokyo` (PHP) via des blocs `location` dédiés dans le même vhost nginx (voir [CLAUDE.md](CLAUDE.md) pour les détails).

## Projets associés

- [open-runo](https://github.com/aon-co-jp/open-runo) — le cœur de l'écosystème open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — la source de la convention d'implémentation Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — passerelle serveur web généraliste
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — couche base de données (ce dépôt en est indépendant)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — source canonique des règles de développement
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — serveur d'authentification OTP et de gestion de sites (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)
