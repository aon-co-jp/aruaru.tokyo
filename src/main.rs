//! aruaru.tokyo — Rust + Poem 版TOPページ。
//! audiocafe.tokyo (PHP) とは異なり、こちらはpoem-cosmo-tauriのエコシステム
//! 方針に合わせてRust+Poemで実装する。DB非依存・1バイナリ完結。

use poem::endpoint::StaticFileEndpoint;
use poem::listener::TcpListener;
use poem::http::StatusCode;
use poem::web::{Html, Json, Query};
use poem::{get, handler, post, EndpointExt, Route, Server};
use rand::seq::SliceRandom;
use serde::Deserialize;

mod i18n;
mod meta_index;

const ARUARU_EASYWEB_URL: &str = "https://runo.tokyo/";

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

/// クリックした瞬間にYouTube検索を行うリンク(検索結果ページ自体は
/// 検索の都度最新のものが表示される。掲載時点での特定動画へのリンクでは
/// ないため、リンク切れが起きない/検索結果は常に最新)。
fn youtube_search_link(label: &str, query: &str) -> String {
    format!(
        r#"<a href="https://www.youtube.com/results?search_query={}" target="_blank" rel="noopener noreferrer">▶️ {}</a>"#,
        percent_encode(query),
        label
    )
}

/// Googleの動画検索(YouTube以外の動画サイトも横断的に含まれる)を、
/// クリックした瞬間に行うリンク。
fn google_video_search_link(label: &str, query: &str) -> String {
    format!(
        r#"<a href="https://www.google.com/search?q={}&tbm=vid" target="_blank" rel="noopener noreferrer">🎬 {}</a>"#,
        percent_encode(query),
        label
    )
}

/// runo.tokyoのTOPページ(このサイトとは別ドメイン、東京都西部の暮らし・
/// テレワーク紹介とopen-cosmoエコシステムの入口)への明示リンク先
/// (2026-07-20追記、ユーザー指示: 「aruaru.tokyo から runo.tokyo への
/// リンク」)。既存の`ARUARU_EASYWEB_URL`も同じ`https://runo.tokyo/`を
/// 指すが、「🔧 aruaru-easyweb を開く」というラベルでは行き先がruno.tokyo
/// だと分かりにくいため、別途分かりやすいラベルのリンクを追加する。
const RUNO_TOKYO_URL: &str = "https://runo.tokyo/";

/// ユーザー提供のブログ記事(タイトルをリンクテキストにし、URLそのものは
/// 表示しない、2026-07-20追記)。
const BLOG_POST_URL: &str = "https://ameblo.jp/www-aon/entry-12973252437.html";
const BLOG_POST_TITLE_JA: &str = "プログラム言語やフレームワークなどの全てをRust(Poemやhyper)版に移植するメリット?";
/// ユーザー指示(2026-07-20)により日英両方で掲載。ブログ本文自体は
/// 日本語のみだが、リンクのラベルは英語話者にも内容が伝わるよう
/// 意訳したもの(URLは日英共通、リンク先は変えない)。
const BLOG_POST_TITLE_EN: &str = "The benefits of migrating everything — programming languages, frameworks, and more — to Rust";

/// 2件目のブログ記事(ユーザー指示、2026-08-04追記)。
const BLOG_POST2_URL: &str = "https://ameblo.jp/www-aon/entry-12974607800.html";
const BLOG_POST2_TITLE_JA: &str = "上下水道配管や屋根瓦などのハイテク新素材。パナホームとヤマダホームのコーキングレス外壁";

/// 3件目のブログ記事(ユーザー指示、2026-08-08追記)。「民間のガン治療法に
/// 関する報道」セクションの直下に掲載する(ユーザー指示の掲載位置)。
const BLOG_POST3_URL: &str = "https://ameblo.jp/www-aon/entry-12975130765.html";
const BLOG_POST3_TITLE_JA: &str = "足振りで腰痛改善＋ダイエット";

const GITHUB_ORG: &str = "aon-co-jp";
const GITHUB_ORG_URL: &str = "https://github.com/aon-co-jp";

const GITHUB_REPOS: &[&str] = &[
    "open-cosmo",
    "poem-cosmo-tauri",
    "open-web-server",
    "aruaru-db",
    "open-raid-z",
    "open-easyweb",
    "aruaru-easyweb",
    "rs-to-readme",
    "readme-to-rs",
    "aruaru-ai",
    "open-cuda",
];

/// リポジトリ名として妥当な形式か検証する(GitHubの実際の命名規則: 英数字・
/// ハイフン・アンダースコア・ドットのみ)。「🔄 最新のリポジトリ一覧を取得」で
/// 動的に取得したリポジトリは`GITHUB_REPOS`の静的リストに含まれないため、
/// 固定リストとの照合ではなくこの形式検証で受け付ける。
fn is_valid_repo_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

const REPO_FILES: &[(&str, &str, bool)] = &[
    ("README.md", "README(概要)", true),
    ("CLAUDE.md", "CLAUDE.md(開発方針 & 開発環境ルール)", false),
    ("PORTING.md", "PORTING.md(お引越し可能ファイル)", false),
];

struct RelatedSite {
    label_ja: &'static str,
    label_en: &'static str,
    url_ja: &'static str,
    url_en: &'static str,
}

const RELATED_SITES: &[RelatedSite] = &[
    RelatedSite {
        label_ja: "audiocafe.tokyo/aruaru(IT・建築系求人 日本語版)",
        label_en: "audiocafe.tokyo/aruaru (English, translated by Claude Code)",
        url_ja: "https://audiocafe.tokyo/aruaru/",
        url_en: "https://audiocafe.tokyo/aruaru/index-en.php",
    },
    RelatedSite {
        label_ja: "audiocafe.tokyo/aruaru-lady(女性向け求人 日本語版)",
        label_en: "audiocafe.tokyo/aruaru-lady (English, translated by Claude Code)",
        url_ja: "https://audiocafe.tokyo/aruaru-lady/",
        url_en: "https://audiocafe.tokyo/aruaru-lady/index-en.php",
    },
    RelatedSite {
        label_ja: "aon.tokyo(AI・IT・WEB・オーディオ)",
        label_en: "aon.tokyo (AI/IT/WEB & audio equipment)",
        url_ja: "https://aon.tokyo/",
        url_en: "https://aon.tokyo/",
    },
    RelatedSite {
        label_ja: "aon.co.jp(AI・IT・WEB・オーディオ、aon.tokyoと同一内容)",
        label_en: "aon.co.jp (AI/IT/WEB & audio equipment, same content as aon.tokyo)",
        url_ja: "https://aon.co.jp/",
        url_en: "https://aon.co.jp/",
    },
    RelatedSite {
        label_ja: "karu.tokyo(軽井沢・あきる野・東京の観光とリモートワーク)",
        label_en: "karu.tokyo (Karuizawa/Akiruno/Tokyo tourism & remote work)",
        url_ja: "https://karu.tokyo/",
        url_en: "https://karu.tokyo/",
    },
    RelatedSite {
        label_ja: "aruaru.tokyo/rakuten-mobile(楽天モバイル情報)",
        label_en: "aruaru.tokyo/rakuten-mobile (Rakuten Mobile info)",
        url_ja: "https://aruaru.tokyo/rakuten-mobile/",
        url_en: "https://aruaru.tokyo/rakuten-mobile/",
    },
];

/// 新規事業提案(不動産電子契約・AUDIOルーム/シアタールームリフォーム・
/// 各種保険・酪農家黒字化の人工授精士の話・植林〈杉より高級木材〉・
/// 商工会議所&民生委員のSET創業支援&AI駆動プログラマー育成)の
/// 日本語原文+英訳(2026-09-09追加、ユーザー指示によりrunno.tokyoと同じ
/// 内容をaruaru.tokyoにも掲載)。
const PROPOSAL_TEXT_JA_1: &str = "新規提案として、Amazon倉庫、コンビニ、スーパーなどのコーヒーの自動販売機コーナーや、イートインコーナーなどで、新規企画で始められるなら不動産の売買、賃貸などの電子契約、建設会社＆AUDIOルーム、IMAXシアターや4DXシアターの家庭版や住宅の壁や天井にスピーカーを埋め込んで映画館の様にリフォームしたり、それを一戸建て注文住宅として建設する工務店サービス、それを本物の映画館やコンサートホールとして建設する建設会社のサービス。AFLACやNISSAYやSONY損保などの人間やペットや火災保険、地震保険、自動車保険、ちなみに海外の自動車保険のセールスマンは自動車の新車も中古車の売買も行ないますし、お買い得な売出し中の一戸建て注文住宅も、建売物件もご紹介可能らしいです。";

const PROPOSAL_TEXT_JA_2: &str = "ちなみ、先祖にレモン牛乳を作った関東牛乳の創業者がいますが、酪農家が黒字になる話として、まずは、人工授精士と言う資格の講座を受講して頂き、一生懸命ノートに講座の内容を書いてメモして、そしてTESTを受けてみごと合格して頂き、和牛の精子と卵子の受精卵を乳牛のメスに移植して妊娠すれば、乳牛が和牛と乳牛の両方の子牛を出産して牛乳とそれらの加工品と合わせて出荷する事で、酪農家が黒字なりやすいです。と言うお話もしておきます。ちなみに北海道くらい広い土地なら、なるべくフラットに平らに整地して、冬の豪雪が解けるタイミングで放牧の牛糞と雪が解けていっしょになると処理が大変な時に、あらかじめ、泥もじゃりも一緒に流し込めて、リサイクル土や肥料として出荷して販売してもらう前提で、排水溝や下水溝を建設する企画もご提案しておきます。";

