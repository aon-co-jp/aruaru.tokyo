# aruaru-tokyo-server

[aruaru.tokyo](https://aruaru.tokyo/)의 TOP 페이지. Rust + [Poem](https://github.com/poem-web/poem)으로 작성되었으며, DB 비의존·단일 바이너리로 완결됩니다.

`audiocafe.tokyo`(PHP)와는 다른 도메인·다른 스택을 가진 자매 사이트로, poem-cosmo-tauri 생태계의 규약(hyper/Poem을 직접 사용하고 무거운 프레임워크나 DB에 의존하지 않음)에 맞춰 구현되었습니다.

## 기능

- 「아루아루」(누구나 공감하는 일상 속 이야기) 콘텐츠, 장르별 5개 카테고리 + 랜덤 표시
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)로의 바로가기 링크
- GitHub(aon-co-jp) 계정 최상단 페이지로의 링크
- 「🔄 최신 리포지토리 목록 가져오기」버튼으로 GitHub API에서 최신 전체 리포지토리 이름을 가져와 선택 항목을 동적으로 갱신
- 선택한 리포지토리의 GitHub 페이지로의 직접 링크
- README.md·CLAUDE.md·PORTING.md를 GitHub 스타일로 렌더링한 HTML 표시와, rustdoc 주석(`//!`) 형식의 `.rs`풍 텍스트 표시 양쪽 모두 탭으로 전환하며 볼 수 있음(readme-to-rs 구상의 구현). 표시 영역은 화면 폭 대부분을 사용(94vw, 최대 1400px)

## 빌드 및 실행

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

`ARUARU_TOKYO_BIND`을 지정하지 않으면 `0.0.0.0:4100`에서 대기합니다.

## 프로덕션 구성(참고)

VPS에서는 systemd 서비스로 `127.0.0.1:4100`에 바인딩되며, nginx가 443번 포트에서 TLS를 종료한 뒤 리버스 프록시로 연결합니다. `/aruaru/`·`/aruaru-lady/`·`/rakuten-mobile/` 경로는 `audiocafe.tokyo` 측 실제 콘텐츠(PHP)로의 미러로서, 같은 nginx vhost 내에 개별 location 블록을 추가해 두었습니다(자세한 내용은 [CLAUDE.md](CLAUDE.md) 참조).

## 관련 프로젝트

- [open-runo](https://github.com/aon-co-jp/open-runo) — Rust→WASM/tokio+hyper 기반 open-runo 생태계 본체
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Poem/Tauri 구현 규약의 출처
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 범용 웹 서버 게이트웨이
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — DB 계층(이 저장소는 DB에 의존하지 않음)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 개발 규칙의 정본
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP 인증·사이트 관리 서버(tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP)
