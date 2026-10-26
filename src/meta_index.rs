//! `/open-aruaru-runo-iLumi`(エイリアス`/open-aruaru-runo`)ページ。
//!
//! `F:\open-runo`エコシステム全体のメタ索引リポジトリ
//! [`aon`](https://github.com/aon-co-jp/aon)(旧称: `open-aruaru-runo-iLumi`、
//! 2026-07-20にユーザーがリポジトリ名を変更)と同じ内容(プロジェクト一覧・
//! README/PORTING.md/CLAUDE.mdへのリンク)を
//! Webページとして表示する。GitHubのリポジトリファイル一覧のような
//! 見た目(📄アイコン+ファイル名+リンクのミニリスト)で各プロジェクトの
//! ファイルを提示し、クリック時にGitHub APIから最新情報(⭐Star数・
//! 最終更新日時・既定ブランチ)を取得するボタンを添える。
//!
//! 13ヶ国語対応(`?lang=`)・言語別UIについては`crate::i18n`参照。
//!
//! **正直な開示**: 各プロジェクトの役割説明(`ROLE_JA`)は日本語のみで、
//! 13言語へは翻訳していない(スコープの限界、`crate::i18n`冒頭コメント
//! 参照)。

use crate::html_escape;
use crate::i18n::{index_strings, Lang};

/// メタ索引リポジトリ`aon`(旧称: `open-aruaru-runo-iLumi`)のREADME.mdに掲載した
/// プロジェクト一覧と全く同じ内容(名称・GitHubリポジトリ・各ファイルの
/// 有無・役割の要約)。`F:\open-runo`配下で実際にgitリポジトリとして
/// 存在し、`.git/config`のリモートURLで`aon-co-jp`上の実在を確認した
/// プロジェクトのみを掲載している(推測での追加はしていない)。
pub struct Project {
    pub name: &'static str,
    /// `aon-co-jp/<repo>`形式(GitHub APIの`owner/repo`)。
    pub repo: &'static str,
    pub has_readme: bool,
    pub has_porting: bool,
    pub has_claude: bool,
    pub role_ja: &'static str,
}