const PROPOSAL_TEXT_JA_3: &str = "日本の国土の7割は、山間部でして、植林をさせるのでしたら、杉の木よりも、花粉が舞わないように品種改良してヒノキやその他、丈夫な木で一戸建て注文住宅を建てたい人も多い黒檀やその他、白檀など高級木材は沢山の種類が御座います。日本全国や世界中の材木屋さんや建設会社や不動産会社と一緒にSETで工務店も経営したり、大工さんと業務提携すれば、杉の木よりは、比較的良い値段で売れやすいのではないでしょうか？";

const PROPOSAL_TEXT_JA_4: &str = "日本全国の商工会議所と民生委員の方には是非国家公務員と民間の外郭団体の両方からお給料がもらえる様な必要なら法改正もして頂き、その他、商工会議所と民生委員と市区町村役場の方が、植林組合と大工さんと不動産会社と工務店とリフォーム会社や外壁塗装のサービスやAFLACや自動車などの保険と新車と中古車の販売会社も一緒にSETで創業し経営するノウハウを伝授したり次のサポートや業務提携やコンサルや就職サイトや不動産サイトも経営して頂き、企業誘致や工場誘致にはドローン空撮してその動画をYoutubeにUPしてそのリンクを市区町村のホームページなどでも紹介して頂いたり、その他、TOYOTAやLEXUSが中国の無人運転技術に完全に依存している問題は、得にAI駆動プログラマー育成を含む、国内のエンジニア育成をしていく得に、未経験者を企業研修しながら育成していく為の助成金や、補助金やその様なお金で、CLAUDE　CODE DESKTOP利用料を無償化してAI　先生として利用出来ると良い提案になると思っております。TOYOTAの様に中国の工場での生産コストと同等か、もしくは、それ以下に、日本の工場での生産コストを抑える為に、エンジンの一部を3Dプリンターで製造したり、搬入ロボットやロボットアームを含む生産ロボットなどの導入して実現しておりますので、Made in Japanのフル復活の為にも、なるべくやすく、壊れにくくメンテナンスもトラックから製造工場への搬入するロボットも含めて、国策、政策の一部に、日本全国の皆様のご協力が大切で不可欠だと思われます。";

const PROPOSAL_TEXT_EN_1: &str = "As a new business proposal: at coffee vending-machine corners and eat-in areas inside Amazon warehouses, convenience stores, supermarkets and the like, new ventures could be launched — electronic contracts for real-estate sales and rentals; construction companies paired with audio rooms; home versions of IMAX or 4DX theaters, or renovation services that embed speakers into a house's walls and ceiling to turn it into a movie-theater-like space; local builders (koumuten) offering this as a custom-built house; and construction companies offering it as an actual movie theater or concert hall. Insurance such as AFLAC, Nissay, and Sony Sompo for people and pets, fire insurance, earthquake insurance, and auto insurance — incidentally, overseas auto-insurance salespeople reportedly also handle both new- and used-car sales, and can apparently introduce good-deal custom-order houses currently on the market as well as ready-built (spec) homes.";

const PROPOSAL_TEXT_EN_2: &str = "By the way, one of my ancestors founded Kanto Milk, maker of \"lemon milk.\" Here's a story about how dairy farmers can turn a profit: first, take a course toward the \"artificial-insemination technician\" qualification, diligently write notes on the course content, sit the test, and pass it. Then, by implanting into a female dairy cow a fertilized egg made from Wagyu sperm and egg and having her become pregnant, the dairy cow gives birth to a calf that is part Wagyu and part dairy breed, and by shipping both the milk and these processed products together, it becomes much easier for the dairy farmer to turn a profit. I'll mention that as well. Also, on land as vast as Hokkaido, I'd propose leveling the ground as flat as possible, and building drainage/sewer channels designed so that, when winter's heavy snow melts and mixes with grazing cattle manure (which then becomes hard to handle), mud and gravel can be flushed in together too — on the premise that the resulting mixture is shipped and sold as recycled soil or fertilizer.";

const PROPOSAL_TEXT_EN_3: &str = "About 70% of Japan's land is mountainous. If reforestation is to be carried out, rather than cedar (sugi), there are many other options — hinoki (Japanese cypress) bred not to scatter pollen, and other sturdy woods that many people want for building custom-order houses, as well as high-grade timbers such as ebony and sandalwood, of which there are many varieties. By also running a construction/building business as a \"set\" together with lumber merchants, construction companies, and real-estate companies across Japan and around the world, and by forming business partnerships with carpenters, wouldn't these be relatively easier to sell at a better price than cedar?";

const PROPOSAL_TEXT_EN_4: &str = "I'd like to see chambers of commerce and civil welfare commissioners (minsei-iin) all across Japan be able to receive salaries from both national civil-service positions and private-sector affiliated organizations — amending the law if necessary. Beyond that, I'd like chambers of commerce, civil welfare commissioners, and municipal government staff to pass on the know-how for founding and running a combined (\"set\") business together with a reforestation cooperative, carpenters, real-estate companies, builders (koumuten), renovation companies, exterior-wall painting services, AFLAC and auto insurance, and new/used car dealers — and to also run the follow-up support, business partnerships, consulting, job-placement sites, and real-estate sites that go with it. For attracting companies and factories, I'd like them to shoot drone aerial footage, upload the videos to YouTube, and introduce those links on municipal websites as well. On another note, regarding the problem that Toyota and Lexus have become completely dependent on Chinese autonomous-driving technology: I think it would make a great proposal to use subsidies and grants for training domestic engineers — including especially training AI-driven programmers — by putting inexperienced people through in-house company training, and using that funding to make Claude Code Desktop free to use as an AI teacher. Just as Toyota has managed to keep production costs in its Japanese factories at or below the level of its Chinese factories by manufacturing part of the engine with 3D printers and introducing delivery robots, robot arms, and other production robots, I believe nationwide cooperation from everyone across Japan is essential, as part of national and government policy, toward a full revival of \"Made in Japan\" — including robots, as cheap and durable as possible with easy maintenance, that carry parts all the way from the truck into the factory.";

fn render_proposal_section() -> String {
    format!(
        r#"<section class="block" id="proposal"><h2>新規事業提案 / New Business Proposal</h2><p>{ja1}</p><p>{ja2}</p><p>{ja3}</p><p>{ja4}</p><h3>English</h3><p>{en1}</p><p>{en2}</p><p>{en3}</p><p>{en4}</p></section>"#,
        ja1 = PROPOSAL_TEXT_JA_1,
        ja2 = PROPOSAL_TEXT_JA_2,
        ja3 = PROPOSAL_TEXT_JA_3,
        ja4 = PROPOSAL_TEXT_JA_4,
        en1 = PROPOSAL_TEXT_EN_1,
        en2 = PROPOSAL_TEXT_EN_2,
        en3 = PROPOSAL_TEXT_EN_3,
        en4 = PROPOSAL_TEXT_EN_4,
    )
}

