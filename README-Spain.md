# aruaru-tokyo-server

La página TOP de [aruaru.tokyo](https://aruaru.tokyo/). Escrita en Rust + [Poem](https://github.com/poem-web/poem), sin dependencia de base de datos, binario único.

Un sitio hermano de `audiocafe.tokyo` (PHP) en un dominio y stack diferentes, implementado según la convención del ecosistema poem-cosmo-tauri (usar hyper/Poem directamente, sin frameworks pesados ni dependencia de base de datos).

## Funciones

- Contenido "aruaru" (momentos cotidianos con los que todos se identifican) en 5 categorías, más visualización aleatoria
- Un enlace rápido a [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- Un enlace a la página principal de la cuenta de GitHub (aon-co-jp)
- Un botón "🔄 Obtener la lista más reciente de repositorios" que actualiza dinámicamente las opciones de repositorio mediante la API de GitHub
- Un enlace directo a la página de GitHub del repositorio seleccionado
- Visualización de README.md / CLAUDE.md / PORTING.md tanto como HTML renderizado al estilo GitHub como texto `.rs` al estilo comentario rustdoc (`//!`), alternable mediante pestañas (una implementación del concepto readme-to-rs). El área de visualización ocupa casi todo el ancho de la pantalla (94vw, hasta 1400px).

## Compilación y ejecución

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

Escucha en `0.0.0.0:4100` si `ARUARU_TOKYO_BIND` no está definido.

## Configuración de producción (referencia)

En el VPS se ejecuta como un servicio systemd vinculado a `127.0.0.1:4100`, con nginx terminando TLS en el puerto 443 y actuando como proxy inverso. Las rutas `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` se reflejan hacia el contenido real en `audiocafe.tokyo` (PHP) mediante bloques `location` dedicados en el mismo vhost de nginx (ver [CLAUDE.md](CLAUDE.md) para más detalles).

## Proyectos relacionados

- [open-runo](https://github.com/aon-co-jp/open-runo) — el núcleo del ecosistema open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — el origen de la convención de implementación Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — gateway de servidor web de propósito general
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — capa de base de datos (este repositorio es independiente de ella)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — fuente canónica de las reglas de desarrollo
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — servidor de autenticación OTP y gestión de sitios (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)