pub const PROJECTS: &[Project] = &[
    Project {
        name: "RCosmo",
        repo: "aon-co-jp/RCosmo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rust製GraphQL Federationプラットフォーム(Poem/Tauri/Cosmoは非依存・互換自前実装)。WunderGraph Cosmoの有料版機能をOSS・Pure Rustで実現し、独自の自己学習AIを搭載(外部LLM契約不要)。姉妹リポジトリpoem-cosmo-tauriと並行開発。",
    },
    Project {
        name: "RFrontEnd",
        repo: "aon-co-jp/RFrontEnd",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "HTML5/CSS3/TypeScript/React相当を、既存実装のコードを一切流用せず一から開発する複数プロジェクト(RHTML/RCSS/RTypeScript等)を束ねる親リポジトリ。",
    },
    Project {
        name: "RPoem",
        repo: "aon-co-jp/RPoem",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "RCosmoと同種のRust製GraphQL Federationプラットフォーム(Poem/Tauri/Cosmo非依存の互換自前実装)。open-runoを正本として分岐したpoem-runoをさらにリネーム・統合した後継リポジトリ。",
    },
    Project {
        name: "aruaru-db-archive(非公開)",
        repo: "aon-co-jp/aruaru-db-archive",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "aruaru-db(VPS高速キャッシュ層)から古くなったデータをGit-on-SQL経由で自動バックアップする非公開アーカイブ先。open-LiveKitのノードルーティング状態管理(DUAL DB構成)で使用予定。2026-09-26時点は雛形のみ、自動同期の実装は未着手。",
    },
    Project {
        name: "open-github-backup(非公開)",
        repo: "aon-co-jp/open-github-backup",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "aon-co-jpの全リポジトリを GitHub Actions でruno-aruaru(非公開)へ自動ミラーするバックアップ。RS-Syncの同期とVPS cronを置き換えた。2026-10-08に全件成功、以降はGitHub課金/利用上限でジョブ未開始(要Billing確認、残る課題はこれのみ)。VPSデータの暗号化バックアップ(毎日03:30)も稼働中。",
    },
    Project {
        name: "backup-to-discord-by-github",
        repo: "aon-co-jp/backup-to-discord-by-github",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "GitHubのリポジトリ(公開・非公開)をgit bundleにしてクライアント側で暗号化・分割し、非公開のDiscordサーバーへ保存/復元するバックアップ(Rust+RPoem)。CLIbdg(doctor/backup/list/restore/prune/run)と、管理API+GitHub Webhook受信のbdg-server。2026-10-26に本物のDiscordで往復を実機確認(公開repo1つ)。非公開repo・大容量・easy-web.tokyo配置は未了。Discordの規約リスクがあるため唯一のバックアップにはしない。RPoemにwebhook/body_limit/multipart_streamを同時追加。",
    },
    Project {
        name: "aruaru-db",
        repo: "aon-co-jp/aruaru-db",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "CockroachDBの分散強整合×Snowflakeのストレージ/コンピュート分離×Git-on-SQLバージョン管理を、すべてPure Rustで実装するハイブリッド分散データベース。",
    },
    Project {
        name: "aruaru-llm",
        repo: "aon-co-jp/aruaru-llm",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "aruaruエコシステム(aruaru-tokyo・aruaru-db・e-gov.info・karu.tokyo等)共通の「AIチャットコマース」応答サービス。リポジトリ名は「LLM」を冠するが、実際のLLM推論への差し替えは今後という正直な開示がREADMEに明記されている。",
    },
    Project {
        name: "aruaru-search",
        repo: "aon-co-jp/aruaru-search",
        has_readme: true,
        has_porting: false,
        has_claude: true,
        role_ja: "Rust製の自前メタ検索(APIキー不要・VPSで完全無料)。Bing・Brave・Yahoo! JAPAN等の公開ページを解析・統合し、世界約130言語で英語圏に偏らないよう並べ替える。検索元の変更はaruaru-llmの無料AIで毎朝自動保守し、aruaru-llmの検索の第一候補として連動する。",
    },
    Project {
        name: "aruaru-tokyo",
        repo: "aon-co-jp/aruaru-tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "aruaru.tokyoのTOPページ。Rust + Poem製、DB非依存・1バイナリ完結。audiocafe.tokyo(PHP)とは別ドメイン・別スタックの姉妹サイト。",
    },
    Project {
        name: "aruaru.tokyo",
        repo: "aon-co-jp/aruaru.tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "aruaru.tokyoのTOPページ(このリポジトリ自身)。Rust + Poem製、DB非依存・1バイナリ完結。audiocafe.tokyo(PHP)とは別ドメイン・別スタックの姉妹サイト。",
    },
    Project {
        name: "aruaru-vpn",
        repo: "aon-co-jp/aruaru-vpn",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Outline VPN/Algo VPNを参考に、コードは流用せず一からRust+open-web-serverで再実装する、利用者が自分で借りたVPS上に自分専用のプライバシー中継を立てられるOSSテンプレート。aon-co-jpは中継サーバーを運営しない(用途を隠さない透明性方針)。2026-09-27時点は設計ドキュメント段階、実装は未着手。",
    },
    Project {
        name: "aruaru-IME",
        repo: "aon-co-jp/aruaru-IME",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "画面の文脈から入力モードを予測し、漢字に変換することを前提に打つそばから変換結果を出す AI IME(Windows、2026-10-12 v0.2.0 アルファ公開、オンライン画面 https://easy-web.tokyo/ime)。Mozc OSS 辞書+Viterbi、日本語⇔英語の相互翻訳(JMdict)、単語・熟語・用語辞書(検索で作成)、プログラミング言語/フレームワークの補完と訂正、Google ドライブへの自動バックアップ。aruaru-llm・open-cuda・open-easy-web と連携。中国語(ピンイン)・季節用語・優先順位の調整・世界132言語の一覧に対応。準備中言語の入力エンジン・macOS/Linux/Android のIME本体は未了。",
    },
    Project {
        name: "audiocafe-tokyo-rust",
        repo: "aon-co-jp/audiocafe-tokyo-rust",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "audiocafe.tokyoの既存PHPモノリスをRust + Poemへ段階的に移行するプロジェクト(第一段)。既存PHP実装はaudiocafe-tokyoリポジトリのまま並行運用。",
    },
    Project {
        name: "audiocafe.tokyo",
        repo: "aon-co-jp/audiocafe-tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "PHP製のマルチコンテンツサイト。IT/建築系求人情報(aruaru)・女性向け求人/夜間エンターテインメント情報(aruaru-lady)・楽天モバイル関連情報・会社案内などを扱う。",
    },
    Project {
        name: "e-gov.info",
        repo: "aon-co-jp/e-gov",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "行政のデジタル化と、個人〜貿易商社まで対応するオンライン貿易・不動産プラットフォームを、LINEアプリ・WEBサイト・コンビニ端末という複数の入口から利用できる形で統合するプロジェクト。まだサンプル・デモンストレーション段階(READMEに明記)。",
    },
    Project {
        name: "karu.tokyo",
        repo: "aon-co-jp/karu-tokyo",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "karu.tokyoのTOPページ。Rust + Poem製、DB非依存の1バイナリ完結サーバー。軽井沢・あきる野市・東京の観光とリモートワーク、IT・AI・AUDIO・貿易産業を紹介。",
    },
    Project {
        name: "open-audio-sr",
        repo: "aon-co-jp/open-audio-sr",
        has_readme: true,
        has_porting: false,
        has_claude: true,
        role_ja: "AudioSR(実在する拡散モデル、コードMIT・重みApache-2.0)による音声帯域拡張(超解像)をRust+RPoemでラップ。ライブラリ呼び出し・HTTP API双方で実機検証済み(単純補間との定量比較で高域エネルギー生成を確認)。",
    },
    Project {
        name: "open-av",
        repo: "aon-co-jp/open-av",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "MP4/MKV映像+DSD音声+イマーシブ配置メタデータを1ファイルにまとめるオープンな映像コンテナ仕様。音声専用プロファイルはopen-mqa-dsdへ2026-09-26に統合済み、本体は映像プロファイル専用の薄いクレート。",
    },
    Project {
        name: "open-bar",
        repo: "aon-co-jp/open-bar",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "foobar2000をリスペクトした高音質・高画質プレーヤー(共有/排他/DoP再生モード、DSD対応、open-mqa-dsdのopen-audioコンテナ再生対応)。",
    },
    Project {
        name: "open-cuda",
        repo: "aon-co-jp/open-cuda",
        has_readme: false,
        has_porting: false,
        has_claude: false,
        role_ja: "OmniGPU設計文書(OmniGPU-Design.md)等を含むGPUランタイム関連プロジェクト(現時点で標準構成のREADME.md/CLAUDE.md/PORTING.mdは未整備)。",
    },
    Project {
        name: "open-directx",
        repo: "aon-co-jp/open-directx",
        has_readme: false,
        has_porting: false,
        has_claude: false,
        role_ja: "ローカルドライブには実体(git clone)が無く、GitHub上に空リポジトリとして存在するのみを確認(git ls-remoteで疎通確認、refなし)。DirectX関連プロジェクトと推測されるが、内容が無いため役割は未確認・推測で埋めない。",
    },
    Project {
        name: "open-mqa-dsd",
        repo: "aon-co-jp/open-mqa-dsd",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "DSD(DSF/DSDIFF)読み込み・DSD⇄PCM変換・DoPパッキングのRust実装(open-mqaのDSD版相棒、MQA互換ではない)。2026-09-26にopen-avの音声専用コンテナ実装(containerモジュール、旧称open-audio)を統合。",
    },
    Project {
        name: "open-music-llm",
        repo: "aon-co-jp/open-music-llm",
        has_readme: true,
        has_porting: false,
        has_claude: true,
        role_ja: "MusicGen(実在するオーディオLLM、EnCodecトークン+Transformer次トークン予測)をRust+RPoemでラップしテキストから音楽・効果音を生成する。2026-09-26時点は環境構築のみで実際の生成は未検証、学習済み重みはCC-BY-NC 4.0(非商用限定)。",
    },
    Project {
        name: "open-easy-web",
        repo: "aon-co-jp/open-easy-web",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "「第二のKUSANAGI」——アプリのアップロード後にIPアドレスで起動し、ドメイン登録・HTTPS化を簡単に自動適用できる運用ツール(Rust → WebAssembly、フレームワーク不使用)。",
    },
    Project {
        name: "open-LiveKit",
        repo: "aon-co-jp/open-LiveKit",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "LiveKit(Go+Pion製)のSFUアーキテクチャを参考に、コードは流用せず一からRust+RPoemで再実装するWebRTC SFU/リアルタイム通信基盤。open-tv-chatのリレーサーバーとして使う。2026-09-26時点は設計ドキュメント段階、実装は未着手。",
    },
    Project {
        name: "open-english",
        repo: "aon-co-jp/open-english",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "【2026-10-21 maid-cafe-english へ改名・お引越し。この旧名のリポジトリも永久に残し、リンク・リリース・自動更新はそのまま使える】ブラウザ製の静的フロントエンド+aruaru-llmのローカル常駐サーバーを組み合わせた「PC/Android向け英会話AI学習アプリ」。オンライン/オフライン両対応のハイブリッド構成、Windows/Android用インストーラー・アンインストーラー・バージョン管理付き。",
    },
    Project {
        name: "maid-cafe-english",
        repo: "aon-co-jp/maid-cafe-english",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "open-english の改名・お引越し先(2026-10-21、同一の履歴・内容)。更新は新旧の両方で配布する(アプリは両方を確認して新しい版を採用)。経緯・多言語の告知は旧リポジトリのRENAMED.md。",
    },
    Project {
        name: "open-raid-z",
        repo: "aon-co-jp/open-raid-z",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rust製の実マウント可能なRAID-Z/Z2/Z3ストレージプール実装(ZFS「風」のCoW/チェックサム/スナップショット。ZFS自体・OpenZFSへの依存やオンディスク互換性はなし)。エコシステム開発ルールの正本リポジトリ。",
    },
    Project {
        name: "maid-cafe-se",
        repo: "aon-co-jp/maid-cafe-se",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "maid-cafeによる秘書サービス(secretary)。Androidスマホ向け。定型スケジュール(毎日/平日/土日祝/曜日/第N週/N週ごと)で著作権フリー音のアラーム、または入力文章をメイドカフェ風/低い男性声で読み上げ。Googleカレンダー連動+30分前予告。2026-09-30時点でv0.4.0(Android署名つきAPK+Windows版インストーラーmaid-cafe-se-installer.exe、installer/に入手・ビルド方法)。アプリ本体のRust(+RPoem)化を進行中。",
    },
    Project {
        name: "open-tv-chat",
        repo: "aon-co-jp/open-tv-chat",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Skype風「世界約130ヶ国語リアルタイム音声翻訳」対応のTV会議/ビデオチャットアプリ構想(Windows/macOS/Linux/Android/iPhone向け)。open-web-server+Rust/RPoem(Cosmo互換)構成。2026-09-26時点は設計ドキュメント段階、実装は未着手。",
    },
    Project {
        name: "open-vsc-claude",
        repo: "aon-co-jp/open-vsc-claude",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "言語・フレームワーク(GraphQL+WunderGraph Cosmo/RPoem、TypeScript+Node+REST、または言葉で自由指定)を選んで生成コードをプレビューし、Claudeで改良して保存できるVS Code拡張。Web拡張(vscode.dev等)とブラウザデモ(PWA・オフライン可、easy-web.tokyo/open-vps-claude)付き、自動更新確認、README/CLAUDE/PORTINGは12言語。2026-10-18時点でv0.1.1。Marketplace公開・Cosmo Router実起動・実機iOS/Androidは未了。",
    },
    Project {
        name: "360-speaker",
        repo: "aon-co-jp/360-speaker",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "360度全方位5WAYトールボーイスピーカー(1〜5WAY・通常/スーパーツィーター/サブウーファーの3版、アンプ内蔵またはアンプ無し+別売プリ)の設計案・調査・製品比較と、Rust→WASMのWebシミュレーター(指向性・T/S箱設計・ポート・壁材・ボリューム方式別コスト)。受動ネットワーク設計図(PDF/JPG)付き。公開: easy-web.tokyo/360-speaker。2026-10-25時点で設計初版。実機試作・実測は未了、コストは推定値。",
    },
    Project {
        name: "version-control",
        repo: "aon-co-jp/version-control",
        has_readme: true,
        has_porting: false,
        has_claude: false,
        role_ja: "全リポジトリが最新のライブラリ(crates.io/npm/packagist)・クローンを使っているかを走査し、open-cuda/aruaru-llm/open-cpu/open-directxの自動保守で足りないものを、Claude向けの貼り付けメッセージにしてメールとLINE(Messaging API)で通知。依存の書き換え・git pullはしない。運用: easy-web.tokyo/version-control、デモ: /demo/。2026-10-26時点で初版・単体テスト・画面表示・VPS配置を確認、メール/LINE送信は未確認(宛先・トークン未設定)、VPSには配置済み。",
    },
    Project {
        name: "open-web-server",
        repo: "aon-co-jp/open-web-server",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "Rust + tokio/hyper自前実装のWebサーバー — 課金アイテム・金融データを「消失させない」ために設計。open-runo・aruaru-dbと4層防御通信で連携するミッションクリティカル向け。",
    },
    Project {
        name: "realdata.pro",
        repo: "aon-co-jp/realdata.pro",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "SAS Viyaを参考にした、誰でも使えるRust製オープンソースのデータ分析プラットフォーム(旧名rs-real-data)。open-cuda・RPoem・open-web-server・aruaru-llm等のエコシステム上に構築し、CSV・検索ワード(Google/YouTube/GitHub)・URLからの取り込み、クレンジング・集計・グラフ・重回帰、AIによる多言語説明を提供する。公開予定ドメインはrealdata.pro。",
    },
    Project {
        name: "webeditor-sftp",
        repo: "aon-co-jp/webeditor-sftp",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "ブログ感覚のWEB文章編集エディター(HTMLタグ非表示のブログモード/生コード編集のコーディングモード切替)と、秘密鍵・公開鍵を安全に受け渡しできるSFTPツールをハイブリッド統合したクロスプラットフォームアプリ(Tauri採用、Android/Windows/将来のiOS・macOS/Linux対応)。最初の実地検証対象はaudiocafe.tokyo(Rust+Poem)。2026-09-28時点は設計ドキュメント+最小プロトタイプ段階。",
    },
    Project {
        name: "rs-to-readme",
        repo: "aon-co-jp/rs-to-readme",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "RustクレートのCargo.tomlメタデータからREADME.mdを自動生成するCLIツール(crates.io公開)。",
    },
    Project {
        name: "RS-JSON",
        repo: "aon-co-jp/RS-JSON",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "RustのJSON処理ライブラリ。寛容モード(JSON5風)・厳密モード・部分抽出に加え、信頼できない入力向けの安全モードparse_secure(サイズ・深さ上限、重複キー拒否)と、速い値の型で読むparse_secure_sonic(fast機能、実測でserde_jsonの約2.4倍速い)を提供。旧sonic-rs(評価記録)を統合済み。型付き読み書きfast::from_str等と、RS-JSON<->JSON変換(最大約7倍速い)も提供。",
    },
    Project {
        name: "rs-dpiAwareness",
        repo: "aon-co-jp/rs-dpiAwareness",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "WindowsのDPI認識(PerMonitorV2)を依存ゼロのRustで扱う小さなクレート+診断CLI(段階的な代替つきのinit()、モニターごとのDPI、WM_DPICHANGED、マニフェストの生成・埋め込み・読み出し)。Microsoftの公開仕様へのリスペクト版で、Microsoftとは無関係。実機(Windows 11・4K 200%)で確認済み、複数モニター間の移動は未確認。",
    },
    Project {
        name: "RUNO(このリポジトリ、旧称: open-aruaru-runo-iLumi → aon)",
        repo: "aon-co-jp/RUNO",
        has_readme: true,
        has_porting: true,
        has_claude: true,
        role_ja: "このエコシステム全体の「プロジェクトシリーズ索引」を担うメタリポジトリ。個別のコード実装は持たない。",
    },
];