/// aon.tokyoに掲載済みの3本の動画セクション(鴨頭さんのマクドナルド動画・
/// 宇宙の広さ体感動画・宇宙の大きさを体感できる動画)をそのままコピーした
/// もの(2026-08-08追加、ユーザー指示)。カテゴリ一覧とガン治療報道
/// セクションの間に掲載する。
fn render_video_sections() -> String {
    r##"<p class="blog-link" style="font-size:1.4rem;"><a href="https://www.youtube.com/results?search_query=%E3%83%89%E3%82%A4%E3%83%84%E4%BC%81%E6%A5%AD%E3%81%AFIT%20WEB%20AI%E6%B4%BB%E7%94%A8%E3%81%A7%E6%99%82%E7%9F%AD%E5%8B%A4%E5%8B%99%20%E6%9C%89%E7%B5%A6%E6%B6%88%E5%8C%96%E7%8E%87%E3%81%8C%E9%AB%98%E3%81%84%20%E9%80%B1%E4%BC%913%E6%97%A5" target="_blank" rel="noopener noreferrer">▶️ ドイツ企業はIT WEB AI活用で時短勤務 有給消化率が高い 週休3日</a></p>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">【2ch馴れ初め】クレーム地獄の最下位スーパーに左遷された俺 →実はみんな優秀だったので本社を見返した結果 【ゆっくり】</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/In-Zf9LlUGs" title="【2ch馴れ初め】クレーム地獄の最下位スーパーに左遷された俺 →実はみんな優秀だったので本社を見返した結果 【ゆっくり】" loading="lazy" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1600622331706696?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">【漫画】俺を嫌う社長息子にクレーム地獄の最下位スーパーへ左遷されたが店内を見て回り俺は「改善できますね」と改革を進めることに…【恋愛マンガ動画】</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/kErqo1sG4vw" title="【漫画】俺を嫌う社長息子にクレーム地獄の最下位スーパーへ左遷されたが店内を見て回り俺は「改善できますね」と改革を進めることに…【恋愛マンガ動画】" loading="lazy" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/2307818063323329?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">AI × 電話が作り出す衝撃的すぎる販売方法</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/SY6ur61NRQY" title="AI × 電話が作り出す衝撃的すぎる販売方法" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1862687241780111?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">コールセンターのAI革命！3つのAI搭載型CRMを徹底比較 （ブログ公開）</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/ytel7hnI4yQ" title="コールセンターのAI革命！3つのAI搭載型CRMを徹底比較 （ブログ公開）" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1858159605169381?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
<p><a href="https://gigxit.co.jp/blog/blog-14717/" target="_blank" rel="noopener noreferrer">Blog（ブログ）</a></p>
<p><a href="https://ameblo.jp/www-aon/entry-12975242866.html" target="_blank" rel="noopener noreferrer">BlogのBackup（ブログのバックアップ）</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">AISmartCallが凄すぎる！意地悪な客にも神対応？最新AIコールセンターの実力を徹底検証</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/GfwmakE6Usc" title="AISmartCallが凄すぎる！意地悪な客にも神対応？最新AIコールセンターの実力を徹底検証" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1722317175553405?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">AIが最適な回答を提示　カスタマーセンター向け新カスハラ対策(2024年10月10日)</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/ryYgyiM07-A" title="AIが最適な回答を提示　カスタマーセンター向け新カスハラ対策(2024年10月10日)" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1555401388944798?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">EC×AIで10億円を1人で回す時代｜代理店はもはや不要？</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/f8i7tIcyrTY" title="EC×AIで10億円を1人で回す時代｜代理店はもはや不要？" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1378596376931848?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">GoogleのEC特化AI「UCP」がついに開始！事業者がやるべき準備とは？</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/iFn9LGnL6zQ" title="GoogleのEC特化AI「UCP」がついに開始！事業者がやるべき準備とは？" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1064557992696202?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">Claude Codeで公式LINEのAIチャットボットをゼロから作る実戦ガイド</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/Z89kjQRe0mw" title="Claude Codeで公式LINEのAIチャットボットをゼロから作る実戦ガイド" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p><a href="https://www.facebook.com/reel/1036159045970005?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">【衝撃】Claudeを使って10分でAIチャットボットを作成する方法を解説</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/c6zKkPYM4Uo" title="【衝撃】Claudeを使って10分でAIチャットボットを作成する方法を解説" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">I Built an AI Chatbot with Claude Code in 10 Minutes</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/rfewrxRs8Aw" title="I Built an AI Chatbot with Claude Code in 10 Minutes" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">鴨頭さんがマクドナルドの最低な店長だった話。指示、命令では人は動かない…元マクドナルドの店長が語る現場での対応</h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/vCTkpVXHmU4" title="鴨頭さんがマクドナルドの最低な店長だった話。指示、命令では人は動かない…元マクドナルドの店長が語る現場での対応" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">動画が表示されない場合はこちらで検索 / If the video does not play, search here: <a href="https://www.youtube.com/results?search_query=%E9%B4%A8%E9%A0%AD%E5%98%89%E4%BA%BA%EF%BC%88%E3%81%8B%E3%82%82%E3%81%8C%E3%81%97%E3%82%89%20%E3%82%88%E3%81%97%E3%81%B2%E3%81%A8%EF%BC%89" target="_blank" rel="noopener noreferrer">▶️ 鴨頭嘉人（かもがしら よしひと）の動画をYouTubeで検索</a></p>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたらYouTube検索と次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via YouTube search or the following URL:
<a href="https://www.facebook.com/reel/2030814436950418?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。<br><span style="color:var(--muted);">Experience the vastness of space beyond imagination, and how tiny Earth truly is.</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/HXwtnUEga7M" title="想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。 / Experience the vastness of space beyond imagination, and how tiny Earth truly is." frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">動画が表示されない場合はこちらで検索 / If the video does not play, search here: <a href="https://www.youtube.com/results?search_query=%E6%83%B3%E5%83%8F%E3%82%92%E8%B6%85%E3%81%88%E3%81%9F%E5%AE%87%E5%AE%99%E3%81%AE%E5%BA%83%E3%81%95%E3%80%81%E5%9C%B0%E7%90%83%E3%81%AE%E3%81%A1%E3%81%A3%E3%81%BD%E3%81%91%E3%81%95%E3%82%92%E4%BD%93%E6%84%9F%E3%81%97%E3%81%A6%E3%81%8F%E3%81%A0%E3%81%95%E3%81%84%E3%80%82" target="_blank" rel="noopener noreferrer">▶️ 想像を超えた宇宙の広さ、地球のちっぽけさを体感してください。 / Experience the vastness of space beyond imagination, and how tiny Earth truly is.</a></p>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたらYouTube検索と次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via YouTube search or the following URL:
<a href="https://www.facebook.com/reel/1514589143178165?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">宇宙の大きさを体感できる動画<br><span style="color:var(--muted);">A Video to Experience the Scale of the Universe</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/jM02C3uSBXY" title="宇宙の大きさを体感できる動画 / A Video to Experience the Scale of the Universe" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">動画が表示されない場合はこちらで検索 / If the video does not play, search here: <a href="https://www.youtube.com/results?search_query=%E5%AE%87%E5%AE%99%E3%81%AE%E5%A4%A7%E3%81%8D%E3%81%95%E3%82%92%E4%BD%93%E6%84%9F%E3%81%A7%E3%81%8D%E3%82%8B%E5%8B%95%E7%94%BB" target="_blank" rel="noopener noreferrer">▶️ 宇宙の大きさを体感できる動画 / A Video to Experience the Scale of the Universe</a></p>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたらYouTube検索と次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via YouTube search or the following URL:
<a href="https://www.facebook.com/reel/2142495496668566?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">日本にイスラエルのダビデ王家の子孫が生きている。中心はダビデ(大避〈おおさけ〉)神社である。<br><span style="color:var(--muted);">Descendants of the House of David are said to be living in Japan, centered on Osake (Ōsake) Shrine.</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/gBAqOBrKwFA" title="日本にイスラエルのダビデ王家の子孫が生きている。中心はダビデ(大避)神社である。" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/4618844761723076?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">「神秘すぎて、もはや怖い…」伊勢神宮で日本の宗教観を知ったイスラム教徒の夫婦。結果、イスラム教を辞める<br><span style="color:var(--muted);">"Too mysterious, almost frightening…" A Muslim couple learns of Japan's religious worldview at Ise Grand Shrine — and as a result, leaves Islam.</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/x44VLHDQ8_g" title="伊勢神宮で日本の宗教観を知ったイスラム教徒の夫婦。結果、イスラム教を辞める" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/954906917654955?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">東亜紀 君を乗せて<br><span style="color:var(--muted);">Toaki - Kimi wo Nosete</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/rxu7SU53zJ4" title="東亜紀 君を乗せて" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/1046062047914231?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">東亜樹 いつも何度でも<br><span style="color:var(--muted);">Toaki - Itsumo Nando Demo</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/dRQl07pSfpU" title="東亜樹 いつも何度でも" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/1079734318342948?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">東亜紀 童神<br><span style="color:var(--muted);">Toaki - Warabigami</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/FNUKj6qMQZw" title="東亜紀 童神" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">歌心りえ 冬のソナタの日本語版 最初から今まで<br><span style="color:var(--muted);">Utagokoro Rie - "From the Beginning Until Now" (Japanese version, Winter Sonata)</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/qVymStzDEjU" title="歌心りえ 冬のソナタの日本語版 最初から今まで" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/2273699263045573?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">歌心りえ 道化師のソネット<br><span style="color:var(--muted);">Utagokoro Rie - Doukeshi no Sonnet</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/iC1Y-1pwbm0" title="歌心りえ 道化師のソネット" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/1071996554862372?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">歌心りえ たしかなこと<br><span style="color:var(--muted);">Utagokoro Rie - Tashikana Koto</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/vPGm2q79px4" title="歌心りえ たしかなこと" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
If the YouTube link breaks, you can also watch it via the following URL:
<a href="https://www.facebook.com/reel/1339787288201605?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
</div>"##
        .to_string()
}

