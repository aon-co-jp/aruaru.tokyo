# aruaru-tokyo-server

Главная страница [aruaru.tokyo](https://aruaru.tokyo/). Написана на Rust + [Poem](https://github.com/poem-web/poem), без зависимости от базы данных, единый бинарный файл.

Сайт-побратим `audiocafe.tokyo` (PHP) на другом домене и с другим стеком, реализованный по соглашению экосистемы poem-cosmo-tauri (использовать hyper/Poem напрямую, без тяжёлых фреймворков и зависимости от БД).

## Возможности

- Контент «аруару» (повседневные моменты, понятные каждому) по 5 жанровым категориям, плюс случайный показ
- Быстрая ссылка на [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- Ссылка на главную страницу аккаунта GitHub (aon-co-jp)
- Кнопка «🔄 Получить актуальный список репозиториев», динамически обновляющая варианты репозиториев через GitHub API
- Прямая ссылка на страницу GitHub выбранного репозитория
- Просмотр README.md / CLAUDE.md / PORTING.md либо в виде HTML, отрендеренного в стиле GitHub, либо в виде текста `.rs` в стиле rustdoc-комментариев (`//!`), с переключением через вкладки (реализация концепции readme-to-rs). Область отображения занимает почти всю ширину экрана (94vw, до 1400px).

## Сборка и запуск

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

Если `ARUARU_TOKYO_BIND` не задан, прослушивается `0.0.0.0:4100`.

## Продакшн-конфигурация (справочно)

На VPS работает как служба systemd, привязанная к `127.0.0.1:4100`, при этом nginx завершает TLS на порту 443 и выполняет обратное проксирование. Пути `/aruaru/`, `/aruaru-lady/`, `/rakuten-mobile/` зеркалируются на реальный контент `audiocafe.tokyo` (PHP) через отдельные блоки `location` в том же vhost nginx (подробности см. в [CLAUDE.md](CLAUDE.md)).

## Связанные проекты

- [open-runo](https://github.com/aon-co-jp/open-runo) — ядро экосистемы open-runo (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — источник соглашения о реализации Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — универсальный шлюз веб-сервера
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — уровень базы данных (этот репозиторий от неё не зависит)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — канонический источник правил разработки
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — сервер OTP-аутентификации и управления сайтами (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)
