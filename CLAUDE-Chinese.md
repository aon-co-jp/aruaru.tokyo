# 开发方针与开发环境规则 (aruaru-tokyo-server)

工作驱动器为`F:\open-runo`。本节以[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)的`CLAUDE.md`为权威版本，复制并同步到各个项目。

## 开发方针・开发环境规则(所有仓库共通头部，2026-07-15追加)

### 1. 较新语言/框架的参考资料一览

Rust本身历史悠久，但本生态系统采用的[Poem](https://github.com/poem-web/poem)这类相对较新、资料较少的Web框架，与Python+FastAPI这类广泛普及的组合相比，AI模型的训练数据、公开的实现示例/问答/博客文章的绝对数量较少。因此，AI驱动开发(Claude等)在处理这些技术时，容易出现实现误解・API名称记忆错误・使用旧版本API实现(本项目实际多次发生过的已知失败模式)所导致的返工・打地鼠现象。

作为对策，AI在开始工作前，应先只查阅下表中与该任务相关的部分(无需全部阅读，浏览1~2条相关内容即可)。这有望提高成功率，减少AI驱动开发的返工。

| 技术 | 官方文档 | GitHub | 补充・博客等 |
|---|---|---|---|
| Rust语言本体 | https://doc.rust-lang.org/book/ | https://github.com/rust-lang/rust | https://blog.rust-lang.org/ |
| Poem(Web框架) | https://docs.rs/poem/latest/poem/ | https://github.com/poem-web/poem | https://crates.io/crates/poem |
| Tokio(异步运行时) | https://tokio.rs/tokio/tutorial | https://github.com/tokio-rs/tokio | https://tokio.rs/blog |
| async-graphql | https://async-graphql.github.io/async-graphql/en/index.html | https://github.com/async-graphql/async-graphql | https://crates.io/crates/async-graphql |
| Tauri | https://tauri.app/ | https://github.com/tauri-apps/tauri | https://tauri.app/blog/ |
| wasm-bindgen / web-sys | https://rustwasm.github.io/wasm-bindgen/ | https://github.com/rustwasm/wasm-bindgen | https://rustwasm.github.io/docs/book/ |
| SurrealDB | https://surrealdb.com/docs | https://github.com/surrealdb/surrealdb | https://surrealdb.com/blog |
| sqlx | https://docs.rs/sqlx/latest/sqlx/ | https://github.com/launchbadge/sqlx | |
| WinFsp | https://winfsp.dev/ | https://github.com/winfsp/winfsp | |
| DirectX 12 / DirectML | https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide | https://github.com/microsoft/DirectML | https://devblogs.microsoft.com/directx/ |
| WebAssembly(wasm32全体) | https://webassembly.org/ | https://github.com/WebAssembly | https://rustwasm.github.io/docs/book/ |

⚠️ **重要提示(诚实披露)**: 此URL列表是在没有网络搜索工具的会话中根据训练数据编写的，其真实存在性・当前有效性・记载内容的准确性均未经验证。AI(包括Claude)应避免盲目相信此列表作为实现或回答的依据，应由开发者本人实际访问确认，或在可使用网络搜索的会话中重新核实一手信息。始终考虑链接失效・重定向・版本变更(尤其是破坏性API变更)的可能性。采用新技术时请在此表中补充记录。

### 2. 关于AI驱动开发工具的感想(2026-07-15，作为用户感想记录)

截至2026-07-15，ChatGPT等通用AI聊天工具能够开发小规模Web应用程序，但当系统达到一定复杂度・规模后，返工会显著增加，一次能处理的程序规模也很快达到上限。

Claude Code / Claude Desktop能够直接指定本地驱动器进行文件读写，也能读取GitHub仓库(适用于本项目这种跨多个仓库的生态系统)，因此被认为适合本项目这种规模的AI驱动开发。推荐作为搭建新的AI驱动开发环境时的选择之一。

## 本仓库的作用

`aruaru.tokyo`的TOP页面。2026-07-15，将此前用PHP实现的内容改写为Rust+[Poem](https://github.com/poem-web/poem)(用户指示："aruaru.tokyo请采用Rust+Poem架构")。`audiocafe.tokyo`继续保持PHP——这是按域名采用不同技术栈的有意设计。

## 技术栈

- Rust + Poem(基于hyper的轻量级Web框架)。不依赖数据库，单一二进制文件即可完成。
- 不使用重量级框架・ORM(遵循poem-cosmo-tauri生态系统的约定)。
- 前端为服务器端以字符串拼装的纯HTML(不使用模板引擎)。JS仅限于页面内的`<script>`(shuffle按钮的行为)。

## 主要模块(`src/main.rs`单一文件)

- `categories()` — 「Aruaru」内容的静态数据
- `render_related_sites()` / `RELATED_SITES` — 指向audiocafe.tokyo相关页面的链接
- `fetch_repo_file()` / `markdown_to_rs()` — 获取GitHub原始内容并转换为`.rs`风格文本的readme-to-rs功能
- `markdown_to_github_style_html()` — 使用`pulldown-cmark`将README等实际渲染为GitHub风格HTML的功能(可通过标签与`.rs`转换显示切换查看)
- `fetch_org_repos()` / `api_repos`处理器(`GET /api/repos`) — 每次从GitHub API获取`aon-co-jp`的最新完整仓库名称列表。**注意：`aon-co-jp`并非Organization而是个人账户，因此必须使用`/users/{name}/repos`端点**(已在实际环境中确认`/orgs/`会返回404)。由于调用未经身份验证，会受到GitHub API的速率限制(60次/小时/IP)。
- `is_valid_repo_name()` — 仓库名称的格式校验。由于动态获取的仓库不在静态列表`GITHUB_REPOS`中，因此采用此格式校验(而非与固定列表比对)来接受它们。
- `top()`处理器 — 组装整个TOP页面的HTML(通过`Query<TopQuery>`接收`?repo=`参数)

## 部署

在VPS(ConoHa、AlmaLinux)上直接运行`cargo build --release`，将生成的二进制文件作为systemd服务(`aruaru-tokyo-server.service`)绑定到`127.0.0.1:4100`。nginx(`/etc/nginx/conf.d/aruaru.tokyo.conf`)在443端口终止TLS后，通过`location /`反向代理到该端口。

**`/aruaru/`・`/aruaru-lady/`・`/rakuten-mobile/`的镜像location**：这些路径原本只存在于`audiocafe.tokyo`一侧的内容(PHP)中，为了让`aruaru.tokyo`也能直接查看，在nginx一侧单独添加了内部代理到`http://127.0.0.1:80/`(将Host头改写为`audiocafe.tokyo`)的location块。此二进制文件本身完全不处理这些路径(`Route`中仅注册了`/`和`/healthz`)。

## 相关项目

- [open-runo](https://github.com/aon-co-jp/open-runo) — open-runo生态系统的核心本体(Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Poem/Tauri实现约定的来源
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 通用Web服务器网关
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — 数据库层(本仓库不依赖数据库)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 开发规则的权威来源
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP认证・站点管理服务器
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP) — 镜像location的实体

## 运营规则

- 不要向本仓库引入依赖数据库的功能或重量级框架。
- 外部链接常量必须使用公开URL(回环地址对访问者的浏览器不可达)。
- 不要在代码・文档中记录VPS的真实IP地址。
- 每次更新CLAUDE.md时，也要以相同内容更新这10种语言版本(`CLAUDE-<语言>.md`)，并一同push。

## 现状

- 2026-07-15 引导启动・生产环境投入完成。
- 2026-07-16 GitHub集成功能扩展(组织链接・动态仓库列表获取・GitHub风格README显示・全宽显示)完成。

## HANDOFF(近期工作日志，最新在上)

- **2026-07-16**：扩展GitHub集成功能。添加了指向GitHub组织首页的链接、最新仓库列表的动态获取(`GET /api/repos`)、指向选中仓库的直接链接、通过`pulldown-cmark`实现的GitHub风格README显示(可与现有`.rs`转换切换)，以及全宽显示。实现过程中发现：`aon-co-jp`是个人账户而非Organization，因此`/users/{name}/repos`才是正确的端点。仓库校验也从静态列表比对改为格式校验。
- **2026-07-15**：从PHP改写为Rust+Poem并完成生产环境投入。

## 应用服务器层的作用(open-runo / poem-cosmo-tauri，2026-07-16追加)

虽然在「分发引擎(vhost)」中添加了`open-web-server`作为选项，但在open-web-server尚未成为Apache+Nginx混合规格的Web服务器之前，扮演类似Tomcat兼容层角色的是`open-runo`或`poem-cosmo-tauri`。

它们与`open-raid-z`及VersionlessAPI一起，在兼顾无版本运用与版本管理・Git管理的同时，通过与兼容ACID和ZFS的`aruaru-db`以及PostgreSQL构成的DUAL DATABASE配置，构建「四层四重」的最先进通信系统，凭借易于变更规格的数据库设计，力求成为一个能全面支持3D网络游戏内购道具、网络金融、网络证券、网络信用卡支付等互联网上绝不能丢失数据的关键任务用途、实现24小时365天不间断服务器支持的网站开发的框架・中间件。