/// 民間のガン治療法に関する報道記事の紹介セクション(2026-07-24追加)。
/// ユーザーから提供された実際の報道見出し・リンクをそのまま紹介するのみに留め、
/// 独自の医療的な効能・安全性の主張や推奨は一切追加しない。
fn render_cancer_news_section() -> String {
    format!(
        r##"<section class="category">
    <h2>民間のガン治療法に関する報道 / News on Cancer Treatment Research</h2>
    <p style="font-size:.85rem;color:var(--muted);">以下は報道・公開情報の紹介のみで、独自の医療的な効能・安全性の主張は行っていません。 /
    The items below are simply introduced as reported information; no independent medical claims are made.</p>
    <ul>
      <li>癌の根治目指す「飲み薬」　狙うはノーベル賞！新薬の構造に迫る【ブレイクスルー】<br>
      <span style="color:var(--muted);">A "Pill" Aiming for a Complete Cure for Cancer — Targeting a Nobel Prize! Examining the Structure of the New Drug [Breakthrough]</span><br>
      <a href="https://youtu.be/ZKRBr5Zb6uY" target="_blank" rel="noopener noreferrer">▶️ YouTube</a> /
      <a href="https://www.facebook.com/reel/1581437446958593?locale=ja_JP" target="_blank" rel="noopener noreferrer">📘 Facebook(予備 / backup)</a></li>
      <li>衝撃波で腫瘍を破壊「メスも針も使わない」肝臓がんの新治療法　大阪公立大の研究チームが特定臨床研究を開始　来年中の薬事承認を目指す<br>
      <span style="color:var(--muted);">Destroying Tumors with Shockwaves — a New "No Scalpel, No Needle" Liver Cancer Treatment: Osaka Metropolitan University Research Team Begins Specified Clinical Research, Aiming for Drug/Medical Device Approval Within the Next Year</span><br>
      <a href="https://www.youtube.com/watch?v=hRFXYCGX8Fo" target="_blank" rel="noopener noreferrer">▶️ YouTube</a> /
      <a href="https://www.facebook.com/masahiro.ishizuka.54?locale=ja_JP" target="_blank" rel="noopener noreferrer">📘 Facebook</a></li>
      <li>マックトリガー。世界初！からだ自身が"がん治療"　九州大学が開発<br>
      <span style="color:var(--muted);">Mac Trigger. A World First! The Body Itself Fights Cancer — Developed by Kyushu University</span><br>
      <a href="https://www.youtube.com/watch?v=84EkcJmgmnQ" target="_blank" rel="noopener noreferrer">▶️ YouTube</a> /
      <a href="https://www.facebook.com/reel/1793445321653771?locale=ja_JP" target="_blank" rel="noopener noreferrer">📘 Facebook(予備 / backup)</a></li>
      <li>がんが小さくなる 金沢大学がん薬物療法とは？ 分子標的療法<br>
      <span style="color:var(--muted);">Cancer Shrinkage: What Is Kanazawa University's Cancer Drug Therapy? Molecular Targeted Therapy</span><br>
      <a href="https://youtu.be/u4xTbs4JZ30" target="_blank" rel="noopener noreferrer">▶️ YouTube</a></li>
      <li><a href="https://aon.tokyo/cancer" target="_blank" rel="noopener noreferrer">民間のガン治療法についての情報は aon.tokyo/cancer をご覧ください</a><br>
      <span style="color:var(--muted);">For information on non-clinical/private-sector cancer treatment approaches, see aon.tokyo/cancer.</span></li>
      <li>{cancer_search_jp} / {cancer_search_en}</li>
      <li>{cancer_video_search_jp} / {cancer_video_search_en}</li>
      <li>{banana_search_jp} / {banana_search_en}</li>
      <li>{baking_soda_search_jp} / {baking_soda_search_en}</li>
      <li>{citric_acid_search_jp} / {citric_acid_search_en}</li>
    </ul>
  </section>"##,
        cancer_search_jp = youtube_search_link("がんの治療法について調べる", "がん 治療法"),
        cancer_search_en = youtube_search_link("Cancer treatment methods", "cancer treatment methods"),
        cancer_video_search_jp = google_video_search_link("がんの治療法の動画を調べる", "がん 治療法"),
        cancer_video_search_en = google_video_search_link("Cancer treatment method videos", "cancer treatment methods"),
        banana_search_jp = youtube_search_link("「バナナ ガン治療法」で調べる", "バナナ ガン治療法"),
        banana_search_en = youtube_search_link("Search \"banana cancer treatment\"", "banana cancer treatment"),
        baking_soda_search_jp = youtube_search_link("「重曹水 ガン治療法」で調べる", "重曹水 ガン治療法"),
        baking_soda_search_en = youtube_search_link("Search \"baking soda water cancer treatment\"", "baking soda water cancer treatment"),
        citric_acid_search_jp = youtube_search_link("「クエン酸水 ガン治療法」で調べる", "クエン酸水 ガン治療法"),
        citric_acid_search_en = youtube_search_link("Search \"citric acid water cancer treatment\"", "citric acid water cancer treatment"),
    )
}

fn categories() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "IT エンジニアあるある",
            vec![
                "「動かないんですけど」→ 5分後に自己解決している",
                "本番環境で試したらすぐ直る不具合、ローカルでは絶対再現しない",
                "会議は「あとでSlackで」で締めるのに結局Slackでも決まらない",
                "ドキュメントを書いた瞬間に仕様が変わる",
                "「ちょっと直しますね」が気づいたら3時間経っている",
            ],
        ),
        (
            "在宅ワークあるある",
            vec![
                "カメラオンの会議、下だけパジャマ",
                "「聞こえてますか?」を1日に3回は言う",
                "昼休みのつもりが気づいたら1時間半経っている",
                "宅配便のインターホンにビクッとする",
                "椅子から立った回数より水を飲んだ回数の方が少ない",
            ],
        ),
        (
            "朝活あるある",
            vec![
                "前日の夜は「明日は5時起きする」と誓う",
                "起きた瞬間には「今日はやめとこう」に変わっている",
                "三日坊主どころか初日で心が折れる",
                "それでも次の日また同じ誓いを立てる",
            ],
        ),
        (
            "SNSあるある",
            vec![
                "「見るだけのつもり」が気づいたら1時間",
                "投稿した3秒後に誤字を発見する",
                "「いいね」の数を意味もなく確認しにいく",
                "通知オフにしたはずなのに結局アプリを開いている",
            ],
        ),
        (
            "日本の会社あるある",
            vec![
                "「一応」で始まる念のための確認メールが多い",
                "会議の議題より雑談の方が盛り上がる",
                "有給を取る理由を無駄に考えてしまう",
                "「持ち帰って検討します」が一番よく使うフレーズ",
            ],
        ),
    ]
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// READMEなど各Markdownを`//!`付きrustdocコメント形式の`.rs`風テキストへ変換する
/// (readme-to-rs構想の簡易実装)。
fn markdown_to_rs(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| if line.is_empty() { "//!".to_string() } else { format!("//! {line}") })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// MarkdownをGitHub風の見た目でレンダリングしたHTMLへ変換する
/// (`pulldown-cmark`使用。見出し・リスト・コードブロック・リンク等の
/// 標準的なMarkdown記法に対応)。
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

/// GitHub APIから`aon-co-jp`の全リポジトリ名を最新の状態で取得する。
/// `aon-co-jp`はOrganizationではなく個人アカウントのため`/users/`
/// エンドポイントを使う(`/orgs/`だと404になることを実機確認済み)。
/// 認証無しの公開APIを使うため、レート制限(未認証: 60回/時/IP)に注意。
async fn fetch_org_repos(client: &reqwest::Client) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/users/{GITHUB_ORG}/repos?per_page=100&sort=updated");
    let resp = client
        .get(&url)
        .header("User-Agent", "aruaru.tokyo-repo-list/0.1")
        .header("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let names = body
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|repo| repo.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(names)
}

#[handler]
async fn api_repos() -> poem::web::Json<serde_json::Value> {
    let client = reqwest::Client::new();
    match fetch_org_repos(&client).await {
        Ok(names) => poem::web::Json(serde_json::json!({ "repos": names })),
        Err(e) => poem::web::Json(serde_json::json!({ "error": e })),
    }
}

async fn fetch_repo_file(client: &reqwest::Client, repo: &str, filename: &str) -> Option<String> {
    for branch in ["main", "master"] {
        let url = format!("https://raw.githubusercontent.com/{GITHUB_ORG}/{repo}/{branch}/{filename}");
        if let Ok(resp) = client
            .get(&url)
            .header("User-Agent", "aruaru.tokyo-readme-to-rs/0.1")
            .timeout(std::time::Duration::from_secs(6))
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    return Some(body);
                }
            }
        }
    }
    None
}

#[derive(Deserialize)]
struct TopQuery {
    repo: Option<String>,
}

