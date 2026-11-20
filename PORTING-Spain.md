# PORTING.md — Archivos portables

Lista de patrones de implementación reutilizables tal cual (o con cambios menores) en otros proyectos.

## `markdown_to_rs()` (src/main.rs)

Una función que convierte cada línea de Markdown a estilo comentario rustdoc prefijado con `//!`. Utilizable de forma genérica para README.md/CLAUDE.md/PORTING.md. Se puede copiar tal cual en otros repositorios que adopten el concepto readme-to-rs.

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

Un helper que obtiene contenido crudo de GitHub sin autenticación (con respaldo main→master). Reutilizable para otras funciones de integración con GitHub simplemente cambiando el nombre de la organización y el valor de tiempo de espera.

## Patrón nginx de "prioridad conf.d"

`/etc/nginx/conf.d/*.conf` se incluye en `nginx.conf` antes que `/etc/nginx/sites-enabled/*.conf`, por lo que colocar una configuración con el mismo `server_name` en `conf.d/` le permite tener prioridad sobre las configuraciones auto-generadas por herramientas de UI (p. ej. aruaru-easyweb) en `sites-enabled/`. Técnica reutilizable siempre que surja el mismo conflicto en otro dominio (ver `CLAUDE.md` para más detalles).

## API de GitHub: diferencia entre endpoints de Organization y de cuenta personal

`GET /orgs/{name}/repos` solo funciona para Organizaciones de GitHub; llamarlo para una cuenta personal (User) devuelve 404 (confirmado en infraestructura real). Para obtener la lista completa de repositorios de una cuenta personal, use en su lugar `GET /users/{name}/repos`. `aon-co-jp` es una cuenta personal, no una Organización.

```rust
// ✗ No funciona para una cuenta personal (devuelve 404)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ Correcto para una cuenta personal
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

Al portar funciones de integración con GitHub a otros proyectos, verifique de antemano mediante `GET /users/{name}` si el campo `"type"` es `"Organization"` o `"User"`, o considere un respaldo que pruebe ambos endpoints.

## `markdown_to_github_style_html()` + `pulldown-cmark` (src/main.rs)

Una función que realmente renderiza un README externo al estilo GitHub para su visualización. Un patrón para combinarlo con la vista de conversión `.rs` (`markdown_to_rs`) y hacerlos alternables mediante pestañas.

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

Preparar un CSS al estilo `.markdown-body` en el destino de visualización (subrayado de encabezados, bloques de código, bordes de tabla, etc.) acerca la apariencia a la de GitHub.

## Salir de un contenedor `main` estrecho (centrado basado en vw)

Cuando toda la página debe permanecer estrecha por legibilidad (p. ej. `max-width: 780px`), pero una sección específica (como la visualización de un README) debe expandirse a todo el ancho de la pantalla:

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

Esto crea una sección ancha centrada respecto al viewport, independientemente del `max-width` del contenedor padre.

## Validar listas obtenidas dinámicamente mediante verificación de formato (no mediante coincidencia con lista blanca)

Cuando se desea aceptar tanto una lista codificada de forma fija al iniciar el servidor (p. ej. `GITHUB_REPOS`) como una lista obtenida dinámicamente de una API en tiempo de ejecución, validar la entrada mediante "coincidencia con la lista estática" rechazará valores nuevos que solo existen en la lista dinámica. En su lugar, valide mediante "el formato es correcto" (solo alfanumérico/guion/guion bajo/punto, límites de longitud, etc.).

## El patrón de "ubicación espejo" (proxy interno mediante reescritura del encabezado Host)

Cuando se desea hacer proxy internamente solo de rutas específicas bajo un dominio hacia el contenido real de otro dominio:

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

Reutilizable siempre que coexistan varios dominios en el mismo VPS y se desee que el contenido sea visible desde otro dominio sin duplicarlo.
