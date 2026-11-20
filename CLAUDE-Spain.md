# Política de desarrollo y reglas del entorno (aruaru-tokyo-server)

La unidad de trabajo es `F:\open-runo`. Esta sección considera el `CLAUDE.md` de [`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) como canónico, copiado y sincronizado a cada proyecto.

## Política de desarrollo y reglas del entorno (encabezado común a todos los repositorios, añadido el 2026-07-15)

### 1. Material de referencia para lenguajes/frameworks más recientes

Rust en sí tiene una larga historia, pero frameworks web relativamente nuevos y con poca documentación adoptados por este ecosistema, como [Poem](https://github.com/poem-web/poem), tienen muchos menos datos de entrenamiento, ejemplos publicados, Q&A y cobertura de blogs que combinaciones ampliamente adoptadas como Python+FastAPI. Esto hace que el desarrollo impulsado por IA (Claude, etc.) sea propenso a recordar mal las APIs, usar versiones obsoletas de API y otros patrones de fallo repetitivos que ya han ocurrido varias veces en este proyecto.

Como contramedida, antes de comenzar una tarea, la IA debería primero consultar solo la parte relevante de la siguiente tabla (no es necesario leerlo todo — hojear 1-2 entradas relevantes es suficiente). Esto debería mejorar el rendimiento y reducir el retrabajo impulsado por IA.

| Tecnología | Documentación oficial | GitHub | Notas/Blog |
|---|---|---|---|
| Lenguaje Rust | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem (framework web) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio (runtime asíncrono) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly (wasm32 en general) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **Aviso importante (divulgación honesta)**: esta lista de URL se escribió a partir de datos de entrenamiento en una sesión sin acceso a búsqueda web, y su existencia, validez actual y precisión no han sido verificadas. La IA (incluido Claude) debería evitar tomar esta lista al pie de la letra como base para implementación o respuestas — verifique accediendo usted mismo, o reconfirme las fuentes primarias en una sesión con búsqueda web disponible. Considere siempre la posibilidad de enlaces rotos, redirecciones y cambios de versión (especialmente cambios de API disruptivos). Añada nuevas tecnologías a esta tabla según se adopten.

### 2. Reflexiones sobre herramientas de desarrollo impulsadas por IA (2026-07-15, registrado como observación del usuario)

A partir del 2026-07-15, las herramientas de chat de IA de propósito general como ChatGPT pueden desarrollar aplicaciones web pequeñas, pero el retrabajo crece significativamente una vez que un sistema se vuelve moderadamente complejo/grande, y existe un límite estricto en el tamaño de programa manejable a la vez.

Claude Code / Claude Desktop puede leer/escribir archivos especificando directamente una unidad local, y también puede leer repositorios de GitHub (relevante para un ecosistema multi-repositorio como este proyecto), lo que lo hace muy adecuado para el desarrollo impulsado por IA a esta escala de proyecto. Recomendado como opción al configurar un nuevo entorno de desarrollo impulsado por IA.

## Rol de este repositorio

La página TOP de `aruaru.tokyo`. El 2026-07-15, la implementación previamente en PHP se reescribió en Rust + [Poem](https://github.com/poem-web/poem) (según instrucción del usuario: "aruaru.tokyo debería basarse en Rust+Poem"). `audiocafe.tokyo` permanece en PHP — un diseño intencional donde el stack difiere según el dominio.

## Stack tecnológico

- Rust + Poem (un framework web ligero basado en hyper). Sin dependencia de base de datos, binario único.
- Sin frameworks/ORM pesados (según la convención del ecosistema poem-cosmo-tauri).
- El frontend es HTML plano ensamblado en el servidor como cadenas (sin motor de plantillas). El JS se limita a un `<script>` incrustado en la página (comportamiento del botón shuffle).

## Módulos principales (archivo único `src/main.rs`)

- `categories()` — datos estáticos para el contenido "aruaru" (momentos cotidianos identificables)
- `render_related_sites()` / `RELATED_SITES` — enlaces a páginas relacionadas en audiocafe.tokyo
- `fetch_repo_file()` / `markdown_to_rs()` — obtiene el contenido crudo de GitHub y lo convierte a texto estilo `.rs` (funcionalidad readme-to-rs)
- `markdown_to_github_style_html()` — usa `pulldown-cmark` para renderizar realmente el README, etc., como HTML al estilo GitHub (visible mediante una pestaña alternable junto a la conversión `.rs`)
- `fetch_org_repos()` / el manejador `api_repos` (`GET /api/repos`) — obtiene cada vez la lista completa y actualizada de nombres de repositorios de `aon-co-jp` desde la API de GitHub. **Nota: `aon-co-jp` es una cuenta personal, no una Organización, por lo que debe usarse el endpoint `/users/{name}/repos`** (confirmado en infraestructura real que `/orgs/` devuelve 404). Sujeto al límite de tasa de la API de GitHub (60/hora/IP) ya que las llamadas no están autenticadas.
- `is_valid_repo_name()` — valida el formato del nombre del repositorio. Dado que los repositorios obtenidos dinámicamente no están en la lista estática `GITHUB_REPOS`, se usa esta verificación de formato (no una coincidencia con lista estática) para aceptarlos.
- El manejador `top()` — ensambla todo el HTML de la página TOP (acepta el parámetro `?repo=` mediante `Query<TopQuery>`)

## Despliegue

Ejecuta `cargo build --release` directamente en el VPS (ConoHa, AlmaLinux), vinculando el binario resultante como servicio systemd (`aruaru-tokyo-server.service`) en `127.0.0.1:4100`. nginx (`/etc/nginx/conf.d/aruaru.tokyo.conf`) termina TLS en el puerto 443 y hace proxy inverso mediante `location /` hacia este puerto.

**Ubicaciones espejo `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/`**: estas rutas originalmente solo existen en el lado de `audiocafe.tokyo` (PHP); para hacerlas visibles directamente también desde `aruaru.tokyo`, bloques `location` de nginx dedicados hacen proxy interno hacia `http://127.0.0.1:80/` con el encabezado Host reescrito a `audiocafe.tokyo`. Este binario nunca maneja estas rutas por sí mismo (solo `/` y `/healthz` están registrados en `Route`).

## Proyectos relacionados

- [open-runo](https://github.com/aon-co-jp/open-runo) — el núcleo del ecosistema open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — el origen de la convención de implementación Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — gateway de servidor web de propósito general
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — capa de base de datos (este repositorio es independiente de ella)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — fuente canónica de las reglas de desarrollo
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — servidor de autenticación OTP y gestión de sitios
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP) — el contenido real detrás de las ubicaciones espejo

## Reglas operativas

- No introducir funcionalidades dependientes de base de datos ni frameworks pesados en este repositorio.
- Usar siempre una URL pública para las constantes de enlaces externos (una dirección loopback es inalcanzable para los navegadores de los visitantes).
- Nunca registrar la dirección IP real del VPS en el código o la documentación.
- Cada vez que se actualice CLAUDE.md, actualizar también estas versiones en 10 idiomas (`CLAUDE-<Idioma>.md`) con el mismo contenido y hacer push juntos.

## Estado actual

- 2026-07-15 Bootstrap y despliegue en producción completados.
- 2026-07-16 Extensiones de integración con GitHub (enlace a la organización, lista dinámica de repositorios, renderizado de README estilo GitHub, visualización a ancho completo) completadas.

## HANDOFF (registro de trabajo reciente, más reciente primero)

- **2026-07-16**: Se extendió la integración con GitHub. Se añadió un enlace a la página principal de la organización de GitHub, obtención dinámica de la última lista de repositorios (`GET /api/repos`), enlace directo al repositorio seleccionado, renderizado de README estilo GitHub mediante `pulldown-cmark` (alternable con la conversión `.rs` existente), y visualización a ancho completo. Descubierto durante la implementación: `aon-co-jp` es una cuenta personal, no una Organización, por lo que `/users/{name}/repos` es el endpoint correcto. La validación de repositorios también cambió de una coincidencia con lista estática a una verificación de formato.
- **2026-07-15**: reescritura de PHP a Rust+Poem y despliegue en producción completados.

## Rol de la capa de servidor de aplicaciones (open-runo / poem-cosmo-tauri, añadido el 2026-07-16)

Se añadió `open-web-server` como opción para el "motor de entrega (vhost)", pero mientras `open-web-server` aún no funcione como un servidor web híbrido Apache+Nginx completo, el rol de capa de compatibilidad al estilo Tomcat lo desempeña `open-runo` o `poem-cosmo-tauri`.

Estos, junto con `open-raid-z` y una VersionlessAPI, buscan funcionar como un framework/middleware que respalde completamente el desarrollo web crítico, sin interrupciones las 24 horas del día, los 365 días del año — para usos donde nunca se puede tolerar la pérdida de datos en internet, como compras de artículos en juegos en línea 3D, finanzas en línea, corretaje de valores en línea y pagos con tarjeta de crédito en línea — construyendo un sistema de comunicación de vanguardia de "cuatro capas, cuádruple redundancia" que combina `aruaru-db` (compatible con ACID y ZFS) con una configuración DUAL DATABASE junto a PostgreSQL, sobre un diseño de base de datos que facilita los cambios de especificación mientras mantiene la operación sin versiones junto con el control de versiones y la gestión de Git.
