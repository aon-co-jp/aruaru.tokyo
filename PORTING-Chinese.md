# PORTING.md — 可迁移文件

可原样(或稍作修改)迁移到其他项目的实现模式一览。

## `markdown_to_rs()` (src/main.rs)

将Markdown每一行转换为带`//!`前缀的rustdoc注释风格的函数。可通用于README.md/CLAUDE.md/PORTING.md。可原样复制到采用readme-to-rs构想的其他仓库。

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

无需身份验证即可获取GitHub原始内容的辅助函数(带main→master回退)。只需更换组织名称・超时时间即可转用于其他GitHub集成功能。

## nginx「conf.d优先」模式

由于`/etc/nginx/conf.d/*.conf`在`nginx.conf`中比`/etc/nginx/sites-enabled/*.conf`更早被include，将带有相同`server_name`的配置放在`conf.d/`中，可使其优先于UI工具(如aruaru-easyweb)自动生成到`sites-enabled/`一侧的配置。当其他域名也发生类似冲突时可复用的方法(详见`CLAUDE.md`)。

## GitHub API：面向Organization的端点与面向个人账户的端点的区别

`GET /orgs/{name}/repos`仅适用于GitHub Organization，对个人账户(User)调用会返回404(已在实际环境中确认)。要获取个人账户的完整仓库列表，应使用`GET /users/{name}/repos`。`aon-co-jp`是个人账户，而非Organization。

```rust
// ✗ 对个人账户不适用(会返回404)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ 适用于个人账户
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

将GitHub集成功能移植到其他项目时，请事先通过`GET /users/{name}`(`"type"`字段是`"Organization"`还是`"User"`)确认目标账户类型，或考虑同时尝试两个端点的回退方案。

## `markdown_to_github_style_html()` + `pulldown-cmark`(src/main.rs)

将外部README实际渲染为GitHub风格以供显示的功能。可与`.rs`转换显示(`markdown_to_rs`)并用，通过标签实现切换的模式。

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

在显示端准备`.markdown-body`风格的CSS(标题下划线・代码块・表格边框等)可使外观更接近GitHub。

## 从狭窄的`main`容器中break-out(基于vw的居中)

当整个页面为了可读性需要保持较窄的`max-width`(例如780px)，但希望特定区域(如README显示)占满整个屏幕宽度时：

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

无论父容器的`max-width`如何，都可以创建一个以视口为基准居中的宽区域。

## 动态获取的列表应采用「格式校验」进行验证(不进行白名单比对)

当希望同时接受服务器启动时硬编码的列表(如`GITHUB_REPOS`)和运行时从API动态获取的列表时，若采用「与静态列表比对」进行输入校验，则仅存在于动态列表中的新值会被拒绝。应改为通过「格式是否正确」(仅限字母数字・连字符・下划线・点，长度限制等)进行校验。

## 「镜像location」模式(通过Host头重写实现内部代理)

当只想将某个域名下的特定路径内部代理到另一个域名的实际内容时：

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

在同一VPS上多个域名共存的架构中，若希望不复制内容即可从其他域名查看，可复用此方法。