fn render_related_sites() -> String {
    RELATED_SITES
        .iter()
        .map(|s| {
            format!(
                r#"<a href="{ja_url}" target="_blank" rel="noopener">{ja_label}</a>
    <a href="{en_url}" target="_blank" rel="noopener">🌐 {en_label}</a>"#,
                ja_url = s.url_ja,
                ja_label = html_escape(s.label_ja),
                en_url = s.url_en,
                en_label = html_escape(s.label_en),
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn render_categories() -> String {
    categories()
        .into_iter()
        .map(|(cat, items)| {
            let lis = items
                .iter()
                .map(|item| format!("<li>{}</li>", html_escape(item)))
                .collect::<Vec<_>>()
                .join("\n      ");
            format!(
                r#"<section class="category">
    <h2>{cat}</h2>
    <ul>
      {lis}
    </ul>
  </section>"#,
                cat = html_escape(cat)
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ")
}

fn render_repo_options(selected: &str) -> String {
    GITHUB_REPOS
        .iter()
        .map(|repo| {
            let sel = if *repo == selected { " selected" } else { "" };
            format!(r#"<option value="{repo}"{sel}>{repo}</option>"#)
        })
        .collect::<Vec<_>>()
        .join("\n        ")
}

async fn render_repo_results(selected_repo: &str) -> String {
    if !is_valid_repo_name(selected_repo) {
        return String::new();
    }
    let client = reqwest::Client::new();
    let repo_url = format!("{GITHUB_ORG_URL}/{selected_repo}");
    let mut out = String::new();
    out.push_str(&format!(
        "<p class=\"repo-link\"><a href=\"{repo_url}\" target=\"_blank\" rel=\"noopener\">🔗 {selected_repo} をGitHubで開く</a></p>\n"
    ));
    for (idx, (filename, label, required)) in REPO_FILES.iter().enumerate() {
        let markdown = fetch_repo_file(&client, selected_repo, filename).await;
        out.push_str("<div class=\"repo-file-block\">\n");
        out.push_str(&format!("  <h3>{}</h3>\n", html_escape(label)));
        match markdown {
            Some(md) => {
                let gh_html = markdown_to_github_style_html(&md);
                let rs = html_escape(&markdown_to_rs(&md));
                let tab_id = format!("{selected_repo}-{idx}");
                out.push_str(&format!(
                    r#"  <div class="view-toggle" data-tab="{tab_id}">
    <button type="button" class="view-toggle-btn active" data-view="gh">GitHub風表示</button>
    <button type="button" class="view-toggle-btn" data-view="rs">.rs形式</button>
  </div>
  <div class="markdown-body" id="gh-{tab_id}">{gh_html}</div>
  <pre class="rs-output hidden" id="rs-{tab_id}">{rs}</pre>
"#
                ));
            }
            None => {
                let msg = if *required {
                    format!("❌ {filename} を取得できませんでした({selected_repo})。")
                } else {
                    format!("❌ {filename} はこのリポジトリにはありません。")
                };
                out.push_str(&format!("  <p class=\"rs-error\">{}</p>\n", html_escape(&msg)));
            }
        }
        out.push_str("</div>\n");
    }
    out
}

fn flat_items_json() -> String {
    let mut items = Vec::new();
    for (cat, list) in categories() {
        for text in list {
            items.push(serde_json::json!({"category": cat, "text": text}));
        }
    }
    // shuffle server-side once at render time is unnecessary; client shuffles on click.
    let _ = items.choose(&mut rand::thread_rng());
    serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string())
}

#[handler]
async fn top(Query(q): Query<TopQuery>) -> Html<String> {
    let selected_repo = q.repo.unwrap_or_default();
    let selected_repo = if is_valid_repo_name(&selected_repo) { selected_repo } else { String::new() };

    let repo_results = render_repo_results(&selected_repo).await;
    let related_sites = render_related_sites();
    let categories_html = render_categories();
    let repo_options = render_repo_options(&selected_repo);
    let flat_json = flat_items_json();
    let page_data_json = format!(r#"{{"items":{flat_json}}}"#);
    let video_sections = render_video_sections();
    let proposal_section = render_proposal_section();
    let cancer_news_section = render_cancer_news_section();
    let claude_code_desktop_search = youtube_search_link("AI駆動開発 CLAUDE CODE DESKTOP", "AI駆動開発 CLAUDE CODE DESKTOP");

    let body = format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>aruaru.tokyo | みんなの「あるある」集めました</title>
<meta name="description" content="IT・在宅ワーク・SNS・日本の会社など、誰もが頷く「あるある」をジャンル別にまとめたサイト。GitHubリポジトリのREADMEを.rs風に変換して表示する機能も搭載。Rust+Poem製。">
<link rel="stylesheet" href="/style.css">
</head>
<body>
<main>
  <p class="blog-link"><a href="{BLOG_POST_URL}" target="_blank" rel="noopener">📝 {BLOG_POST_TITLE_JA}</a> / <a href="{BLOG_POST_URL}" target="_blank" rel="noopener">{BLOG_POST_TITLE_EN}</a></p>
  <p class="blog-link"><a href="{BLOG_POST2_URL}" target="_blank" rel="noopener">📝 {BLOG_POST2_TITLE_JA}</a></p>

  <header>
    <h1>aruaru<span>.tokyo</span></h1>
    <p>「それ、あるある!」を集めました。 / A collection of everyday "aruaru" moments.</p>
    <p class="related-sites" style="font-size:0.85rem;">🔗 関連サイト / Related sites:
    <a href="https://aon.tokyo/">aon.tokyo</a> ・
    <a href="https://aon.tokyo/cancer">aon.tokyo/cancer</a> ・
    <a href="https://aon.co.jp/">aon.co.jp</a> ・
    <a href="https://aruaru.tokyo/">aruaru.tokyo</a> ・
    <a href="https://karu.tokyo/">karu.tokyo</a> ・
    <a href="https://icpo.tokyo/">icpo.tokyo</a> ・
    <a href="https://fbi.tokyo/">fbi.tokyo</a> ・
<a href="https://runo.tokyo/">runo.tokyo</a></p>
    <p style="font-size:0.85rem;">イエス・キリスト</p>
    <p style="font-size:0.85rem;">気功</p>
    <p style="font-size:0.85rem;">気功。KIKOU.<br>
    <a href="https://youtu.be/WBUojmKng3M" target="_blank" rel="noopener noreferrer">https://youtu.be/WBUojmKng3M</a></p>
    <p style="font-size:0.85rem;">気功。KIKOU. Facebook Backup.<br>
    <a href="https://www.facebook.com/100000656454938/videos/566679491579232" target="_blank" rel="noopener noreferrer">https://www.facebook.com/100000656454938/videos/566679491579232</a></p>
    <p style="font-size:0.85rem;">気功。KIKOU. 自ら"気"を感じながら相手に流す練習法(動画)。<br>
    <video controls preload="metadata" style="width:100%;max-width:640px;aspect-ratio:16/9;height:auto;background:#000;border-radius:6px;object-fit:contain;">
    <source src="/video/kikou-practice.mp4" type="video/mp4">
    お使いのブラウザは動画再生に対応していません。<a href="/video/kikou-practice.mp4">動画ファイルを直接開く</a>
    </video></p>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">【簡単】気を強力に出して気功治療もできる【気功講座12】</h2>
    <div class="fb-click" data-href="https://www.facebook.com/reel/932810149903379" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:linear-gradient(135deg,#1c2b4a,#0b1020);border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.facebook.com/reel/932810149903379?locale=ja_JP" target="_blank" rel="noopener noreferrer" title="【簡単】気を強力に出して気功治療もできる【気功講座12】" style="display:flex;align-items:center;justify-content:center;width:100%;height:100%;color:#fff;text-decoration:none;">
    <span style="font-size:4rem;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/932810149903379?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/932810149903379?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=-FrUNr6ozfY" target="_blank" rel="noopener noreferrer">【気功治療】自分にやってみる【気功講座20】</a></h2>
    <div class="yt-click" data-id="-FrUNr6ozfY" data-title="【気功治療】自分にやってみる【気功講座20】" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=-FrUNr6ozfY" target="_blank" rel="noopener noreferrer" title="【気功治療】自分にやってみる【気功講座20】" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/-FrUNr6ozfY/hqdefault.jpg" alt="【気功治療】自分にやってみる【気功講座20】" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=zNqA9q9l5M0" target="_blank" rel="noopener noreferrer">【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】</a></h2>
    <div class="yt-click" data-id="zNqA9q9l5M0" data-title="【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=zNqA9q9l5M0" target="_blank" rel="noopener noreferrer" title="【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/zNqA9q9l5M0/hqdefault.jpg" alt="【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">【金融シリーズ①】これを見逃すとエライことになります。日本国債＋リスクヘッジで7％の利息受取？</h2>
    <video controls preload="metadata" style="width:100%;max-width:640px;aspect-ratio:16/9;height:auto;background:#000;border-radius:6px;object-fit:contain;">
    <source src="/video/NIHON-KOKUSAI.mp4" type="video/mp4">
    お使いのブラウザは動画再生に対応していません。<a href="/video/NIHON-KOKUSAI.mp4">動画ファイルを直接開く</a>
    </video>
    <p><a href="https://www.youtube.com/watch?v=0zFcPiy6K10" target="_blank" rel="noopener noreferrer">https://www.youtube.com/watch?v=0zFcPiy6K10</a></p>
    <p><a href="https://www.facebook.com/reel/1589704945870968?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
    <p><a href="/video/7-percent.mp4" target="_blank" rel="noopener noreferrer">動画(mp4、予備 / backup)</a></p>
    <div id="tokoro-gate" style="margin-top:1rem;">
    <p>パスワードを入力して下さい。</p>
    <form id="tokoro-form" autocomplete="off">
    <input type="password" id="tokoro-pw" autocomplete="off" aria-label="パスワード" style="padding:0.4rem;font-size:1rem;">
    <button type="submit">入力完了</button>
    </form>
    <p id="tokoro-msg" style="color:#c00;font-size:0.9rem;"></p>
    </div>
    <div id="tokoro-video" style="display:none;margin-top:1rem;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">所さん！事件ですよ　ＴＫＧが海外で大人気！？　ニッポンの卵が大進化</h2>
    <video id="tokoro-player" controls preload="none" style="width:100%;max-width:640px;aspect-ratio:16/9;height:auto;background:#000;border-radius:6px;object-fit:contain;"></video>
    </div>
    <div style="margin-top:1rem;">
    <p>個人的バックアップ用</p>
    <p><a href="https://drive.google.com/file/d/1XEpYGPhKF7zWCoWfG_Nv2qrTBlXUhmnU/view?usp=sharing" target="_blank" rel="noopener noreferrer">Google Drive(予備 / backup)</a></p>
    <div style="margin-top:1rem;text-align:left;font-size:1.05rem;">
    <p><b>所さん事件ですよ TKGが海外で大人気 与えるエサ 発酵薬膳の熟成海鮮リゾットのレシピと作り方が知りたい</b></p>
    <p>番組の回は見つかりましたが、「発酵薬膳の熟成海鮮リゾット」のレシピは見つかりませんでした。</p>
    <p><b>分かったこと</b></p>
    <ul>
    <li>該当回は、NHK「所さん！事件ですよ」の「TKGが海外で大人気！？ニッポンの卵が大進化」です。</li>
    <li>番組情報によると、この回には「熟成海鮮リゾットでできたニワトリのエサ」が出てきます。つまり、人が食べるリゾットではありません。高級ブランド卵を産ませるために鶏へ与える飼料のようです。</li>
    <li>飼料の配合、発酵や熟成の工程、薬膳素材の中身は、検索結果の範囲では公開されていませんでした。</li>
    </ul>
    <p><b>詳しく知る方法</b></p>
    <ul>
    <li>番組の公式ページや見逃し配信（NHKプラス）で、該当コーナーを確認する。</li>
    <li>番組の回の紹介ページで、紹介された養鶏場名を調べる。そこの公式サイトに、飼料のこだわりが載っている可能性があります。</li>
    </ul>
    <p><b>発酵薬膳風 熟成海鮮リゾット（2人分）</b><br>番組の再現ではなく、人が食べるためのAIによるオリジナルレシピです。あくまでも番組の再現は出来ませんでしたので、ご参考まで。</p>
    <p><b>材料</b></p>
    <ul>
    <li>米（洗わない）：1合</li>
    <li>魚介（エビ、ホタテ、イカ、白身魚など）：200g</li>
    <li>塩麹：大さじ1（魚介の熟成用）</li>
    <li>昆布：5cm角1枚</li>
    <li>玉ねぎ（みじん切り）：1/4個</li>
    <li>生姜（みじん切り）：1片</li>
    <li>ごま油またはオリーブオイル：大さじ2</li>
    <li>日本酒：50ml</li>
    <li>昆布だし：約500ml</li>
    <li>味噌：小さじ1</li>
    <li>クコの実：小さじ1（ぬるま湯で戻す）</li>
    <li>長ねぎ（小口切り）：適量</li>
    <li>黒こしょう、ごま：少々</li>
    </ul>
    <p><b>作り方</b></p>
    <ol>
    <li>熟成：魚介に塩麹をなじませ、昆布で挟んで冷蔵庫に3〜6時間おきます。使う前に塩麹を軽くぬぐい、一口大に切ります。</li>
    <li>鍋で油と生姜を弱火で炒めます。香りが出たら魚介をさっと焼いて、いったん取り出します。</li>
    <li>玉ねぎを透き通るまで炒め、米を加えて2分ほど炒めます。</li>
    <li>日本酒を入れて煮詰めます。</li>
    <li>温めた昆布だしをお玉1杯ずつ加え、混ぜながら15分ほど煮ます。味噌は、だしの一部で溶いて途中で加えます。</li>
    <li>米にわずかに芯が残る状態で、魚介とクコの実を戻します。1〜2分温めて、塩気を見て調整します。</li>
    <li>長ねぎ、黒こしょう、ごまをかけて完成です。</li>
    </ol>
    <p><b>ポイント</b></p>
    <ul>
    <li>塩麹と味噌の塩分があるので、塩は最後に味を見てから足します。</li>
    <li>魚介を熟成させると身がしっとりして、旨みが増します。長くおくと塩辛くなるので、6時間までにします。</li>
    <li>薬膳らしくしたい場合は、なつめ、陳皮（みかんの皮）、松の実を少量加えてもよいです。</li>
    </ul>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/shorts/7wp8UYDx0dY" target="_blank" rel="noopener noreferrer">トヨタが全固体電池を超えた新電池を開発して米中EV絶望で世界中大パニック</a></h2>
    <div id="toyota-box" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;">
    <a id="toyota-link" href="https://www.youtube.com/shorts/7wp8UYDx0dY" target="_blank" rel="noopener noreferrer" title="トヨタが全固体電池を超えた新電池を開発して米中EV絶望で世界中大パニック" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/7wp8UYDx0dY/hqdefault.jpg" alt="トヨタが全固体電池を超えた新電池を開発して米中EV絶望で世界中大パニック" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <script>
    document.getElementById('toyota-link').addEventListener('click',function(e){{
      e.preventDefault();
      document.getElementById('toyota-box').innerHTML='<iframe src="https://www.youtube.com/embed/7wp8UYDx0dY?autoplay=1" title="トヨタが全固体電池を超えた新電池を開発して米中EV絶望で世界中大パニック" style="width:100%;height:100%;border:0;" allow="autoplay; encrypted-media; picture-in-picture" allowfullscreen></iframe>';
    }});
    </script>
    <p><a href="https://www.youtube.com/shorts/7wp8UYDx0dY" target="_blank" rel="noopener noreferrer">https://www.youtube.com/shorts/7wp8UYDx0dY</a></p>
    <div style="margin-top:1rem;text-align:left;font-size:1.05rem;">
    <p><b>AI による概要</b><br>トヨタが開発している「ファイバー電池」および同時期に注目されている次世代電池（全固体電池など）の特性について、項目ごとに解説します。なお、トヨタ中央研究所が2024年に発表した「ファイバー電池（同心円状の電極構造を持つ小型リチウムイオン電池）」は、主にドローンやウェアラブル機器、デバイスの骨格自体をバッテリーにする技術として研究が進められています。</p>
    <p><b>1. 温度耐性（低い温度・高い温度）</b></p>
    <ul>
    <li><b>低い温度（低温耐性）：</b> 従来の液体リチウムイオン電池は、氷点下になると電解液の粘度が上がり性能が著しく低下します。一方、ファイバー電池は極細の同心円構造によってイオンの移動経路が非常に短いため、低温環境でも効率よくイオンを拡散させやすく、性能低下を抑えられるポテンシャルを持っています。</li>
    <li><b>高い温度（高温耐性）：</b> ファイバー電池は従来の液体電解質（有機溶媒）をベースにしているため、極端な高温（一般的に80℃以上）環境では熱暴走のリスクがあり、全固体電池ほどの超高温耐性はありません。</li>
    </ul>
    <p><b>2. 急速充電</b></p>
    <ul>
    <li>非常に高い急速充放電性能を誇ります。</li>
    <li>ファイバー電池は、中心の炭素繊維（負極）の周りをセパレーターと正極で覆う「3次元の同心円状構造」を採用しています。これにより、従来のシートを重ねるタイプの電池に比べて電極の対向面積が圧倒的に広く、イオンの移動距離が短いため、大電流を一気に流す「急速充電」や「高出力な放電」が構造上得意となっています。</li>
    </ul>
    <p><b>3. 安全性</b></p>
    <ul>
    <li><b>形状自由度によるフェイルセーフ：</b> 糸のように細いユニットを束ねて使用するため、万が一どこか1本が破損・短絡（ショート）しても、電池全体が一気に熱暴走を起こすリスクを分散・軽減しやすい構造をしています。</li>
    <li><b>電解質の制約：</b> 構造的な強みはあるものの、内部に可燃性の液体電解質を使用している点では従来のリチウムイオン電池と同じです。そのため、電解質自体が燃えない固体でできている「全固体電池」と比較すると、根本的な発火リスクの低さ（安全性）という面では一歩譲ります。</li>
    </ul>
    <p><b>💡 補足：EV用の「全固体電池」との違い</b><br>トヨタの次世代電池のニュースでは、EV（電気自動車）向けに2027〜2028年の実用化を目指している「全固体電池」も有名です。これらは以下のように特性や用途が住み分けられると予想されています。</p>
    <table style="border-collapse:collapse;width:100%;font-size:0.95rem;">
    <tr><th style="border:1px solid #888;padding:6px;">項目</th><th style="border:1px solid #888;padding:6px;">ファイバー電池（中研開発）</th><th style="border:1px solid #888;padding:6px;">全固体電池（EV向け本命）</th></tr>
    <tr><td style="border:1px solid #888;padding:6px;">主な用途</td><td style="border:1px solid #888;padding:6px;">ドローン、ロボット、ウェアラブル機器</td><td style="border:1px solid #888;padding:6px;">電気自動車（EV）</td></tr>
    <tr><td style="border:1px solid #888;padding:6px;">構造の特徴</td><td style="border:1px solid #888;padding:6px;">糸状（繊維）で、デバイスの骨格に組み込める</td><td style="border:1px solid #888;padding:6px;">電解質をすべて固体（セラミクス等）に置き換え</td></tr>
    <tr><td style="border:1px solid #888;padding:6px;">温度耐性</td><td style="border:1px solid #888;padding:6px;">短い経路により低温に強いが、高温は並み</td><td style="border:1px solid #888;padding:6px;">低温から高温まで極めて広い範囲で安定</td></tr>
    <tr><td style="border:1px solid #888;padding:6px;">急速充電</td><td style="border:1px solid #888;padding:6px;">イオン移動距離が短いため非常に速い</td><td style="border:1px solid #888;padding:6px;">高温に強く大電流を流せるため10分以下</td></tr>
    <tr><td style="border:1px solid #888;padding:6px;">安全性</td><td style="border:1px solid #888;padding:6px;">破損時のリスク分散構造</td><td style="border:1px solid #888;padding:6px;">液漏れせず、絶対に燃えない高い安全性</td></tr>
    </table>
    <p style="margin-top:1rem;">Google検索ワード：<b>セルロース　カーボン ファイバー バッテリー</b><br>その検索結果は<a href="https://www.google.com/search?q=%E3%82%BB%E3%83%AB%E3%83%AD%E3%83%BC%E3%82%B9%E3%80%80%E3%82%AB%E3%83%BC%E3%83%9C%E3%83%B3+%E3%83%95%E3%82%A1%E3%82%A4%E3%83%90%E3%83%BC+%E3%83%90%E3%83%83%E3%83%86%E3%83%AA%E3%83%BC" target="_blank" rel="noopener noreferrer">こちら</a></p>
    <p><a href="https://www.google.com/search?q=%E3%82%BB%E3%83%AB%E3%83%AD%E3%83%BC%E3%82%B9%E3%83%8A%E3%83%8E%E3%83%95%E3%82%A1%E3%82%A4%E3%83%90%E3%83%BC%EF%BC%88CNF%EF%BC%89+%E3%83%90%E3%83%83%E3%83%86%E3%83%AA%E3%83%BC" target="_blank" rel="noopener noreferrer">セルロースナノファイバー（CNF） バッテリー</a></p>
    <p><a href="https://www.youtube.com/results?search_query=%E3%83%87%E3%83%A5%E3%82%A2%E3%83%AB%E3%82%AB%E3%83%BC%E3%83%9C%E3%83%B3%E3%83%90%E3%83%83%E3%83%86%E3%83%AA%E3%83%BC" target="_blank" rel="noopener noreferrer">デュアルカーボンバッテリー</a></p>
    <p>デュアルカーボンファイバーバッテリー<br><a href="https://ameblo.jp/www-aon/entry-12980558678.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12980558678.html</a></p>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=p0j1ULzUadQ" target="_blank" rel="noopener noreferrer">8割の面積が消えた回路／日清紡がシリコンマザーボードを発表／チップレットで載せきれない受動部品までICと1枚に</a></h2>
    <div class="yt-click" data-id="p0j1ULzUadQ" data-title="8割の面積が消えた回路／日清紡がシリコンマザーボードを発表／チップレットで載せきれない受動部品までICと1枚に" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=p0j1ULzUadQ" target="_blank" rel="noopener noreferrer" title="8割の面積が消えた回路／日清紡がシリコンマザーボードを発表／チップレットで載せきれない受動部品までICと1枚に" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/p0j1ULzUadQ/hqdefault.jpg" alt="8割の面積が消えた回路／日清紡がシリコンマザーボードを発表／チップレットで載せきれない受動部品までICと1枚に" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/1123034143705564?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1123034143705564?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=wUHR7L9LwM0" target="_blank" rel="noopener noreferrer">空中発射型ロケット実験成功（2021年1月19日）</a></h2>
    <div class="yt-click" data-id="wUHR7L9LwM0" data-title="空中発射型ロケット実験成功（2021年1月19日）" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=wUHR7L9LwM0" target="_blank" rel="noopener noreferrer" title="空中発射型ロケット実験成功（2021年1月19日）" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/wUHR7L9LwM0/hqdefault.jpg" alt="空中発射型ロケット実験成功（2021年1月19日）" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/1799737474697368?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1799737474697368?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=BEDhnuk4kJg" target="_blank" rel="noopener noreferrer">空飛ぶロケット発射機【世界最大の飛行機】極超音速飛行体の空中発射母機</a></h2>
    <div class="yt-click" data-id="BEDhnuk4kJg" data-title="空飛ぶロケット発射機【世界最大の飛行機】極超音速飛行体の空中発射母機" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=BEDhnuk4kJg" target="_blank" rel="noopener noreferrer" title="空飛ぶロケット発射機【世界最大の飛行機】極超音速飛行体の空中発射母機" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/BEDhnuk4kJg/hqdefault.jpg" alt="空飛ぶロケット発射機【世界最大の飛行機】極超音速飛行体の空中発射母機" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/943388708419201?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/943388708419201?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/shorts/PiAzzP8vYp8" target="_blank" rel="noopener noreferrer">遠心力でロケットを打ち上げ!</a></h2>
    <div class="yt-click" data-id="PiAzzP8vYp8" data-title="遠心力でロケットを打ち上げ!" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/shorts/PiAzzP8vYp8" target="_blank" rel="noopener noreferrer" title="遠心力でロケットを打ち上げ!" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/PiAzzP8vYp8/hqdefault.jpg" alt="遠心力でロケットを打ち上げ!" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/2274281903372044?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/2274281903372044?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=10UgbgnsAcE" target="_blank" rel="noopener noreferrer">日本の新たな発見、コンクリートに取って代わる可能性</a></h2>
    <div class="yt-click" data-id="10UgbgnsAcE" data-title="日本の新たな発見、コンクリートに取って代わる可能性" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=10UgbgnsAcE" target="_blank" rel="noopener noreferrer" title="日本の新たな発見、コンクリートに取って代わる可能性" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/10UgbgnsAcE/hqdefault.jpg" alt="日本の新たな発見、コンクリートに取って代わる可能性" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/2530793094085429?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/2530793094085429?locale=ja_JP</a></p>
    <div style="text-align:left;font-size:1.05rem;">
    <p>通常のコンクリートの場合は、セルロースナノファイバーを混ぜるとより頑丈になります。<br>更に固まる直前に、モバイルバイブマッサージャーなどで振動を与えて均一感を与えると、更に頑丈になります。</p>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/shorts/EifHEsWIBDU" target="_blank" rel="noopener noreferrer">人のウンチを混ぜたら強度42％アップ!?</a></h2>
    <div class="yt-click" data-id="EifHEsWIBDU" data-title="人のウンチを混ぜたら強度42％アップ!?" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/shorts/EifHEsWIBDU" target="_blank" rel="noopener noreferrer" title="人のウンチを混ぜたら強度42％アップ!?" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/EifHEsWIBDU/hqdefault.jpg" alt="人のウンチを混ぜたら強度42％アップ!?" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/4601253880112669?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/4601253880112669?locale=ja_JP</a></p>
    <div style="text-align:left;font-size:1.05rem;">
    <p>2026年9月29日<br>2026年<br>人の排泄物から作ったバイオ炭を、コンクリートのセメントの一部として利用する研究が行われています。<br>10％を置き換えた実験では、91日後に圧縮強度が21％、曲げ強度が42％増加しました。<br>廃棄物を建築材料として活用できる可能性が研究されています。</p>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: left; max-width: 640px; font-size:1.05rem;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;">奈良林直×加藤康子◆放射性廃棄物の無害化に成功！Part2 核技術界に激震!!! 原発の未来に光！</h2>
    <p>信憑性は不明です。嘘とか不可能とか出来ないと言う動画も御座います。</p>
    <p>Youtube検索結果は<a href="https://www.youtube.com/results?search_query=%E5%A5%88%E8%89%AF%E6%9E%97%E7%9B%B4%C3%97%E5%8A%A0%E8%97%A4%E5%BA%B7%E5%AD%90%E2%97%86%E6%94%BE%E5%B0%84%E6%80%A7%E5%BB%83%E6%A3%84%E7%89%A9%E3%81%AE%E7%84%A1%E5%AE%B3%E5%8C%96%E3%81%AB%E6%88%90%E5%8A%9F%EF%BC%81Part2+%E6%A0%B8%E6%8A%80%E8%A1%93%E7%95%8C%E3%81%AB%E6%BF%80%E9%9C%87!!!+%E5%8E%9F%E7%99%BA%E3%81%AE%E6%9C%AA%E6%9D%A5%E3%81%AB%E5%85%89%EF%BC%81" target="_blank" rel="noopener noreferrer">こちら</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=Xm1faoNSYK4" target="_blank" rel="noopener noreferrer">【朗報！】なんと大阪公立大学がレアアース回収に成功！南鳥島レアアースに追い風になる理由を解説します！！</a></h2>
    <div class="yt-click" data-id="Xm1faoNSYK4" data-title="【朗報！】なんと大阪公立大学がレアアース回収に成功！南鳥島レアアースに追い風になる理由を解説します！！" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=Xm1faoNSYK4" target="_blank" rel="noopener noreferrer" title="【朗報！】なんと大阪公立大学がレアアース回収に成功！南鳥島レアアースに追い風になる理由を解説します！！" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/Xm1faoNSYK4/hqdefault.jpg" alt="【朗報！】なんと大阪公立大学がレアアース回収に成功！南鳥島レアアースに追い風になる理由を解説します！！" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/1778440423261550?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1778440423261550?locale=ja_JP</a></p>
    </div>
    <script>
    document.querySelectorAll('.yt-click').forEach(function(b){{
      b.querySelector('a').addEventListener('click',function(e){{
        e.preventDefault();
        var f=document.createElement('iframe');
        f.src='https://www.youtube.com/embed/'+b.dataset.id+'?autoplay=1';
        f.title=b.dataset.title;
        f.style.cssText='width:100%;height:100%;border:0;';
        f.allow='autoplay; encrypted-media; picture-in-picture';
        f.allowFullscreen=true;
        b.innerHTML='';
        b.appendChild(f);
      }});
    }});
    </script>
    <script>
    document.querySelectorAll('.fb-click').forEach(function(b){{
      b.querySelector('a').addEventListener('click',function(e){{
        e.preventDefault();
        var f=document.createElement('iframe');
        f.src='https://www.facebook.com/plugins/video.php?href='+encodeURIComponent(b.dataset.href)+'&show_text=false&autoplay=true';
        f.style.cssText='width:100%;height:100%;border:0;';
        f.allow='autoplay; encrypted-media; picture-in-picture';
        f.allowFullscreen=true;
        b.innerHTML='';
        b.appendChild(f);
      }});
    }});
    </script>
    </div>
    <script>
    (function(){{
      var f=document.getElementById('tokoro-form');
      f.addEventListener('submit',function(e){{
        e.preventDefault();
        var m=document.getElementById('tokoro-msg');
        m.textContent='';
        fetch('/api/tokoro-unlock',{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{password:document.getElementById('tokoro-pw').value}})}})
          .then(function(r){{return r.ok?r.json():Promise.reject();}})
          .then(function(j){{
            document.getElementById('tokoro-player').src='/video/tokoro.mp4?t='+encodeURIComponent(j.token);
            document.getElementById('tokoro-gate').style.display='none';
            document.getElementById('tokoro-video').style.display='block';
          }})
          .catch(function(){{m.textContent='パスワードが違います。';}});
      }});
    }})();
    </script>
    </div>
  </header>

  <div class="quick-links">
    <a href="{RUNO_TOKYO_URL}" target="_blank" rel="noopener">🏠 runo.tokyo を見る</a>
    <a href="{ARUARU_EASYWEB_URL}" target="_blank" rel="noopener">🔧 aruaru-easyweb を開く</a>
    <a href="/help">❓ 困った時は</a>
    <a href="/open-aruaru-runo-iLumi">📚 プロジェクトシリーズ</a>
    <br />
    <a href="https://www.youtube.com/watch?v=mrCAz7mU9Zo" target="_blank" rel="noopener noreferrer">▶️ 【1日密着】Claude Codeに取り憑かれたエンジニア｜その衝撃の開発手法に迫る</a>
    <br />
    <a href="https://www.facebook.com/reel/1753405299237847?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook Backup</a>
    <br />
    {claude_code_desktop_search}
    <br />
    {related_sites}
  </div>

  <div class="shuffle-bar">
    <button id="shuffle-btn" type="button">🎲 ランダムに1つ表示</button>
    <div id="shuffle-result">
      <div class="cat"></div>
      <div class="txt"></div>
    </div>
  </div>

  {categories_html}

  {video_sections}

  {proposal_section}

  {cancer_news_section}

  <p class="blog-link" style="font-size:1.4rem;"><a href="{BLOG_POST3_URL}" target="_blank" rel="noopener">📝 {BLOG_POST3_TITLE_JA}</a></p>

  <section class="tool">
    <h2>📄 README/CLAUDE.md/PORTING.md → .rs 変換ビューア</h2>
    <p class="desc">
      GitHub({GITHUB_ORG})のリポジトリを選ぶと、README(概要)・CLAUDE.md
      (開発方針 & 開発環境ルール)・PORTING.md(お引越し可能ファイル)を
      それぞれrustdocコメント(<code>//!</code>)形式の <code>.rs</code> 風
      テキストに変換して表示します。(readme-to-rs構想の簡易実装、Rust+Poem実装)
    </p>
    <div class="repo-fetch-row">
      <button type="button" id="fetch-repos-btn">🔄 最新のリポジトリ一覧を取得</button>
      <span class="repo-fetch-status" id="fetch-repos-status"></span>
    </div>
    <form class="repo-form" method="get">
      <select name="repo" id="repo-select">
        <option value="">リポジトリを選択…</option>
        {repo_options}
      </select>
      <button type="submit">表示</button>
    </form>
    {repo_results}
  </section>

  <div class="org-link">
    <a href="{GITHUB_ORG_URL}" target="_blank" rel="noopener">🏢 GitHub organization: {GITHUB_ORG} のトップを見る</a>
  </div>

  <div class="space-video" style="margin: 1.5rem 0; text-align: center;">
  <h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">一番より大切なものを選んだ子</h2>
  <div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
  <iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/np9DpR9dWfM" title="一番より大切なものを選んだ子" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
  </div>
  <p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
  If the YouTube link breaks, you can also watch it via the following URL:
  <a href="https://www.facebook.com/reel/1345482087666813?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
  </div>

  <div class="space-video" style="margin: 1.5rem 0; text-align: center;">
  <h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">「日本の若者は全てを知っている…」日本だけ宗教戦争がない理由を、日本人JKが完全論破</h2>
  <div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
  <iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/ckPntNHlSI8" title="日本だけ宗教戦争がない理由を、日本人JKが完全論破" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
  </div>
  <p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
  If the YouTube link breaks, you can also watch it via the following URL:
  <a href="https://www.facebook.com/reel/1218746654048455?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
  </div>

  <div class="space-video" style="margin: 1.5rem 0; text-align: center;">
  <h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">数多くの日本人を泣かせた伝説のCM</h2>
  <div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
  <iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/qUqPhVLPB8c" title="数多くの日本人を泣かせた伝説のCM" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
  </div>
  <p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
  If the YouTube link breaks, you can also watch it via the following URL:
  <a href="https://www.facebook.com/reel/1382447680598734" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
  </div>

  <footer>&copy; 2026 aruaru.tokyo (Rust + Poem)</footer>
</main>
<script type="application/json" id="page-data">{page_data_json}</script>
<script src="/app.js" defer></script>
</body>
</html>
"#
    );
    Html(body)
}

#[derive(Deserialize)]
struct LangQuery {
    lang: Option<String>,
}

/// `/open-aruaru-runo-iLumi`(エイリアス`/open-aruaru-runo`)——
/// エコシステム全体のメタ索引リポジトリ`open-aruaru-runo-iLumi`と
/// 同内容のプロジェクトシリーズ索引ページ。両パスとも同じ内容を返す。
#[handler]
async fn meta_index_page(Query(q): Query<LangQuery>) -> Html<String> {
    let lang = i18n::Lang::parse(q.lang.as_deref());
    Html(meta_index::render_page(lang, "/open-aruaru-runo-iLumi"))
}

#[handler]
fn healthz() -> &'static str {
    "ok"
}

static TOKORO_TOKENS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

#[derive(serde::Deserialize)]
struct TokoroUnlock {
    password: String,
}

/// パスワードは環境変数`ARUARU_TOKYO_TOKORO_PASSWORD`(ソースには置かない)。未設定なら常に拒否。
#[handler]
async fn tokoro_unlock(Json(body): Json<TokoroUnlock>) -> poem::Result<Json<serde_json::Value>> {
    let expected = std::env::var("ARUARU_TOKYO_TOKORO_PASSWORD").unwrap_or_default();
    let expected = expected.trim();
    if expected.is_empty() || body.password.trim() != expected {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Err(poem::Error::from_status(StatusCode::UNAUTHORIZED));
    }
    let token: String = (0..4).map(|_| format!("{:016x}", rand::random::<u64>())).collect();
    let mut tokens = TOKORO_TOKENS.lock().unwrap();
    if tokens.len() >= 1000 {
        tokens.clear();
    }
    tokens.push(token.clone());
    Ok(Json(serde_json::json!({ "token": token })))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    let bind = std::env::var("ARUARU_TOKYO_BIND").unwrap_or_else(|_| "0.0.0.0:4100".to_string());
    let app = Route::new()
        .at("/", get(top))
        .at("/healthz", get(healthz))
        .at("/help", StaticFileEndpoint::new("static/help.html"))
        .at("/style.css", StaticFileEndpoint::new("static/style.css"))
        .at("/p", StaticFileEndpoint::new("static/p.html"))
        .at("/k", StaticFileEndpoint::new("static/k.html"))
        .at("/app.js", StaticFileEndpoint::new("static/app.js"))
        .at("/meta-index.js", StaticFileEndpoint::new("static/meta-index.js"))
        .at("/api/repos", get(api_repos))
        .at("/open-aruaru-runo-iLumi", get(meta_index_page))
        .at("/open-aruaru-runo", get(meta_index_page))
        .at("/video/7-percent.mp4", StaticFileEndpoint::new("video/7%.mp4"))
        .at("/video/kikou-practice.mp4", StaticFileEndpoint::new("video/kikou-practice.mp4"))
        .at("/video/NIHON-KOKUSAI.mp4", StaticFileEndpoint::new("video/NIHON-KOKUSAI.mp4"))
        .at("/api/tokoro-unlock", post(tokoro_unlock))
        .at(
            "/video/tokoro.mp4",
            StaticFileEndpoint::new("video/所さん！事件ですよ　ＴＫＧが海外で大人気！？　ニッポンの卵が大進化.mp4").before(|req| async move {
                let t = req.uri().query().and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("t=")));
                match t {
                    Some(t) if TOKORO_TOKENS.lock().unwrap().iter().any(|x| x == t) => Ok(req),
                    _ => Err(poem::Error::from_status(StatusCode::FORBIDDEN)),
                }
            }),
        );
    tracing::info!(%bind, "starting aruaru-tokyo-server");
    Server::new(TcpListener::bind(&bind)).run(app).await
}
