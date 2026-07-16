# PORTING.md — 이전 가능 파일

다른 프로젝트로 그대로(또는 약간의 수정으로) 가져갈 수 있는 구현 패턴 목록.

## `markdown_to_rs()` (src/main.rs)

Markdown의 각 줄을 `//!`가 붙은 rustdoc 주석 형식으로 변환하는 함수. README.md/CLAUDE.md/PORTING.md 어디에나 범용으로 사용 가능. readme-to-rs 구상을 채택하는 다른 저장소로 그대로 복사 가능.

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

GitHub raw content을 인증 없이 가져오는 헬퍼(main→master 폴백 포함). 조직 이름・타임아웃 값만 바꾸면 다른 GitHub 연계 기능에도 전용 가능.

## nginx 「conf.d 우선」패턴

`/etc/nginx/conf.d/*.conf`는 `/etc/nginx/sites-enabled/*.conf`보다 먼저 `nginx.conf`에서 include되므로, 같은 `server_name`을 가진 설정을 `conf.d/`에 두면 UI 도구(aruaru-easyweb 등)가 `sites-enabled/` 쪽에 자동 생성하는 설정보다 우선시할 수 있습니다. 다른 도메인에서도 같은 충돌이 발생할 경우 재사용 가능한 방법(자세한 내용은 `CLAUDE.md` 참조).

## GitHub API: Organization용 엔드포인트와 개인 계정용 엔드포인트의 차이

`GET /orgs/{name}/repos`는 GitHub Organization에만 사용할 수 있으며, 개인 계정(User)에 대해 호출하면 404가 반환됩니다(실기 확인 완료). 개인 계정의 전체 저장소 목록을 가져오려면 `GET /users/{name}/repos`를 사용할 것. `aon-co-jp`는 Organization이 아닌 개인 계정입니다.

```rust
// ✗ 개인 계정에는 사용 불가(404가 됨)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ 개인 계정용
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

GitHub 연계 기능을 다른 프로젝트로 이식할 때는, 대상 계정이 Organization인지 개인 계정인지를 `GET /users/{name}`(`"type"` 필드가 `"Organization"`인지 `"User"`인지)로 사전 확인하거나, 양쪽 엔드포인트를 모두 시도하는 폴백을 검토할 것.

## `markdown_to_github_style_html()` + `pulldown-cmark`(src/main.rs)

외부 README를 GitHub풍으로 실제 렌더링하여 표시하는 기능. `.rs` 변환 표시(`markdown_to_rs`)와 병용하여 탭으로 전환 가능하게 만드는 패턴.

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

표시할 곳의 CSS에 `.markdown-body`풍 스타일(제목 밑줄・코드 블록・테이블 테두리 등)을 준비하면 GitHub의 모습에 가까워집니다.

## 좁은 `main` 컨테이너로부터의 break-out(vw 기준 중앙 정렬)

페이지 전체는 가독성을 위해 좁은 `max-width`(예: 780px)로 유지하면서, 특정 섹션(README 표시 등)만 화면 너비 가득 넓히고 싶은 경우:

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

부모 컨테이너의 `max-width`와 무관하게, 뷰포트 기준으로 중앙 정렬된 넓은 섹션을 만들 수 있습니다.

## 동적으로 가져온 목록의 검증은 「형식 검증」으로 수행(화이트리스트 대조하지 않음)

서버 시작 시 하드코딩된 목록(`GITHUB_REPOS` 등)과, 실행 시 API에서 동적으로 가져온 목록을 모두 받아들이고 싶은 경우, 입력값의 타당성 검증을 「정적 목록과의 대조」로 수행하면 동적 목록에만 존재하는 새로운 값이 거부됩니다. 대신 「형식이 올바른가」(영숫자・하이픈・언더스코어・점만, 길이 제한 등)로 검증할 것.

## 「미러 location」패턴(Host 헤더 재작성을 통한 내부 프록시)

특정 도메인 하위의 특정 경로만 다른 도메인의 실제 콘텐츠로 내부적으로 프록시하고 싶은 경우:

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

동일한 VPS에 여러 도메인이 공존하는 구성에서, 콘텐츠를 복제하지 않고 다른 도메인에서도 열람 가능하게 하고 싶을 때 재사용할 수 있습니다.