fn file_line(repo_url: &str, filename: &str, label: &str, present: bool, none_label: &str) -> String {
    if present {
        format!(
            r#"<li>📄 <a href="{repo_url}/blob/main/{filename}" target="_blank" rel="noopener">{label}</a></li>"#,
            repo_url = repo_url,
            filename = filename,
            label = html_escape(label),
        )
    } else {
        format!(r#"<li>📄 <span class="file-missing">{label} {none}</span></li>"#, label = html_escape(label), none = html_escape(none_label))
    }
}

fn render_project_card(p: &Project, s: &crate::i18n::IndexStrings) -> String {
    let repo_url = format!("https://github.com/{}", p.repo);
    let files = format!(
        "{}\n      {}\n      {}",
        file_line(&repo_url, "README.md", s.label_readme, p.has_readme, s.label_none),
        file_line(&repo_url, "PORTING.md", s.label_porting, p.has_porting, s.label_none),
        file_line(&repo_url, "CLAUDE.md", s.label_claude, p.has_claude, s.label_none),
    );
    format!(
        r#"<article class="repo-card" data-repo="{repo}">
  <h3><a href="{repo_url}" target="_blank" rel="noopener">📦 {name}</a></h3>
  <p class="role">{role}</p>
  <ul class="file-list">
      {files}
  </ul>
  <div class="live-fetch-row">
    <button type="button" class="live-fetch-btn" data-repo="{repo}">{btn_label}</button>
    <span class="live-fetch-result" data-repo-result="{repo}"></span>
  </div>
</article>"#,
        repo = html_escape(p.repo),
        repo_url = repo_url,
        name = html_escape(p.name),
        role = html_escape(p.role_ja),
        files = files,
        btn_label = html_escape(s.btn_fetch_live),
    )
}

fn render_lang_switcher(current: Lang, path: &str) -> String {
    Lang::ALL
        .iter()
        .map(|&l| {
            let cur = if l == current { " class=\"current\"" } else { "" };
            format!(r#"<a href="{path}?lang={code}"{cur}>{name}</a>"#, path = path, code = l.code(), name = html_escape(l.native_name()))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// `/open-aruaru-runo-iLumi`ページ本体のHTMLを組み立てる。
pub fn render_page(lang: Lang, canonical_path: &str) -> String {
    let s = index_strings(lang);
    let dir = if lang.is_rtl() { "rtl" } else { "ltr" };
    let cards = PROJECTS.iter().map(|p| render_project_card(p, &s)).collect::<Vec<_>>().join("\n  ");
    let lang_switch = render_lang_switcher(lang, canonical_path);
    let page_data_json = serde_json::json!({
        "loading": s.fetch_loading,
        "stars": s.field_stars,
        "updated": s.field_updated,
        "branch": s.field_default_branch,
        "fail": s.fetch_fail,
    })
    .to_string();

    format!(
        r#"<!DOCTYPE html>
<html lang="{html_lang}" dir="{dir}">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<meta name="description" content="{intro}">
<link rel="stylesheet" href="/style.css">
</head>
<body class="meta-index">
<main>
  <header>
    <h1>{h1}</h1>
  </header>

  <div class="lang-switch">{lang_switch}</div>

  <div class="org-link">
    <a href="https://github.com/aon-co-jp" target="_blank" rel="noopener">{org_link_label}</a>
  </div>

  <p class="meta-intro">{intro}</p>

  {cards}

  <div class="org-link">
    <a href="/">{back_to_top}</a>
  </div>

  <footer>&copy; 2026 aruaru.tokyo (Rust + Poem) — meta index for the open-cosmo ecosystem</footer>
</main>
<script type="application/json" id="page-data">{page_data_json}</script>
<script src="/meta-index.js" defer></script>
</body>
</html>
"#,
        html_lang = lang.html_lang(),
        dir = dir,
        title = html_escape(s.title),
        intro = html_escape(s.intro),
        h1 = html_escape(s.h1),
        lang_switch = lang_switch,
        org_link_label = html_escape(s.org_link_label),
        cards = cards,
        back_to_top = html_escape(s.back_to_top),
        page_data_json = page_data_json,
    )
}
