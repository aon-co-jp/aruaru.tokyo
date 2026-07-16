# 개발 방침 & 개발 환경 규칙 (aruaru-tokyo-server)

작업 드라이브는 `F:\open-runo`입니다. 이 절은 [`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)의 `CLAUDE.md`를 정본으로 삼아 각 프로젝트에 복사·동기화하는 방침에 따릅니다.

## 개발 방침·개발 환경 규칙 (전체 저장소 공통 헤더, 2026-07-15 추가)

### 1. 비교적 새로운 언어·프레임워크 참고 자료 목록

Rust 자체는 역사가 길지만, 이 생태계가 채택한 [Poem](https://github.com/poem-web/poem)과 같은 비교적 새롭고 정보량이 적은 웹 프레임워크는 Python+FastAPI처럼 널리 보급된 조합에 비해 AI 모델의 학습 데이터, 공개된 구현 예시/Q&A/블로그 글의 절대량이 적은 경향이 있습니다. 그로 인해 AI 기반 개발(Claude 등)이 이를 다룰 때 구현 착각·API 이름 착각·오래된 버전 API로의 구현(본 프로젝트에서 실제로 여러 번 발생한 알려진 실패 패턴)에 의한 재작업·다람쥐 쳇바퀴 현상이 일어나기 쉽습니다.

대책으로, AI가 작업을 시작할 때는 아래에서 해당 작업에 필요한 부분만 먼저 참조한 후 구현에 착수할 것(전부 읽을 필요는 없으며, 관련될 만한 1~2건을 훑어보는 정도로 충분함). 이를 통해 성공률이 높아지고 AI 기반 개발의 재작업이 줄어들 것으로 기대됩니다.

| 기술 | 공식 문서 | GitHub | 참고·블로그 등 |
|---|---|---|---|
| Rust 언어 본체 | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem(웹 프레임워크) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio(비동기 런타임) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly(wasm32 전반) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **중요한 주의사항(정직한 공개)**: 이 URL 목록은 웹 검색 도구가 없는 세션에서 학습 데이터를 기반으로 작성된 것으로, 실재 여부·현재 유효성·기재 내용의 정확성을 검증하지 않았습니다. 특히 AI(Claude 포함)가 이 목록을 그대로 믿고 구현이나 답변의 근거로 삼는 것은 피하고, 개발자 본인이 실제로 접속하여 확인하거나 웹 검색이 가능한 세션에서 1차 정보를 재확인한 후 사용할 것. 링크 끊김·리다이렉트·버전 변경(특히 API의 파괴적 변경) 가능성을 항상 고려할 것. 새로운 기술을 추가할 경우 이 표에 추가해 나갈 것.

### 2. AI 기반 개발 도구에 관한 소감 (2026-07-15, 사용자 소감으로 기록)

2026-07-15 시점, ChatGPT 등의 범용 AI 챗은 소규모 웹 앱 정도까지는 개발할 수 있지만, 시스템이 어느 정도 복잡·대규모가 되면 재작업이 커지고 한 번에 다룰 수 있는 프로그램 크기에도 곧 한계가 오는 경향이 있습니다.

Claude Code / Claude Desktop은 로컬 드라이브를 직접 지정하여 파일을 읽고 쓸 수 있고, GitHub 저장소 읽기(본 프로젝트와 같은 여러 저장소에 걸친 생태계)에도 대응할 수 있어, 본 프로젝트와 같은 규모의 AI 기반 개발에 적합하다고 판단됩니다. 새롭게 AI 기반 개발 환경을 구축할 때의 선택지로 추천합니다.

## 이 저장소의 역할

`aruaru.tokyo`의 TOP 페이지. 2026-07-15, 그때까지 PHP로 구현되어 있던 것을 Rust+[Poem](https://github.com/poem-web/poem)으로 다시 작성했습니다(사용자 지시: 「aruaru.tokyo는 Rust+Poem 기반으로 해 주세요」). `audiocafe.tokyo`는 계속 PHP로 유지—도메인마다 스택이 다른 의도적인 설계입니다.

## 기술 스택

- Rust + Poem(hyper 기반 경량 웹 프레임워크). DB에 의존하지 않고 단일 바이너리로 완결.
- 무거운 프레임워크·ORM은 사용하지 않음(poem-cosmo-tauri 생태계 규약에 준거).
- 프런트엔드는 서버 사이드에서 문자열로 조립한 순수 HTML(템플릿 엔진 미사용). JS는 페이지 내 `<script>`만 사용(shuffle 버튼 동작).

## 주요 모듈 (`src/main.rs` 단일 파일)

- `categories()` — 「아루아루」콘텐츠의 정적 데이터
- `render_related_sites()` / `RELATED_SITES` — audiocafe.tokyo 측 관련 페이지로의 링크
- `fetch_repo_file()` / `markdown_to_rs()` — GitHub raw content를 가져와 `.rs`풍으로 변환하는 readme-to-rs 기능
- `markdown_to_github_style_html()` — `pulldown-cmark`를 사용해 README 등을 GitHub 스타일로 실제 렌더링한 HTML로 변환(`.rs` 변환 표시와 탭으로 전환하여 양쪽 다 볼 수 있음)
- `fetch_org_repos()` / `api_repos` 핸들러(`GET /api/repos`) — `aon-co-jp`의 전체 저장소 이름을 GitHub API에서 매번 최신으로 가져옵니다. **주의: `aon-co-jp`는 Organization이 아닌 개인 계정이므로 `/users/{name}/repos` 엔드포인트를 사용해야 합니다**(`/orgs/`는 404가 됨을 실기에서 확인). 인증 없는 호출이므로 GitHub API의 속도 제한(60회/시간/IP)을 받습니다.
- `is_valid_repo_name()` — 저장소 이름의 형식 검증. 동적으로 가져온 저장소는 `GITHUB_REPOS`의 정적 목록에 포함되지 않으므로, 고정 목록과의 대조가 아닌 이 형식 검증으로 받아들입니다.
- `top()` 핸들러 — TOP 페이지 전체의 HTML 조립(`Query<TopQuery>`로 `?repo=` 파라미터를 받음)

## 배포

VPS(ConoHa, AlmaLinux)에서 직접 `cargo build --release`를 실행하고, 생성된 바이너리를 systemd 서비스(`aruaru-tokyo-server.service`)로 `127.0.0.1:4100`에 바인딩합니다. nginx(`/etc/nginx/conf.d/aruaru.tokyo.conf`)가 443번에서 TLS를 종료한 후 `location /`으로 이 포트로 리버스 프록시합니다.

**`/aruaru/`·`/aruaru-lady/`·`/rakuten-mobile/`의 미러 location**: 이 경로들은 원래 `audiocafe.tokyo` 측에만 존재하는 콘텐츠(PHP)로, `aruaru.tokyo`에서도 직접 열람할 수 있도록 nginx 측에서 `http://127.0.0.1:80/`(Host 헤더를 `audiocafe.tokyo`로 덮어씀)로 내부 프록시하는 location 블록을 개별로 추가했습니다. 이 바이너리 자체는 이 경로들을 전혀 처리하지 않습니다(`Route`에는 `/`와 `/healthz`만 등록되어 있음).

## 관련 프로젝트

- [open-runo](https://github.com/aon-co-jp/open-runo) — Rust→WASM/tokio+hyper 기반 open-runo 생태계 본체
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Poem/Tauri 구현 규약의 출처
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 범용 웹 서버 게이트웨이
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB 계층(이 저장소는 DB에 의존하지 않음)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 개발 규칙의 정본
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP 인증·사이트 관리 서버
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP) — 미러 location의 실체

## 운영 규칙

- 이 저장소에 DB 의존 기능·무거운 프레임워크를 들이지 말 것.
- 외부 링크 상수는 반드시 공개 URL을 사용할 것(루프백 주소는 방문자로부터 접근 불가).
- VPS의 실제 IP 주소를 코드·문서에 기록하지 말 것.
- CLAUDE.md를 업데이트할 경우, 이 10개국어 버전(`CLAUDE-<언어>.md`)도 같은 내용으로 업데이트하여 함께 push할 것.

## 현황

- 2026-07-15 부트스트랩·프로덕션 투입 완료.
- 2026-07-16 GitHub 연계 기능 확장(조직 링크·동적 저장소 목록 가져오기·GitHub풍 README 표시·전체 너비 표시) 완료.

## HANDOFF (최근 작업 로그, 위가 최신)

- **2026-07-16**: GitHub 연계 기능 확장. GitHub organization 최상단 페이지로의 링크, 최신 저장소 목록의 동적 가져오기(`GET /api/repos`), 선택 중인 저장소로의 직접 링크, `pulldown-cmark`에 의한 GitHub풍 README 표시(기존 `.rs` 변환과 전환 가능), 표시 영역의 전체 너비화를 구현. 구현 중 발견: `aon-co-jp`는 Organization이 아닌 개인 계정이므로 `/users/{name}/repos`가 올바른 엔드포인트. 저장소 검증도 정적 목록 대조에서 형식 검증으로 변경.
- **2026-07-15**: PHP에서 Rust+Poem으로의 재작성·VPS 프로덕션 투입 완료.

## 애플리케이션 서버 계층의 역할 (open-runo / poem-cosmo-tauri, 2026-07-16 추가)

「배포 엔진(vhost)」에 `open-web-server`를 선택지로 추가했지만, open-web-server가 Apache+Nginx의 하이브리드 사양 웹 서버로서 아직 기능하지 않는 동안은, Tomcat과 같은 호환 레이어로 기능하는 것은 `open-runo` 또는 `poem-cosmo-tauri`입니다.

이들은 `open-raid-z`와 VersionlessAPI에 의해 버전리스 운용과 버전 관리·Git 관리를 양립시키면서, ACID 호환성과 ZFS 호환성에 대응한 `aruaru-db`와 PostgreSQL과의 DUAL DATABASE 구성에 의한 「4층 4중」의 최첨단 통신 시스템을 구축하고, 사양 변경이 쉬운 데이터베이스 설계를 통해 3D 온라인 게임 AI 과금 아이템, 온라인 금융, 온라인 증권, 온라인 신용카드 결제 등 인터넷상에서 분실해서는 안 되는 미션 크리티컬한 용도를 위해 24시간 365일 논스톱 서버 대응 웹사이트 개발을 전면적으로 백업하는 프레임워크·미들웨어로서 기능하는 것을 목표로 합니다.
