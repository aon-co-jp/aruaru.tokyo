# aruaru-tokyo-server

[aruaru.tokyo](https://aruaru.tokyo/)的TOP页面。使用Rust + [Poem](https://github.com/poem-web/poem)编写，不依赖数据库，单一二进制文件即可运行。

作为`audiocafe.tokyo`(PHP)的姊妹网站，位于不同的域名和技术栈上，按照poem-cosmo-tauri生态系统的约定(直接使用hyper/Poem，不依赖重量级框架或数据库)实现。

## 功能

- 「Aruaru」(人人都能感同身受的日常瞬间)内容，按体裁分为5个分类，另有随机显示
- 指向[aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)的快捷链接
- 指向GitHub(aon-co-jp)账号主页的链接
- 「🔄 获取最新仓库列表」按钮，可通过GitHub API动态更新仓库选项为最新列表
- 指向所选仓库GitHub页面的直接链接
- README.md、CLAUDE.md、PORTING.md既可以GitHub风格渲染的HTML显示，也可以rustdoc注释(`//!`)风格的`.rs`风文本显示，可通过标签切换(readme-to-rs构想的实现)。显示区域几乎占满整个屏幕宽度(94vw，最大1400px)

## 构建与启动

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

若未设置`ARUARU_TOKYO_BIND`，则默认监听`0.0.0.0:4100`。

## 生产环境配置(参考)

在VPS上作为systemd服务运行，绑定到`127.0.0.1:4100`，由nginx在443端口终止TLS后进行反向代理。`/aruaru/`、`/aruaru-lady/`、`/rakuten-mobile/`路径通过在同一nginx vhost中添加独立的location块，镜像到`audiocafe.tokyo`一侧的实际内容(PHP)(详见[CLAUDE.md](CLAUDE.md))。

## 相关项目

- [open-runo](https://github.com/aon-co-jp/open-runo) — open-runo生态系统的核心(Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — Poem/Tauri实现约定的来源
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — 通用Web服务器网关
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — 数据库层(本仓库不依赖数据库)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 开发规则的权威来源
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — OTP认证・站点管理服务器(tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo)(PHP)
