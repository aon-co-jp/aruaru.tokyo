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

/// AI駆動開発(WEBスキルが無くても母国語でWEBサイト・スマホアプリを開発する)の研究の一部として
/// 紹介する記事(2026-10-08追加、ユーザー指示)。タイトルはユーザー提供の表記のまま、URLも表示する。
const AI_DRIVEN_RESEARCH_HTML: &str = r#"<div class="ai-driven-research" style="margin:1rem auto;padding:.8rem 1rem;max-width:42rem;border:1px solid rgba(128,128,128,.5);border-radius:8px;text-align:left;">
    <strong>🔬 研究中：Claudeによる、WEBスキルが無くても日本語（母国語）でWEBサイトやスマホアプリを開発する為のAI駆動開発（研究の一部）</strong><br />
    📝 <a href="https://ameblo.jp/www-aon/entry-12973252437.html" target="_blank" rel="noopener noreferrer">プログラム言語やフレームワークなどの全てをRust（Poemやhyper）版に移植するメリット？</a><br />
    <a href="https://ameblo.jp/www-aon/entry-12973252437.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12973252437.html</a>
  </div>"#;

/// トップページの「Claude / aruaru 紹介」リンク群の直前に出す、オンライン学習サイトの紹介
/// (2026-10-08追加、ユーザー指示)。文言・URLはユーザー提供のものをそのまま使う。
/// `format!`の置換対象に埋め込むため、波括弧は含めないこと。
const ONLINE_LEARNING_HTML: &str = r#"<div class="online-learning" style="margin:1rem auto;padding:.8rem 1rem;max-width:42rem;border:1px solid rgba(128,128,128,.5);border-radius:8px;text-align:left;">
    <strong>🎓 オンライン学習サイト / Online learning</strong><br />
    AI駆動開発も学べます。WEBスキルなしで日本語でWEBサイトやスマホアプリが開発出来る様になります。<br />
    <a href="https://progate.com/courses/feature/ai-driven-development/exercises/atmoSFuL468uLDi-Pd-lI" target="_blank" rel="noopener noreferrer">▶️ Progate「AI駆動開発」</a><br />
    HTML,CSSを学ぼう<br />
    <a href="https://progate.com/courses/feature/webpage/exercises/hJQNO8xPIcha_idvTX2dn" target="_blank" rel="noopener noreferrer">▶️ Progate「HTML・CSS」</a><br />
    Pythonなら、無料ならPaizaの<br />
    <a href="https://paiza.jp/works/search_courses/2208" target="_blank" rel="noopener noreferrer">▶️ paizaラーニング「Python3」入門講座一覧</a><br />
    Python プロゲート<br />
    <a href="https://progate.com/courses/python" target="_blank" rel="noopener noreferrer">▶️ Progate「Python入門」</a><br />
    その他無料でPythonが学べます。<br />
    <a href="https://www.python.jp/train/index.html" target="_blank" rel="noopener noreferrer">▶️ python.jp「Python入門講座」</a>
  </div>"#;

/// ユーザー提供のブログ記事(タイトルをリンクテキストにし、URLそのものは
/// 表示しない、2026-07-20追記)。
const TOP_VIDEO_HTML: &str = r#"<section class="top-news" style="max-width:640px;margin:1rem auto;text-align:center;">
<div style="max-width:640px;margin:0 auto 1.4rem;padding:.8rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.8;">
<p style="margin:0 0 .6rem;font-weight:700;">2026/10/28　19:00 から　19:45くらいまで、<br>ヒューレット・パッカード　市ヶ谷本社と八王子と北八王子関係者と日本マイクロソフトと富士通と日立と東芝と品川区のソフトウェーブとバーテックスシステムズとアメリカ政府と日本政府と韓国政府と中国政府と中国公安警察と全人代関係者を名乗る方々が、ヒューレットパッカード様は優秀なのに石　塚　正　浩は馬鹿だから首にしましょうね。と、あきる野市に民泊しながら、大声で、以下の内容と共におっしゃりました。</p>
<p style="margin:0 0 .6rem;">私、石　塚　正　浩が、以前、北八王子のヒューレットパッカード（今は、キーサイト）にお仕事で、HP-UXのOSを10GBしかないHDDへのインストール依頼があったので、インストールと設定は成功したので、報告して帰った後に、社員がアプリを追加でインストールしようとしたら容量不足でインストール出来なかったと言うクレームを頂いた事がありました。</p>
<p style="margin:0 0 .6rem;">ヒューレットパッカード　市ケ谷本社に出向していた期間には、486SXと言うノートパソコンを最新のノートパソコンだから大切に使う様に言われて使いましたが、それで一生懸命お仕事しましたが、仕事が遅いとクレームを受けた事がありました。</p>
<p style="margin:0;">品川のソフトウェーブ経由で、ヒューレットパッカードの仕事で九州へ出張があっても、新幹線代とホテル代も出ませんでした。</p>
<p style="margin:.6rem 0 0;">品川のソフトウェーブの社員だった時は、手取り月給は、最初は、16万円くらいが二ヶ月間で、2回も給料が下がり、最終の手取り月給14万円くらいでした。退職金も1円ももらってません。</p>
<div lang="en" style="margin-top:.8rem;font-size:.9rem;opacity:.85;line-height:1.6;">
<p style="margin:0 0 .5rem;font-weight:700;">From about 19:00 to 19:45 on October 28, 2026, people claiming to be connected with Hewlett-Packard's Ichigaya headquarters, Hachioji and Kita-Hachioji, Microsoft Japan, Fujitsu, Hitachi, Toshiba, Softwave of Shinagawa Ward, Vertex Systems, the U.S. government, the Japanese government, the South Korean government, the Chinese government, the Chinese public security police and the National People's Congress said, &quot;Hewlett-Packard is excellent, but Masahiro Ishizuka is stupid, so let's fire him.&quot; While staying at a private lodging (minpaku) in Akiruno City, they said this in loud voices, together with the following.</p>
<p style="margin:0 0 .5rem;">I, Masahiro Ishizuka, once worked at Hewlett-Packard in Kita-Hachioji (now Keysight). I was asked to install the HP-UX operating system on a hard disk of only 10GB. The installation and configuration succeeded, and I reported back and left. Afterwards, when an employee tried to install additional applications, there was not enough space, and I received a complaint.</p>
<p style="margin:0 0 .5rem;">While I was seconded to Hewlett-Packard's Ichigaya headquarters, I was told to take good care of a 486SX notebook PC because it was the newest model. I worked as hard as I could on it, but I received a complaint that my work was slow.</p>
<p style="margin:0;">Even when I went on a business trip to Kyushu for Hewlett-Packard work through Softwave in Shinagawa, my Shinkansen fare and hotel costs were not paid.</p>
<p style="margin:.5rem 0 0;">When I was an employee of Softwave in Shinagawa, my take-home monthly pay was about 160,000 yen for the first two months, then my salary was cut twice, and my final take-home monthly pay was about 140,000 yen. I did not receive a single yen in retirement allowance.</p>
</div>
</div>
<div style="max-width:640px;margin:0 auto 1.4rem;padding:.8rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.8;">
<p style="margin:0 0 .6rem;font-weight:700;font-size:1.1rem;">消費税とは?</p>
<p style="margin:0 0 .6rem;">法人税が約40%だった時に、将来約20%まで下げる約束で導入された税金で、財源は、法人税だったのです。</p>
<p style="margin:0 0 .6rem;">消費税を全廃、撤廃するには、2026年の今、約20%まで下がっている法人税を元の約40%もしくは、足りなければ、45%くらいまで元に戻し、更に、財源が足りなければ、バラク・オバマ元大統領が言われた様に、我々、裕福層にこそ、所得税などの税金の累進課税率のカーブをUPするべきだと言う、ご提案に賛成で御座います。</p>
<p style="margin:0 0 .6rem;">所得税や社会保険税などの税金は、裕福層への累進課税率のカーブはアップして、逆に低所得層への累進課税率のカーブはダウンさせて、負担率を下げる、しかし一定の金額以下は、全く税金を払わなくて良いと言う考え方は、全く収入が無いに近い場合を除いて、考え直した方が良いと思います。</p>
<p style="margin:0 0 .6rem;">更に、バラク・オバマ元大統領は、次の様にも言われました。政府とは、大きいか？小さいか？ではなく、機能するか？しないか？であると。eガバメント、デジタルガバメントと言う政府やお役所のLINEアプリ化やTVアプリ化やコンビニのロッピーやファミポートなどによって、市区町村役場や都道府県庁の庁舎と司法立法行政府、中央省庁の全公務員を大幅にリストラして少子高齢化予算、専業主婦や働いていない人なども含めて全国民参加の年金を皆保険として、基礎年金+厚生年金+増額年金の年金予算確保、後期高齢者医療制度の低所得層と中間所得層の負担率の大幅なカーブのダウンや失業時だけ、住宅ローンや自動車ローンや携帯電話代やインターネットプロバイダー代金を肩代わりして無償化にする予算などや低所得層の医療費無償化や中間所得層の医療費の累進負担率のカーブも大幅に下げる予算などやイタリア製ミニウェル・プロクサ（お勧めは、左右の視力が異なっていても同じ種類の30cm手元から遠くまでよく見える眼内コンタクトレンズ）や長期間壊れにくくて安心して使える人工心臓や人工関節などを完全に国保対応とする予算確保など。</p>
<div lang="en" style="margin-top:.8rem;font-size:.9rem;opacity:.85;line-height:1.6;">
<p style="margin:0 0 .5rem;font-weight:700;">What is the consumption tax?</p>
<p style="margin:0 0 .6rem;">It was introduced when the corporate tax rate was about 40%, with a promise to lower the corporate tax to about 20% in the future, and its source of revenue was the corporate tax.</p>
<p style="margin:0 0 .6rem;">To abolish the consumption tax completely, in 2026 the corporate tax, which has now fallen to about 20%, should be restored to its original level of about 40%, or to around 45% if that is not enough. If revenue is still short, I agree with the proposal, as former President Barack Obama said, that we, the wealthy, should be the ones to raise the progressive curve of income tax and other taxes.</p>
<p style="margin:0 0 .6rem;">For income tax, social insurance premiums and similar taxes, the progressive curve should be raised for the wealthy and lowered for low-income earners to reduce their burden. However, the idea that people below a certain amount pay no tax at all should be reconsidered, except for those with almost no income.</p>
<p style="margin:0 0 .6rem;">Furthermore, former President Barack Obama also said the following: the question about government is not whether it is big or small, but whether it works or not. With e-government and digital government (government and local office services as LINE apps and TV apps, and at convenience-store terminals such as Loppi and FamiPort), all public employees at city, ward and town offices, prefectural offices, the judicial, legislative and executive branches and the central ministries should be drastically restructured, and the money saved should be used to secure: universal pension insurance covering the whole population, including full-time homemakers and people not working (basic pension + employees' pension + additional pension) to cope with the aging, declining-birth-rate society; a large lowering of the burden-rate curve for low- and middle-income people under the medical care system for people aged 75 and over; a budget for the government to take over the payments of housing loans, car loans, mobile phone bills and internet provider fees, making them free of charge, only while a person is unemployed; free medical care for low-income people and a greatly lowered progressive burden curve for middle-income people; and a budget to place under full national health insurance coverage the Italian Minuwell Proxa (recommended: an intraocular contact lens that lets people see well from 30 cm at hand to far away with the same type of lens, even when the left and right eyes have different vision), and of durable, reliable artificial hearts and artificial joints.</p>
</div>
</div>
<p style="margin:-.6rem 0 1.4rem;text-align:center;"><a href="https://maid-cafe.tokyo" target="_blank" rel="noopener noreferrer">https://maid-cafe.tokyo</a><br><small>「消費税」を質問すると、この説明を答えます / Ask about the consumption tax (消費税) to get this explanation</small></p>
<div style="max-width:360px;margin:0 auto 1.4rem;">
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://youtube.com/shorts/atA0yTg6-YE" target="_blank" rel="noopener noreferrer">Googleを落ちた高卒の23歳が世界を変えた実話</a><br><small lang="en" style="font-weight:400;">The true story of a 23-year-old high-school graduate who failed Google and changed the world</small></p>
<div style="position:relative;width:100%;aspect-ratio:9/16;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/atA0yTg6-YE?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='Googleを落ちた高卒の23歳が世界を変えた実話';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/atA0yTg6-YE/oar2.jpg" onerror="this.onerror=null;this.src='https://i.ytimg.com/vi/atA0yTg6-YE/hqdefault.jpg'" alt="Googleを落ちた高卒の23歳が世界を変えた実話" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="text-align:left;line-height:1.7;margin:.6rem 0 0;">2026年10月現在、TVや新聞や雑誌の広告からYouTubeやSNSなどのWEBに広告はシフトしているのですから、宣伝の仕方も変えていかないとね！<br>株式会社エーオン 代表取締役社長の石　塚　正　浩としても、広告代理店業を早期に立ち上げたいと思っております。</p>
<p lang="en" style="text-align:left;line-height:1.6;font-size:.9rem;opacity:.85;margin:.3rem 0 0;">As of October 2026, advertising has shifted from TV, newspapers and magazines to the web, such as YouTube and social media, so the way we promote things has to change too! As the President and CEO of AON Co., Ltd., Masahiro Ishizuka, I would like to launch an advertising agency business as soon as possible.</p>
</div>
<div lang="en" style="display:flex;flex-wrap:wrap;gap:1rem;justify-content:center;margin:0 auto 1.4rem;">
<div style="flex:1 1 280px;max-width:340px;">
<p style="font-size:1.05rem;font-weight:700;margin:0 0 .5rem;"><a href="https://www.youtube.com/shorts/FCQPLN6zndQ" target="_blank" rel="noopener noreferrer">His Company Worth Millions After He Was Fired - PART1</a></p>
<div style="position:relative;width:100%;aspect-ratio:9/16;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/FCQPLN6zndQ?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='His Company Worth Millions After He Was Fired - PART1';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/FCQPLN6zndQ/oar2.jpg" onerror="this.onerror=null;this.src='https://i.ytimg.com/vi/FCQPLN6zndQ/hqdefault.jpg'" alt="His Company Worth Millions After He Was Fired - PART1" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
</div>
<div style="flex:1 1 280px;max-width:340px;">
<p style="font-size:1.05rem;font-weight:700;margin:0 0 .5rem;"><a href="https://www.youtube.com/shorts/7aLHquRI5oY" target="_blank" rel="noopener noreferrer">His Company Worth Millions After He Was Fired - PART2</a></p>
<div style="position:relative;width:100%;aspect-ratio:9/16;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/7aLHquRI5oY?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='His Company Worth Millions After He Was Fired - PART2';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/7aLHquRI5oY/oar2.jpg" onerror="this.onerror=null;this.src='https://i.ytimg.com/vi/7aLHquRI5oY/hqdefault.jpg'" alt="His Company Worth Millions After He Was Fired - PART2" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
</div>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=4jQRmHMk9hU" target="_blank" rel="noopener noreferrer">米国の新発明が銅を完全に置き換える？ 中国が危機感を強める理由</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/4jQRmHMk9hU?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='米国の新発明が銅を完全に置き換える？ 中国が危機感を強める理由';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/4jQRmHMk9hU/hqdefault.jpg" alt="米国の新発明が銅を完全に置き換える？ 中国が危機感を強める理由" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=OGRP5KomO2U" target="_blank" rel="noopener noreferrer">【世界初の快挙！！】152年の紙の会社が選んだ...2ナノの道（CNF半導体）</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/OGRP5KomO2U?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='【世界初の快挙！！】152年の紙の会社が選んだ...2ナノの道（CNF半導体）';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/OGRP5KomO2U/hqdefault.jpg" alt="【世界初の快挙！！】152年の紙の会社が選んだ...2ナノの道（CNF半導体）" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=xzZuyvt6Jn8" target="_blank" rel="noopener noreferrer">日本製紙が新型蓄電池を開発！2025年の実用化を目指し、木材由来の素材であるセルロースナノファイバー（CNF）に注目が集まる！</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/xzZuyvt6Jn8?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='日本製紙が新型蓄電池を開発！2025年の実用化を目指し、木材由来の素材であるセルロースナノファイバー（CNF）に注目が集まる！';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/xzZuyvt6Jn8/hqdefault.jpg" alt="日本製紙が新型蓄電池を開発！2025年の実用化を目指し、木材由来の素材であるセルロースナノファイバー（CNF）に注目が集まる！" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=ovUeKAGipyA" target="_blank" rel="noopener noreferrer">日本の技術が「炭素繊維」を超えた❗世界を変える新発明の正体 CNF（セルロースナノファイバー）とは？</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/ovUeKAGipyA?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='日本の技術が「炭素繊維」を超えた❗世界を変える新発明の正体 CNF（セルロースナノファイバー）とは？';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/ovUeKAGipyA/hqdefault.jpg" alt="日本の技術が「炭素繊維」を超えた❗世界を変える新発明の正体 CNF（セルロースナノファイバー）とは？" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=xIyiGPwkyg8" target="_blank" rel="noopener noreferrer">【新電池材料】ボールペンの技術で『カーボンナノチューブ分散液』を開発【三菱鉛筆】Mitsubishi Pencil develops &quot;CNT dispersion liquid&quot;</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/xIyiGPwkyg8?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='【新電池材料】ボールペンの技術で『カーボンナノチューブ分散液』を開発【三菱鉛筆】Mitsubishi Pencil develops &quot;CNT dispersion liquid&quot;';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/xIyiGPwkyg8/hqdefault.jpg" alt="【新電池材料】ボールペンの技術で『カーボンナノチューブ分散液』を開発【三菱鉛筆】Mitsubishi Pencil develops &quot;CNT dispersion liquid&quot;" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=gxGx2emqn_w" target="_blank" rel="noopener noreferrer">【ゆっくり解説】中国が握るEV電池市場に異変！日本の黒い素材が世界を変える…ゼオンが単層カーボンナノチューブを数十倍増産、147億円投資でEV200万台分へ</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/gxGx2emqn_w?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='【ゆっくり解説】中国が握るEV電池市場に異変！日本の黒い素材が世界を変える…ゼオンが単層カーボンナノチューブを数十倍増産、147億円投資でEV200万台分へ';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/gxGx2emqn_w/hqdefault.jpg" alt="【ゆっくり解説】中国が握るEV電池市場に異変！日本の黒い素材が世界を変える…ゼオンが単層カーボンナノチューブを数十倍増産、147億円投資でEV200万台分へ" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
</section>
<div class="pixel-intro">
<div class="google-proposal" style="max-width:420px;margin:1rem auto .6rem;padding:.7rem 1rem;border:2px solid rgba(128,128,128,.6);border-radius:10px;text-align:left;line-height:1.6;">
<p style="margin:0 0 .1rem;"><strong>📣 Google様へのご提案！</strong></p>
<p style="margin:0 0 .5rem;">AppleのiPhoneのOS標準の「いつでも拡大縮小」機能を、Androidスマホとタブレットの全ての機種で、OS標準としてアップデートで対応してほしいです。</p>
<p lang="en" style="margin:0 0 .1rem;font-size:.9rem;opacity:.9;"><strong>📣 A proposal to Google!</strong></p>
<p lang="en" style="margin:0;font-size:.9rem;opacity:.9;">We would like the always-available zoom feature built into Apple's iPhone OS to be added as a standard OS feature, via an update, on all Android smartphones and tablets.</p>
</div>
<p style="text-align:center;margin:1rem auto .1rem;max-width:420px;font-size:.95rem;">💡 SMSのワンタイムパスワードが受け取れない場合に、設定から仮想メモリーを4GBから8GBに変更すると受け取れるようになる場合がございますので、お試しください。</p>
<p lang="en" style="text-align:center;margin:0 auto .6rem;max-width:420px;font-size:.88rem;opacity:.85;">💡 If you cannot receive SMS one-time passwords, changing the virtual memory from 4GB to 8GB in Settings may fix it. Please give it a try.</p>
<h2 style="font-size:1.3rem;margin:1rem 0 .2rem;text-align:center;">📱 2026年10月 スマホ紹介(楽天モバイル)<br><small lang="en" style="font-size:.8em;font-weight:600;">October 2026 Smartphone Picks (Rakuten Mobile)</small></h2>
<p style="text-align:center;margin:0 0 .1rem;font-size:.95rem;">8GBメモリの1円スマホはほとんど売り切れ。乗り換えならこの5台がおすすめです。</p>
<p lang="en" style="text-align:center;margin:0 0 .4rem;font-size:.88rem;opacity:.85;">Most 8GB-RAM 1-yen phones are sold out. If you are switching, we recommend these five.</p>
<p style="text-align:center;margin:0 0 .1rem;font-size:.95rem;">💡 スマホでLINEやカカオトークをご利用の方は、定期的にキャッシュをクリアされた方が処理が重くなりにくいのでお勧めです。</p>
<p lang="en" style="text-align:center;margin:0 0 .6rem;font-size:.88rem;opacity:.85;">💡 If you use LINE or KakaoTalk on your phone, clearing the cache regularly helps keep it from slowing down.</p>
<section class="arrows-alpha2" style="max-width:420px;margin:1rem auto;padding:.9rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.6;border-width:2px;">
<h3 style="font-size:1.2rem;margin:0 0 .4rem;text-align:center;">★ arrows Alpha2 <small>(FCNT製・8月27日発売)</small></h3>
<p style="text-align:center;margin:.3rem 0 0;"><img src="data:image/webp;base64,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" alt="arrows Alpha2 12GB/512GB" style="max-width:100%;height:auto;border-radius:8px"></p>
<p style="text-align:center;margin:.4rem 0 .6rem;font-size:1.25rem;font-weight:800;">ARROWS　ALPHA2　お勧めの12GB　512GB</p>
<p style="text-align:center;margin:.3rem 0 0;font-size:1.3rem;font-weight:800;line-height:1.3;">8GB / 256GB<br>と 12GB / 512GB の2モデル<br><span lang="en" style="font-size:.7em;">8GB/256GB and 12GB/512GB models</span></p>
<p style="text-align:center;margin:.2rem 0 .6rem;font-size:.95rem;">12GBモデル(512GB)は楽天モバイルのWeb限定販売です</p>
<p style="margin:.3rem 0 .2rem;font-size:1.1rem;"><strong>💴 8GBと12GBの価格・分割払いの違い(税込・楽天カード払い)</strong></p>
<table style="width:100%;border-collapse:collapse;font-size:.92rem;text-align:center;">
<tr style="background:rgba(128,128,128,.18);"><th style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"></th><th style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">8GB(RAM)<br>/ 256GB(ROM)</th><th style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">12GB(RAM)<br>/ 512GB(ROM)</th><th style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">差</th></tr>
<tr><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">一括払い</td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">85,900円</td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">109,990円</td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">+24,090円</td></tr>
<tr><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">48回払い<br>(月額)</td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"><strong>月1,789円</strong><br><small>(1回目1,817円)</small></td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"><strong>月2,291円</strong><br><small>(1回目2,313円※)</small></td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"><strong>+502円/月</strong></td></tr>
<tr><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);">24回払い<br>(月額)</td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"><strong>月3,579円</strong><br><small>(1回目3,583円)</small></td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"><strong>月4,582円</strong><br><small>(1回目4,604円※)</small></td><td style="padding:.25rem;border:1px solid rgba(128,128,128,.5);"><strong>+1,003円/月</strong></td></tr>
</table>
<p style="margin:.3rem 0 .1rem;"><small>※ 1回目は端数のため少し多くなります(計算値)。48回払いは楽天カードのみ。実質年率0%。512GBの金額は報道(<a href="https://k-tai.watch.impress.co.jp/docs/news/2133368.html" target="_blank" rel="noopener noreferrer">ケータイ Watch</a>)に基づきます。最新は公式で確認してください。</small></p>
<table lang="en" style="width:100%;border-collapse:collapse;font-size:.82rem;text-align:center;opacity:.85;margin-top:.4rem;">
<tr style="background:rgba(128,128,128,.18);"><th style="padding:.2rem;border:1px solid rgba(128,128,128,.5);"></th><th style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">8GB RAM / 256GB</th><th style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">12GB RAM / 512GB</th><th style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">Difference</th></tr>
<tr><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">Lump sum</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">85,900 yen</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">109,990 yen</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">+24,090 yen</td></tr>
<tr><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">48 payments</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">1,789 yen/mo</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">2,291 yen/mo</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">+502 yen/mo</td></tr>
<tr><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">24 payments</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">3,579 yen/mo</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">4,582 yen/mo</td><td style="padding:.2rem;border:1px solid rgba(128,128,128,.5);">+1,003 yen/mo</td></tr>
</table>
<p lang="en" style="font-size:.82rem;opacity:.85;margin:.2rem 0 .6rem;">Tax included. The first payment is slightly higher because of rounding. 48 payments: Rakuten Card only (0% APR). The 512GB figures follow press reports (Keitai Watch); please check Rakuten Mobile for the latest.</p>
<ul style="margin:.4rem 0 .1rem;padding-left:1.3rem;">
<li>メモリ <strong>8GB または 12GB</strong>(12GBモデルは仮想メモリで最大12GB追加) / ストレージ <strong>256GB または 512GB</strong>(microSDで最大2TBまで拡張)</li>
<li>約6.4インチ 有機EL(最大144Hz・最大3,000nit)</li>
<li>SoC:Dimensity 8350 Extreme / Android 16 / 5G(Sub6)</li>
<li>カメラ:メイン約5,030万画素 + 超広角約4,990万画素 / 4K 60fps動画 / フロント約4,990万画素</li>
<li>バッテリー5,370mAh(1%→100%を最短約40分)</li>
<li>約155×72×8.6mm・約187g / IPX6・IPX8・IPX9 / IP6Xの防水防塵 / おサイフケータイ対応</li>
<li>ハイレゾオーディオ対応 / <strong>ワイヤレス充電は非対応</strong> / OSアップデート最大3回・セキュリティ更新最長5年</li>
</ul>
<ul lang="en" style="margin:0 0 .6rem;padding-left:1.3rem;font-size:.88rem;opacity:.85;">
<li><strong>8GB or 12GB RAM</strong> (12GB model: up to 12GB more with virtual memory) / <strong>256GB or 512GB</strong> storage (microSD up to 2TB)</li>
<li>About 6.4-inch OLED (up to 144Hz, up to 3,000 nits)</li>
<li>SoC: Dimensity 8350 Extreme / Android 16 / 5G (Sub6)</li>
<li>Camera: about 50.3MP main + 49.9MP ultra-wide, 4K 60fps video, 49.9MP front camera</li>
<li>5,370mAh battery (1% to 100% in about 40 minutes at the fastest)</li>
<li>About 155 x 72 x 8.6 mm, 187 g / IPX6/IPX8/IPX9 and IP6X / Osaifu-Keitai supported</li>
<li>Hi-res audio supported / <strong>no wireless charging</strong> / up to 3 OS updates and up to 5 years of security updates</li>
</ul>
<p style="margin:.3rem 0 0;">他社から初めて乗り換えると最大16,000ポイント還元(事前エントリー必要)。256GBは、このポイントを使うと実質69,900円(下取りなし・公式ページの試算)。</p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">Up to 16,000 points back when switching from another carrier for the first time (pre-entry required). The 256GB model works out to 69,900 yen with those points (no trade-in; Rakuten's own estimate).</p>
<p style="margin:.5rem 0 0;text-align:center;"><a href="https://network.mobile.rakuten.co.jp/product/smartphone/arrows-alpha2/" target="_blank" rel="noopener noreferrer">▶️ 楽天モバイル arrows Alpha2 の公式ページ</a></p>
</section>
<section class="pixel-11" style="max-width:420px;margin:1rem auto;padding:.9rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.6;border-width:2px;">
<h3 style="font-size:1.2rem;margin:0 0 .4rem;text-align:center;">① Google Pixel 11</h3>
<p style="text-align:center;margin:0 0 .5rem;"><img src="data:image/webp;base64,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" alt="Google Pixel 11 本体(正面・背面) / front and back" width="240" height="240" style="max-width:100%;height:auto;"></p>
<p style="text-align:center;margin:.3rem 0 0;font-size:1.6rem;font-weight:800;line-height:1.3;">システムメモリ12GB<br><span lang="en" style="font-size:.8em;">12GB System Memory (RAM)</span></p>
<p style="text-align:center;margin:.2rem 0 0;font-size:.95rem;">8GB機より4GB多く、アプリの切り替えやAI(Gemini)も余裕で動きます</p>
<p lang="en" style="text-align:center;margin:0 0 .6rem;font-size:.88rem;opacity:.85;">4GB more than 8GB models: smoother app switching and AI (Gemini).</p>
<p style="margin:.3rem 0 .2rem;font-size:1.1rem;"><strong>💴 本体48回払い(楽天カード払いのみ)</strong></p>
<ul style="margin:0 0 .1rem;padding-left:1.3rem;">
<li><strong>256GB</strong>：1〜24回目 <strong>月1,990円</strong> / 25〜47回目 月3,839円 / 48回目 3,843円(一括払い139,900円)</li>
<li><strong>512GB(おすすめ)</strong>：1〜24回目 <strong>月2,990円</strong> / 25〜47回目 月3,881円 / 48回目 3,887円(一括払い164,910円)</li>
</ul>
<p style="margin:0 0 .4rem;"><small>25カ月目以降に条件を満たして返却すれば、残りの支払いは不要です。</small></p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .1rem;"><strong>💴 48-installment plan (Rakuten Card only)</strong></p>
<ul lang="en" style="margin:0 0 .1rem;padding-left:1.3rem;font-size:.88rem;opacity:.85;">
<li><strong>256GB</strong>: payments 1-24 <strong>1,990 yen/month</strong> / 25-47 3,839 yen/month / 48th 3,843 yen (lump sum 139,900 yen)</li>
<li><strong>512GB (recommended)</strong>: payments 1-24 <strong>2,990 yen/month</strong> / 25-47 3,881 yen/month / 48th 3,887 yen (lump sum 164,910 yen)</li>
</ul>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:0 0 .6rem;">If you return the device under the conditions from month 25, the remaining payments are not required.</p>
<ul style="margin:.4rem 0 .1rem;padding-left:1.3rem;">
<li><strong>メモリ12GB</strong> / ストレージ256GB または <strong>512GB(おすすめ)</strong></li>
<li>ハイレゾオーディオ対応</li>
<li>約6.3インチ OLEDディスプレイ</li>
<li>カメラ：48MP広角 + 13MP超広角 + 10.8MP望遠</li>
<li>バッテリー4,985mAh / Gemini Intelligenceに最適化</li>
<li>SoC:Tensor G6 / <strong>ワイヤレス充電対応</strong> / 防水IPX8・防塵IP6X / おサイフケータイ・NFC対応 / 約197g</li>
</ul>
<ul lang="en" style="margin:0 0 .6rem;padding-left:1.3rem;font-size:.88rem;opacity:.85;">
<li><strong>12GB RAM</strong> / 256GB or <strong>512GB (recommended)</strong> storage</li>
<li>Hi-res audio supported</li>
<li>About 6.3-inch OLED display</li>
<li>Camera: 48MP wide + 13MP ultra-wide + 10.8MP telephoto</li>
<li>4,985mAh battery / optimized for Gemini Intelligence</li>
<li>SoC: Tensor G6 / <strong>wireless charging supported</strong> / IPX8 and IP6X / Osaifu-Keitai and NFC / about 197g</li>
</ul>
<p style="margin:.3rem 0 0;">他社から初めて乗り換えると最大16,000ポイント還元(事前エントリー必要)。</p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">Up to 16,000 points back when switching from another carrier for the first time (pre-entry required).</p>
<p style="margin:.5rem 0 0;text-align:center;"><a href="https://network.mobile.rakuten.co.jp/product/smartphone/google-pixel-11/" target="_blank" rel="noopener noreferrer">▶️ 楽天モバイル Google Pixel 11 / Rakuten Mobile</a></p>
</section>
<section class="pixel-10a" style="max-width:420px;margin:1rem auto;padding:.9rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.6;">
<h3 style="font-size:1.1rem;margin:0 0 .4rem;text-align:center;">② Google Pixel 10a</h3>
<p style="text-align:center;margin:0 0 .5rem;"><img src="data:image/webp;base64,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" alt="Google Pixel 10a 本体(正面・背面) / front and back" width="200" height="200" style="max-width:100%;height:auto;"></p>
<p style="margin:.3rem 0 0;">メモリ8GBでハイレゾオーディオ対応。もっと安く乗り換えたい人向けです。</p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">8GB RAM with hi-res audio. For those who want to switch more cheaply.</p>
<p style="margin:.3rem 0 0;"><strong>💴 本体48回払い：1〜24回目は月990円</strong><br><small>(25回目以降は月2,750円、楽天カード払いのみ)。一括払いは89,760円〜。</small></p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .6rem;"><strong>💴 48-installment plan: 990 yen/month for payments 1-24</strong> (2,750 yen/month from the 25th; Rakuten Card only). Lump sum from 89,760 yen.</p>
<ul style="margin:.4rem 0 .1rem;padding-left:1.3rem;">
<li>メモリ8GB / ストレージ128GB・256GB</li>
<li>ハイレゾオーディオ対応</li>
<li>約6.3インチ OLEDディスプレイ</li>
<li>カメラ：48MP広角 + 13MP超広角</li>
<li>バッテリー5,100mAh / Gemini対応</li>
<li>SoC:Tensor G4 / <strong>ワイヤレス充電対応</strong> / 防水IPX8・防塵IP6X / 約183g</li>
</ul>
<ul lang="en" style="margin:0 0 .6rem;padding-left:1.3rem;font-size:.88rem;opacity:.85;">
<li>8GB RAM / 128GB or 256GB storage</li>
<li>Hi-res audio supported</li>
<li>About 6.3-inch OLED display</li>
<li>Camera: 48MP wide + 13MP ultra-wide</li>
<li>5,100mAh battery / Gemini</li>
<li>SoC: Tensor G4 / <strong>wireless charging supported</strong> / IPX8 and IP6X / about 183g</li>
</ul>
<p style="margin:.3rem 0 0;">他社から初めて乗り換えると最大16,000ポイント還元(事前エントリー必要)。</p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">Up to 16,000 points back when switching from another carrier for the first time (pre-entry required).</p>
<p style="margin:.5rem 0 0;text-align:center;"><a href="https://network.mobile.rakuten.co.jp/product/smartphone/google-pixel-10a/" target="_blank" rel="noopener noreferrer">▶️ 楽天モバイル Google Pixel 10a / Rakuten Mobile</a></p>
</section>
<section class="arrows-we2-plus" style="max-width:420px;margin:1rem auto;padding:.9rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.6;">
<h3 style="font-size:1.1rem;margin:0 0 .4rem;text-align:center;">③ 富士通 arrows We2 Plus(未使用品)</h3>
<p style="text-align:center;margin:0 0 .5rem;"><img src="data:image/webp;base64,UklGRuAwAABXRUJQVlA4WAoAAAAQAAAA3wEA3wEAQUxQSIcJAAABx6egbSPnEv6sb/wEEBEZ3JQuG7IlKe9kKf9E0LZtG0c2OvX/j5N07xLR/wmQdKoNrnESydUDM0DiV7gvMrAyM8AX//9nI///d7s/o3amY8/srPftj23btm3z1J9j27Ztmy8ba81as33eD9qdTuedSTZvJSImgAosEBgzfH5hZrI3kfZvXr9y/iwQbPauFIF0ZW5qslukErso9nOzm7F/787NG1fO3QYCkXopJDma4cXKxub6+urS6uJ0AXD74snnHnnnhBi0VxQiHHzX1x0/sDLfTanO8c6V86de/L9/fRxCXRAgsBns9qbmNw4fPbgx25vKGOoBAXzzd37uKn3tDfVZ+oRPfIf1lEFXKDH07GO/9hcnkSqdkLDNcC2uLq+sbGweODCXMzwaJBAYbDH33l/xMVOg8SVm+VO/9eOAPkKgymTAJgFe+OVfpK+KJiGiGQxFZ3Hz0MHNlYX5xR7DPUyMLJvXfPzr8NhsPviz3goDopobS7z44R1UpQQCY4bPzs8srh08cmBjrtNJGHRECLH7kiNv/VppXGx+4FsTLap9dHLoQ16FVYkkiRgZni5vbK5trC2vr8yJ4TYIxB4MMLNmj0UcfltMoPrHyOtOgCuMBgQ2g/lUb3b98JFDq4tT0x2GRkAg9rQcRBgHcyvEQC0MZnZJqIpIguiBwdm1peWVjc3N9YVOwlDbQojXT/ss47wvaqQIiamWQiKaQRWd2Y3DmwfW5peWZnigLUC8XjvceBR7d8wVoxoxuAWuABKyMcN7C7PzK5uHD64tdrs5Q+OAECX52CPaJV4+S6RWimvPsb9LSDEyPJnf3FhbW1s5sDqbMdwGgQbK9PevyrsQdeo/CdTMyIvPaj8SaMBmMO9NTa0dPrq5Oj8zN8FQYxCilKNO/bbYRcU/p34a/uyc4n4iIRx54PTa4vLigUMH1+a6uRi0BwSixK2ffla78df/G2LtgHD9d9kfBULRDCrvTm0c3NxYXVpZnhHDbQRiP4zJIz+DRtL5XwbXEOuP/luxzKQQhHjg5MLM3PLmkc3V+V6vYKgNCMQ+Kr7v5EjiF/9XppZc/SFUWpL6DM9mN9bXVlfXD6xOd8RQGwFiH7Y/KIsa6cNfR6gj8szHU9KBCITp6ZnVw4c3VhbmFrsMty2E2Mf13suM6vwDElNP/cGbMZRQIMKxVx571YmNmcmOGDQGBGK/F8vH0M4Sv+p9UT0JvPqtKV8R2fyA93274wlDo4UQFXLirRj5NcvUVfM+uGxC5NWf+ZGvBGxAiIqpPq/F2on64V2Iqini3TtWydD7sE/dwBaiopqVYHY++TaY2npwlVKVOfwBG0QCFVZMT7BjMbOG6kvvRKlYrJ8gEqi4vamdwcpUfRHdI6hMSKdwoOKK7uQo6xPUV2u1TMRWpBLnBdqJWSzqDPOUZ+TUBVSJ0pQRpxKrtsAULo1w7hFMJZZGKai1OWVp3f2TWJVGz3GdybDKIeovnk1MNU+ptYGSNGd/EVGVNYrqjFFZwM88hytTvS6NwCcethqJslR/6kOCabADr303hybLvGPParJi9r6myRYzbyU1WXBogUZbHCtQkwXrmWm2F2i6e7jhKmi6k8bLjVf/DZr0EETESvdwtPl6899qvFr/t/5/E85iu+Fqvk3ecIEbr9b/zbgbr+ZbjVfr/9b/rf9b/79p7NhwGTVaUvGNn2fVGdc+r3zpSp0RDwPvmBpr9BBAqjOt/1v/t/5v/d/6v/X/GxLHpktd3Gxd+ndCkxW58cU/p9hgQbj+As221Wm4sJuu1v9vCO7Gq/V/C8Kk8bqJGi3xyC25yYLHLtNwbV1stsz2ZRruG6dQs/Xa4zTcn/FuMTRb90zDHdV0idb/rf9b/7fAt5uuSTVd/3wquNn63X8lNltTqzTaYmIRNVmwNNt0vXqGRtscLaKaLOjSdLvxesvP1Xi1/m/93/q/9X/r/9b/rf9b/7f+b/3f+r/1f+v/1v+t/1v/t/5v4e3Gq/X/m4hXw2WShgvSxiupVn4Y42o1suuMiOUg7lUqj9SvM5QF3Kp0d1GduV8O5gqqUPfBo9Tau8glACf7VequGfF6X64x1xElaJ6+i6uSud3fmXz+DvVVXKIkn71Mhb7BqKdv15l4GpeBufzfrkzmikY5u0WsK+bWc5Rj6P+ZXJUCJ5F3Yq6dpMZefxKXgvi3y8HVyCH+P4o7QTf+nVBfnrxQElGP/TmxGsGN/2XnTuK/9uWaYv5qW5Siw/av3QuuROaZ5/GOMP/3JDXVYfsfCeUAMf2i3KpAIb79V0qMKL76NVF1RP3s23t9lUXwO713rER8w3s4jBL8np/lWpL4/T+DhLIUfOiEVX363a+fiBpFDl88a9UQ88VLfZUG8uTb4coT4gd+GGLk4Hf+0Ej9lF/70RZl6uUFgquNnH9O4V2Q+dhpQt0QfPwk5SKKPhU3+D3fzmIX5eX3pYa+9atjoFzNrXMEW5VF/Yn3Z7d9bLNuKC69NeUrLvzLRUJEqibw1j20OxJHe64XcBCrdDC3/+JPt+YgIlDVECuTUex+L1OdCEwklLID//ujn/g+b8NwAx5ND9L+EoW7jNPcugauBzbb1/qxnDAB1l/3Tu90Yr7IUjFGg/0APUg7UNnYgTO3GPcLLxOia0A/cO55gkoKIupD3ltdX56enCiyvFNkWZ7macjSLOuTdrp5liWBPegHWQaNpD1kO+HSf23h8Vg8+w+3STCqcAbkf/sfMGUeFM3IaZplWZ73ybvdbp5meZYURZoXRZ7nRdbN006nyNMiyxOHNKSB8RuDd6BheoCGGRsFbv/5X28jxn/6d//kHqKPhMrFu6S942Eaj7FJgL/79ceQKXsB2okxY1QS0kRpkoSQpkmK0izvZGm3k3eKvDc12S3STpbnaZYkSRKSNCRJSBIxVoMBIYCXfv0n/odgxh8D//D9P//8gQww1k48No3L0u7Y2huWhtgag5EATv7rL/ydFdl/xaCGaEBgwBizB6WQJWmWd7IsL4oiy4s8T7NOkWdZXqRpmhdplmd5nmVZnqZJIh54+9wLT33Xd/9DRGYvWtz/p+94r3c8cWR9WpTrdhzNIWfvxtv3Y1rkjPnW2Ree/u+/ew4C1VLsVDvzTsyYQ5LmaZJlaZakSZakedrJsrxItm9d29o6c+kFFCJ7NQYB3c35ubmpiSKEgXD3psdWTMhj0PbZyzf7o6Hu/Fo3jubtJBkl3Dh76ea202JmdVHeHfvu7Ztbl7dOXQHJ3nMAVlA4IDInAADQ4wCdASrgAeABPmEuk0ckIqitpVFqobAMCWVuleuw3Bng7c/4ECzc/hPP0R2/ipIJdiRTpT+N/H8Wd2++16mvuq4BvmA82b8t/eB/fvUA/tn+j647+r+op/C/+R60n/t9l/+3/+D0xtUY7adzH+r7j3uJe53Smey/Jn8L0C/5vkn8u9Qv2JwLYAv0z+w/8Lw4tWvw57AP6u/9Tkefuf+y9gj+d/2v/o/dB8jf/p5wvz//S/+v3Cv5X/W/+h2Mv3F9jr9dipK7nNIZ5MpVEGTX5RlbF+1l2a6IaBqZ17FhOlmVdbWiVs8960nUzRsCxHGRfI/8B+fAj+RnLCXisuCKhcqrJl46Nx/OWZ3kSWX5aenZC6IN6iV3maz/w0jDImT8Z68PMSiBk+f5k+DkFfJhE+N8ttTdTgvgSZgv2lX1xOB6cNq58PlxaB13y+h1Uepf0gzpqW1rRFHDTP+wO5eDz4lkEc9Jdphx9zs7ilyLaw3d1Qjin9D/ds8YcSgklvMpFUJBuAEyTTO7Yltx/3rb7NfNRHgDSZqA2naacHJ5MQIBpepjT3LUUWhFQ2UIWwLglCWOStE5MMo9sySrCvkb82LfLQc3o4X1wHw/WjBOrSOdWL1yu1/x1M/msG/TgzG/w7pLAVN8rTXNkb4naA0Rh9qFzQWLN2CbBLfxXQoYh0vdbs7SI1MeLH/LuwUX04JCwHxrPTjBwoWhOgS7Chw33PzoqBlAT8UyySDkftfwsOrXSHPJj4lutguC7oXT4OUpVpIEkp5dAoNpQfy6o81tJnSCfoGB0gvHQcAp+I4pDxTntPkm5MBNsZPpYS5bVK7MZGMJmBhRx2q1OR3TBsMteRgugryPhu5dWrqUZdIxykekU6lI08kmh4fPeu7dqkv9J1u6sXwNnoG+eTGnwcpQ6knIMYsXepGlHg9cXiWaPBlUTYmo7pLe4gZyYkYoQTAiVbUkkg8mn87/8L8RqXtVyOi2Ht6japbxn0YHa+e2aKnGuaRPhblw6X3fiEX52y1Nwz7CyPiDCDGON9WcvbxchvJpk22LiOLOVr6iE5Fmd7n9WEvNxZXQ1ug/ubicfMZLVccBFd/nOt1Yrr7EpOHoZfQedQnMPlUA5Mo62Pjj1ARC+v8fuzRgT9wk7r4+IGETFO7pE9bj/F2RaVwFsJuN3fIiyKwo2zy4kH//2psJC9KfaevSKge4HZXm2gRcZkoKcGOk0XZFm8L91OXtIErKYYdnQ9ZkgFuIGHuASTzteFOSiWHprQcAtNnAzYne3xKJgiXB8EWfuqbwxebhYxxB0/yYyah5+eGiCwJiR2qOVsm2z0rs9H/eHtE3T23ThLiEeLBDt/VPedpFeydlPSaLxHC595t8H9rfqdb/hGwAhiGqUSsRKnWhVhuJbnsrAuXfDDGXTk43ula+Fu4gQUl3sNmv5+QyMMFlOsLT6vFop10YMHBvAiMZ6YPT96dYXe0H8yt8CVjyL5WPUGBgrmuHesOp7+dQP/xkIKFOxv/L1OzjH1k2yF2+qlYWJMbGEXJxjtJZbl3Xv+EEFEtTEKEwliEhg/vfggaiMHjcbh8KbK9iUt/W9e+YPebxG1wxwCGeh5RejuLoo+iH8mGti430XRCDOjlxcky3DkD1wJqm03PIYMRIU+mJMBqFR5008pcWT768K7nG0aU0yfMvgXHo3jvNXCVvuD4svq//DIfsXUDcRxEbvkMdagRaySfWMZlOH1zoVJKsk4LwVX8g7ek3e40KRbjNmIw32IYr14Can8ywlGudQOYjzRqy1VGG99xEUzLhDOfn0C3p2f4+iS7X+rI/NkSHuQ/SjIT3y+mdpRqOtgzQuCIkvobe7ICH8jrs3nSk5Im2aTjbdDXcPhTI4i+Kka/n4p+8PM/sM64p0CIaYv18V8MxF6hSPV95HvzL9zvzBnsntCtDxLFr0IDKOFPCrEr8GcUJ3nImAN/FdZOWh8FEofb6l70bB1xpeHYiqs7jfAgRPxriuaWuTEB3oR7CveZrc/26MsY3+XBSOeOWtBYHUo3RjT4NfJpbWSGmcuzsYmNlJ8jxT5VSDUhcaPX9WGO3uDxLYlpUwavAL05vS8ysRoveOK4yUQDhG+LjfVnL0CqKByQtGNlBNNjC+v5m8HfrqO4ESK+lLAHFhllXMtm8+SiAb+K/bqpyhTX4ro89aHwlnl85WKSy6AO5Ou7vyWBrfgb8yOFpq1ComhwJiJ1coRjZU8pvaQTEdysa+YLip1NxQJ1knugVdrg90upAIw4SPS2u5nalruxhRW4Lga6XIcI8cD0va5Y8YoZdj+aA20ahTgjxN5pSzAt8t8B6bUk1G+tZ++7iFtoIr8VpzKtcFKE3KCjJRB2osoStJWW+p51iOx55f+qUg/ooJlTg8B4Qcl3/Vzs+E4vlNyreUEYtY37yoQsP/TOH4oFD06fC2r4AAP79Nmjx+S0OH5UrTYw90XCprs3YTuRNuE+3qbLMPXLXegWaYt/TC4/dVgf0ZBcaLH5op+DRit3LiN/7SkD5yn7aNBYLZXEh9D+HX5fXJVVmI2wqg473nDLQ65yf+s34wTByX1wNObOYfq5ENUSZflejFV2+ZKByegy3/aIm7UdtG8MxFF2c6kTiMi9W2q5zzqUz9C7XLUOSUzcCOqSyx7oCXL50wSw27VoSnjZv/9tfVvD2Gh9j9O98EHePafLim8LN8KHw9zk+bIe0+VG/OcsDpRd7JNT8hkN+4gYne44+BlV2HzIIPCrebE07FUHF1boEuhAshK4BND26oLQLExdbL9IRr1vGXs767oI2v/QAJEY2BWDjEUohsgwyW6ZOPkoL5JzOesrTzpEwJ208THmaOG6tZkfvalMzQgXrNupbIoLm6fyB2Zlg7dTG2A6JWZ3vfq/vg5lt4YznEWNoEBUse+XXHcfrslwwBzQvUN67zruzUkwt1aXrXPy0VYXGhYVe2xnG1URi+NhgDikmm1cjEIvT4VoCdtqUbS8gW2Gp0JPpVZLKxCVtmbFT2vati9JdZn/dGZj7S8VfuDGaYg42S+X3ApfvrGPzDLAj9peZw+qbPMzHfkzDnW0mRoXUhNSROuA0uumIv1yZk2Q3xBDQbyGWxFHhLE87buMaZa9ifAXDFryJ26P2BXnt35srz0RTaxoqCunCeEZGnnzua+1KZF8sKW/NZeYE//r/eVJ0L1hER0U/zLRAirBxV5Yhzq7tmwqB2odgiVYiJEI8lQI/ohSHAwtiHq8YcwlXDIyyDGNoXeLv3QU/ImVY2FzdbcMgsuXk3RUDVD+nCND+bwIMEh6MJnQn60gzXVd52KKr+GdRQy0d703z+otyfsm/CXJDSfaLyy1QQqEzy8x4//8/iGw/l3dmx3saf/VwhW550JMtx9Wdhvr1BYXsmWimVhYJBpMZmevptbMHBkzDMqRm/GCe1Ce8gGuP9qd84btf4OvE0dPNMzbc3SJCoSByM/VyzHbuwe62HaW6DY8Yh7tdWtUIXsfVCc35Ie9kZEc4CzPxY6GqW2xgFP5yXZKdiC/QmtN6j5u73NNXaz9n/xfPiET0cNq+UStgToRYP8Y6mF9bWeShHV13qV/gXy073axpCwvhfsYKQD6JsfBJB0aPIaCJu80klc0bdJUI23uSa/+zMZU/4a6Bey5LLWN1ozxUUSbmV5Sur2VB3pmixVhcsx5IsHvPlXnSrL7ejM34ZBsBKUAa9hE35cyh7g7EtzK4EQihSOvc32wTDEpn6QzOdfmqDhenNXpBO0Z3FdGj7ZUS4FF0KKJpBc/1cawvNxUybzRl31zJDB7vSHF7drbESAmU2w2WJmLqFIMUL2n6D1rCb7v/VA9hzNg2fi/2R1RZ0efgBLhVNvSsNWQWk2zTMJitIDPBpnnka5NxlGBarLboyzyjl6/8yhx23cGwH9/Z9q/+8UzJMVFNTQJ1RI0NLn7Tt+c7vIZ9mhJxpBIn9dCSSW4r+ZyeCFKK3SYWkvtCh3pe8FzxlXOFugkwARwVQ6uRd6jgy+fOnd8bFGf2X8B+cyJXXd0XMBTtwU+cG7H1EhyXfu0zjajCsI5Pon2oQkPVAw/5hcry188fBg3JM1n0eP3/ALYSZiD6dhV7gqH/o8FoR+EOaBLCszu6E/5sc6Pt0TtZue1/DcvZ5r2ntHU5YKozKKHC7l/Ad1r6sFn+pziev4niRaZFHKWftfiu6i9uv60jw22+AwrQDJD48jFjUYJdArlnQsUf8FsTsHzPXqEwFqtxFPxJBBfV8+8FVu3JD4YPWQCgEGWF8d2gi8fpiDUMWnZjH8DlLP5XRDq3UOK9P6hD8WHqQ9w3502Q6rWJvWcOWmiGj1JNwpHHkMXGMh/04qcQkWnBllB52s1XRPX9/KBzl8BYy8iKJynuc27lIzdnTsi5iNtmae4cWt6plNmDgXKj4GUiUkusotqiOzw59WBHt2NZEzFsvliVG/F/kGTiPPBeyjbi8acMQKLHJBbbsozXmrVdBhO6g9NH/8nFbeMsWZWp4LDLke7gQfdN0YxjP2GwnHD8LifuX3B7MeEr8lWbxc3+73pJnWbmt+/4L/FsHszkYDnF70tbSpMDH91hZed9diPl+aLECx3IzMXNuPrcvRre/tG01fPvjXG8hOsLguWqrhzEfynq9Z2sg+bPXtrMZx9pF0PzDGRhl8gmDIWT2DtPUrccBkmcVm3qw2QkLx4kCMp8NgnNXwxIW+9fqeNeXb/FA1z9IZWBAYJrioLqZWH5f83ZAkZka8FRV/LSeniROlFcBbhXXx50vWzz0mZntub4VjPkiiKVW76pkq1Me77N1VMbXlI7xEZAK4xPEJ9kvZoJlYHuLB8bfB2AveWpXn7cmRZuEgY0MdU7NV//m6qYPbPiTMSk3OZGs9JWej+m5oRzH6xCw8Z/4jDGHV5eaZrHvu7XX35XfcuWJp30vmeXYFcEKKD5yxAx/twqEpUKTPBjVde/wI5PHSkH3PhV83PFQ5ePde+wBZxf8stk4/NItFSmAKxYK4x8s5rn39TYoaeCD4lvqDVeYDWb5ANoKHJw0F6uD5oqydMSt8wxyUenIyCJtuqRSb3SlN4kWa1G4RvTce7hFDJSUZTqLhC3llM6MUmZ3kxJUDsEtHSgfeLYmVQDnepMz7XzUb6BDziKqveNzO8E8K32FWjShqaGzzFH0kf9xm5Y+0BktTyBRxTS0xsEqUtN18y0dfbEVOni5ccwPgZb6Sxzx6uCeNBjHVtkBNgYGvEWBE6G4HhCQnDDxwBnglwp2Mx+hYblVwI839n3U/C0DmSYEeSiOkCpo2lBV0Y4sYbDHP/H9DIFoxSfp76JRGEgeJQytf+giAdplUZaoDSfZ3X5Qt5kF6A2ro20mV/zliGj7Hmpr1eWA1NlUAZVs9HInTdFguuHVyGEvRNaS3KwoUY4BovWXjfVLRE12zX61CVc8OutJu21PVmqCL3pcNdVxSgfZT8ZyOp4Vccd7XXfkSL6krvHTvz+65SK8MR7XLXXbgzhCn/DJrzi67RLEI3owGfthU6AVkT4INx1D2tYthrGDhg5SBSIsFO3euasD/BeozMpph9V8liBwxPjSzpDREXc1wcN4fpRfDhjr3ZVodh87ldNEaULWHUmJS/RXSQ9XUmtH4HmYEIssyAZVZDwzz2iyIfWTg40LnDubWuTHyT84qIeXep0ftCIodMdO2cmsL6bhJCWkGsuPE+Oypb9qna22VMLsjeWoVF78IrBGHwfaPGyYd+lLuHKD/rGLguKDuaj0H5sYdA7Ys1NoccHwygCxDRHArl/C1bAPUkGMGG1y5Ia08EZsqkBifDeLJ/9VpWjFvEEGfA6g2EWXZFqZaLjvJkj8XUfBPeOhKM4B+neJwq3ckw4sl9NBwhKfOPs7DRbqzwy1O7Alww8QwNg0xIW1cOGXMdz4Ih8uMwX9ApD3uB7XswY3K0+hwjyerrZjBZ6esL4ymmbzV+z0xKv9zMmlrNRYmYgl/nf2Sn9MDp6GeLB4bFbi5cO3DvXCIjO3N9v80X8qA22cfmrJHZtD/Sw+c1RrSBwZ5m/mfUfnE4PERJbK/5RJtMY4QWh5Oe6HmzWtSY8Y6IRXYrdS1ju6GtFcHfh7yurpG5+GN2eprapl2MZM2rFou5EbDco1LbJjqxslBsOEkU6giZA+N4zvpxTDZ/H9G9NbMuThtNs1xGPiGXGnK6siVC9an5fIP4e+vBmv6+PRWQ/Nar4vvsYIg2KrJLAvohSMoq1AsmmAzFdIXMOAW/WNS9q8jNokR/pb1nj8tBfgIC7meg3jT51oFZulV6Q/Zl9CDD1Lm9k5/5vCyaNsKMTeiHxQmL1H10NtIiM6XiUXTy0QerR4rZJp7gAMhaahpX1DIhxICuE9HQXZ7RcyX9Obxq+LPER+Lct3AIbNjoXJlwn4qkexIBEXr/k5MGMT3yHJQUa37pqa6dibngdnSJOE0vMI0sSgnVrHr5xgL/lwDMZvPr1MqFnIkLOOXuMcblXk0Oicxr3iFfTrJZgOWtL3tXpdXr5Km3LZGxi5cQzNeCdNYDdkk69jdGp63z3DRFSIsQgNEetHJQoxYylmrbSN+mcKYEE3kvycUmT7l/tjwjqkhwPfJydBjyr5d2hPZEBBpQ4NzfP5WctkVq4MlXd9gZxzYPhbwDHCneInI5voRgahxSFFI9qblvoAIWepeD6sko8kUqfQT0BUrl2X+Z0m58c2I3YDBudX6+aNQb7NjFNe5CAEodNc/iXESORa2YizfkBZDjQ98bIgXM6Y/PNY3ZFFqSOmPwiVAADo7wn8t7yYA4vw/Gt4hZ5lj+Rcz/4ZtO0xP+e9RjBkdpEWvP7/Le4X8+FHS1DIDdk7CP4FHji7LyDrFh9CMlB2LALl/K6HPqJutJlBQPYq8QqqQ5mPmGs6I9Ig3yUPn8HbJSNloeUbto3rf+7Gsw2Bwv1TaRpiHpyHcFNQJ6LjwZV5dEHzi/RMsVtsRvM4lHSjX6A3Q/QTxZuvJ36tUWeIOWl2Zgy72L60lWRJY+GD2kNh5Ga8FPY7L9OGjV1v21pwQA+BUTVGr61AbHNfX7FlCSVXlHnDLZxr0AOtR+rVVmU9qtI/YH51L2I+n2KMS6quzpVmuPJn2QpbasJL1UFdUlO/VaXwwcwqgOb9cOP1jKL/Ov3NzpEcglibiTsgeCZWODZhH/hpS9Jnh7/rAuTcwXQCBtE9QYmCd5LuHzf+NPyx6hmdORWRreHNylDIXhk9gA5833TQPsvrlxpxvhon/ylNjJ6lIoax+qWvrv1n38GA1zPQu6jrZRJj3t3IkeixhI7ARltUHnZ1ocfsT40mL5U04HHqLBdimHhdpSNnnsl75LydxPY+qByqqMCVhghVjEYzSn0F1FGBa05Gv7sJ7ItFAmOfMPjsNe6TFrrkixq+oaEAH4w7JsnhmPQMVvrqMjnpUL13HWbrGQk243L7SqmnX5iIDa1nGliE6Nhp2EGG/qdUfvlXDJdOQUb4ru6lfeVMF2eanP7+JL/KOLs6joeyiRDkc2uYmeOsv/thl70k/7PWLqckB/nrJ8YKHzqZ/tZZqlEddhYdVWZYoV7UxzuaWB6j2kr4Dag4cOm+7gsKJQX6p7zRJb6HCfc/bWK/Z3s+5KTTLx24EOcdE29YUr+H5QhWAZDg0xj83iXo2dFRbue1MnezjRc98pBERoIIlo2cp/GkIBxBS1Tn60fgIItr9NlYlxk2hF9NaA+HB/3FdDNHErToIKyTZjXkMwSfFlytGKDYDTVkPLJ7iAFrvJlKIiFKoghgbApwALzh+3z7jc27H6KL11BV58olK3P/xZg1pYXShVNcasRBJEsBQoy3OTd7pzxuMAJ9Ikw5w7mxLztm7p7lCQxW8UprhS3nevruaulo80rCI5QG6pTUfdmpJcZg3aa3Fp0iH4tcikuK8kvHbNPp9YJ+k6GebX938zEU/7Gwv75/24//kFt/flr4rlmwkbP/Kj8TZYfhmzDwDT3wx0aPR3StRHoQE3qf/+U7nqfOpgMtP4nNfCOBkHeunZHynurwlit8wm20oyuXk19TIkNZznfEC6RJQ6g5nV9tSJJmhwdJ8RFmV22TTsKBRRFC9tsJur+uIq902N9L0jcq517mmOvQ+nXRQU8OPbVOsiaIKQasEe1VJujLQxw5a58N+flLewe2uDNF33uQIKvbBdmi4JnAzclj4XAh+C6zEjVn/5OFbXW7LVuVyMuOZdgwpCDF7TuZmVdKdBpU8i6CcNZxj6ZJbgCoF4wUon6qgNvmgf+VyYG1t//8THf0lPR52+Hi8uAUjsCxm+MRMcOD1gsdBSn7DEo2jM3IT5Rhoe6Nev+zcmrMaQ/rF1z2d8VAlgmWfaN2G8njegei0yW23NfWGiEW92bYwqjUvExNxalwvYIoeVnlmPfo7w1wSj0PB/lZfn0jrtE7oxlpWySoJXVb0eqMRMqxhy9Di+3R7+9NThLdqQyarwjVbHzqZH3J9PktyPN4N2wsOjJAtt6ewayWvOpPqmxBhjegPOhH++DF7Fx5v2MiBvuZuBDtUuYhfiUNuC2zIbYeVReTgeffKz3M966BzGc0Ei+3d4i+ngRwto9ch0/OcatP3+fFvq3gcucbFRMKtOBvd5xI3gMpiI4yEQrlxaOwxpEia/7G/d7N/QzGPP21L0eM/a3oL/szwqRa3zDsxWJqqgG2nNMwvB8zp2BGbs3Oe8Y/wo9oV1VQBzyrxlTbes4aHy/SLIEqOAgpKef0D4Yl1AjxyTTfjod64xMIlC3TTQpQp5FMaxTSj4cZd84kvrzGUfUoKpf2y4wMcqjXqBXu2N+UUugpornfdronFD8pRFacbmrgbc9c4PqTbCUW14MxpO18A4XWLJx+zUpteDsC5pXW9+fcMU+oIYlJxQHYPiLOduXTrWtwp0JagiML/siVjmwSnQZ+JferxvzMnJg+Etlp9o/VATD+AiqtNqM+Qxj6O3OEYqEgij3IFXketuvYslSrk+wzTWsf8w4OBJhhA3V215Ez6DIwvB/ZhGBzfc/KaZnUrteaeDEx+Inf35vaxzBNvM67ez+JM6TI6BOIWOiOyQ6xDnxCoQAS5XfwjXUPUrdZRbEw/RUxeew9Ax33V+C0fAV0e81ooAtg0CeIAk1RXx+9Un5B3WiMcB1x5RPmw1/Hm14AMt8V3sOjiU6KLQAwAIAM0x6v47Tf/I6OxwTvrGG4L4H33VJvxH6s3364TusMiOz0z3JJUeUgN2rSZsIVtwHP83igdilxTEUxXugKazHuHtgj/5/pdbvD9HCzqC4eeF7/03uBp95TA4//W9DBMs3bAZZaKoJsG/Z8nqWZ3ELp3iyttbksgnHfhioS8OgQZH1ijRWo8hf/VAnVf+o1Lx97o/8tnBWLAfiRWNikneYwRQgF2yw85a5DZEhV0d/CHUdZrBSsr1esrm5DQfeXMwbP2bQQRNLbs86JeDIDBMaYHXm/TKWBH8S5tI1r2WUBNnrCnhn/C6rjZ2kk2B1wjWujvmKkVkduiJAvfl+KW6VJv/mDiPvKAQEF+Zxw1qnTEDOaO1yYqEWXiddqvT5jIjKUyr6soWple5QwAzdEg2IpAlX5DfPlgzA/p4PY9LI2WmqBwMZWC6fMMKcjtJxGlB07OXqYidNDuQ63/f3RuYu3V5jrZOdUzBQ52ixdFTwyfXDdllCiMuhJ9TmlPPrPMABISdb8QB+pguKdSQwdb+4aRUlXpGF4sStioQLzMObiz5qFpa2VQ+9ID3+wm3DrPmuc1AHF1s3Irqj8cW8bdsiZKfXZf6WuVGBqyvJGJDpsETlSXLSx+Q6eh979mEs8ut/JirMumtvdrQADrAmoxpwghw+4NX18HGwlUCERO01UpuyVHBTs5zlH5MKe6dkaFofTM9pTINR8pLVWEw01xl3Qp/v6y7ZY5vKQDejUPF978Cc4AQCwzNKI/+OqfGpUPSguHnsOxEZ3D5wF/JXaupucfgUnCpKYwJu5pBKEJwGZRmcvNfWMB4DPrKl2z1Fpbp+e8HUTWy5Ob7pSfwz1MHGnSgfPwyASAupEe0Z3CntiYWcl7brfw1ocQR4xTPVOeRKBm1/+i5feYtG/dDwR64DJZxGZB2GyncqQYp++mDg249vKduHdKceahG+vo+diqJMhS+AP2yB+SHSUmtA2Ig3hLBsX3l+4pYz4lmwNVYQUtwgBYKNsRBG7kWHi9BFMOy07c8v1qwbB9gWYb+5TaGrXOhNsigg4TfwBxJw8NEoV1Jbvs/8gbdwIOPOVB/LwZ96w3wvr9/KXHK+HkRjFYANj0J2iKe8owxFoy7ZxUL3a068Vq/n5RO2wJcYfkiDJWhIUpd06kAFERRuYhL0XzcHRjr/vW0bhToOshNP8YC86JtTTC7FLXOPIgL19eyrpEqe8IPXflrDASocFxCVdxi5dOJsQOYGBtMLELa7DWn5r0wcKIE+K4qPU3LeLgtIZFfuYcDZopXOv2EfPJzmSAvpY7FPr0iQW6mzt/ubuSf7ZdNavYvwrwudW0KiePyIqp6cAunM0l3AwZRt52dDy725KFvFYDPfWQ7TbLjvhQ8CG3HIjrvluv1kyaT2w68mY5ho3qHOIPfgAfsFd+zk+5xa1cU+T/MHKgYsBxVaZXywvBK8Pp23YHxzVWzRR38e1xCTsCjqY9KMn3Fwf3+aejQGSwcCHo3BO2XGYEM1KAfRI1svQlI7NjLDB2VldjyalNINGsNY+O5kfNI+dLP00MvV7ITWyzILEFFDagounK1gR+QOphwG1LezoauSJc2CeQQX09BWjOPoMwhEWT4Ej3ANG3meVjM51j4LvT30CPfQJIEzdGltNl8b1l/F6becG4GtJmffM8AgKk3Opj5LHtrfbj5GXPGC6ULEzqPU8XmJj5Ye/3GvAwKMGlrmgvGMS7qd9JoASXTEVsaKI4WgEwzG/03ZDhaY3LrcbtAz0DVZLkuPwLHe4AKgbymMjcOEtsuhzXJ3WSu0M9pLtFk+pBpP2KxZIb1aJ41jeomA8ukKvrcxUyqTi+znPwU34jC2XIPNK0GgvYWcCT421jrfGQrPJ3186urBUcSAEgUg+WwZ5KOjCUekCj/ekZA3AeP2eg2YO1oMlUuwbu2zDmcOFb1F/jKb7HW7kPOC59acyKYKJQxNfFq2Rchmkat9HGvFBIoAphJXNoZ8twvwR7ZHd02B/101pZ4DnS6sXUdAFYO3D357QiIXKEpzrm1ph628rZu/FcPkeWiCULXzssCqmNiWL8sx4J8YD9Jl6L4RT283FrbEHwqVI1VF1afoqQtKN/6Ux/m0ysL5Osqe9hMtj2M8H+Iaronn7H3o7Ykd2ccN1M4fVSpVmU2oMSfvHMLKXMb1ftZyt8rJ7OMPNQDVOXSxlB7GPd9h62oE7a0wMzqgkLvmLyFDsbZwE2jbnKASQrns/pwsy09tBT06Ejz1xi8D2ejZsk3cv/sEtyeDueh8GUQD2R1kTXuGmrQCcmzziSKHRsjkG0ABfVdcgFetqNYRigPRvoKSwq9qhTEuHTzk6GOJT98DLvSOVKPfuZcnAOXa8+Vmb6UclJ9tzYJ3DvrXi977ofMA/oY7gY9av1wDn6A1nRqYIxi2T3U+cFldu1Ist8XKtCQuJmM72Kh28pdAgAIwULnhpxe06zUrpujn2cN2pimNMma0/rDGbDCHG71zMGkrlFsaWMUnMA87zUfxY9pT5r8PLBiu9F8NlBg4KlnkUvuLFx1EDOMOqA1XakAwKfQHPyunumWMHdWVfNpgANYsGkLHGAtIEXyse8UmBKfDtHCVcS2I9pDBsFqSMpLPtjqW49Y5j1X1i/7s4Dh8Mb4U234hq16cTCWWgr4XG01mpgW9L4k/LmyYTrh1qcSO46a+svEOncJ3K6Bvs968RODXbHCWPxuH9DfdNqZbdorA/N+utd8YPz/MGpQ/CIFXlmKr2InRcqrZIdpVRkDYZ2jZDHL26bpPfgleejGGo7Ybtop4LgHyb4VN7ikS8RSHOg1kq1UcUII3SUCska5ck9mVoTniETcqBn1glZwHY/4mOpigepf8rGBXK06+qNUTyfB9KnZ1Tbt2qx/L098X4mDfw3/HvP5HfcpGjLUK4t+g/sEX3sCRBLc+q+NiiJlGAgqyfnTEldGqtzCq3DDsx4AEa2aQB5f05Khyr9oCexZJ8zsDdzsOAEEgtVfMxIN+mtUQmcSXAmOBz8MAGWUE9moZYhI2fnSFedxHj96Xz/kFKRWevOqRc0vmpSahIyqHIKnmcN7lXk71+ZVG1v6Wl+coZSLoLS3g3bMhQWuO6ox4tqQzSNdGVvJg/oD2FTYHVr5WgMuFKdxFvPgR2cN2iXYQv/39Nq86mWB9fNbqhqPpydnKveAc75S77p07QgItcUfV+3Lro7iUrg/uOiPyODGXzhmdee+59Qi3+3kjfjsxK8wYVjisz4ZGVkQ6iH/Sot4IO4+fH6AvpORw3sWRFEYykQKh1CLceQkPDbi7OYMU9i1HfvC2mhopbrz9JdCTWTH53Z18cMF2lN4V5G6TX9ZTZBXHgIPzvIi+6iHhoQ1ZV6wAlmXr2cUhudP2gNAIQyzJVYfynOyd7uuWzmT7scu7FH019CKxzIf00N7+Uu92ARmjjzxDIkaDAHrTkn1C56dPMgL0Fae3ecidRsjUBBemPs6Wc3bnPKasonPS3qaQY6+B+JcO47pg2HraknsBeVuBnhtwDMaH7zeD6F0aJDuw+5w7lw1qJro+y/7SF/r8QctTlfXtR8OB5hZ2+BficIhfQkzt/tyC83+PEe6tfUtBSt/h5cv/wlK8FpPKGhWdVcdK4FyMcfe8Hv20PrkP2lxX64PSEivk0T4VnQndI00RMLbU0Imd16ALb/LMCNXz8rOHQ5sGM62PosYwD1aaWfIeRefQ1tRavctsUS9NoP8s1GfB8nuoULnla9SIn8LdO6yGWh3itd0p9meH0gAB1rVysyZw28rIOb7hyVRWFDO1fc7Ff7/TYXW/nyKjO0V0CFUl45KGIsJN5sHFu4OOYq4cY/kj2BZudLl2VvGQy+C5RaNTxTvQarPkYbe2z7VObrTPFSOvAtC0MtyKzIQmTYMENVpOY987wMr359XbeF4ig6wqsTJWw5DmAD3LLT/hUolf7GCpfbtRxNCcJoF1Ii5MlGlaDJh5yRZDS9rVSddgRLa6yzzRIIEwwWZ795Hz+QedNgGSXShLhFWY9SW2MnUru2ICbynpGiAd9+4Pqdl4AoKlYa9sYXby5UJdczf2umD0FA59FEr0X72pA8sV0IoQT2ANY6TxGBK2OKxDO3wO02g5hB8sTZr/dI1kV1ExitwT+eEv+T65Lt4jDDrw6BBjgnfg3God0zuIIhnwmNSHm+6RyZT7HujnVYAAAA==" alt="arrows We2 Plus 本体(正面・背面) / front and back" width="200" height="200" style="max-width:100%;height:auto;"></p>
<p style="margin:.3rem 0 0;"><strong>メモリ8GB版</strong>と<strong>12GB版</strong>があります(ストレージはどちらも256GB)。国産ブランドで、未使用品を探してみるのもおすすめです。</p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">Available in <strong>8GB</strong> and <strong>12GB</strong> RAM versions (256GB storage for both). A Japanese-brand phone; looking for an unused (new, unopened) unit is also worth a try.</p>
<ul style="margin:.4rem 0 .1rem;padding-left:1.3rem;">
<li>メモリ8GB版(ドコモ・楽天モバイル版) / <strong>12GB版(SIMフリー版 M06。楽天モバイルの公式スペック表は8GB/256GBのみ)</strong></li>
<li>ストレージ256GB(microSDで最大1TBまで拡張可)</li>
<li>約6.6インチ 有機EL(フルHD+ 2,400×1,080)</li>
<li>カメラ：約5,010万画素広角(F1.8・光学手ブレ補正) + 約800万画素超広角 / 前面約1,610万画素</li>
<li>バッテリー5,000mAh / Snapdragon 7s Gen 2</li>
<li>防水(IPX5/IPX8)・防塵(IP6X)</li>
<li>おサイフケータイ対応 / 指紋認証・顔認証</li>
<li>脈波センサーで自律神経の活性度を測定(We2 Plusのみの機能)</li>
<li>約182g、約162×75×8.5mm</li>
<li><strong>ワイヤレス充電は非対応</strong> / イヤホンジャック対応 / Android 14</li>
</ul>
<ul lang="en" style="margin:0 0 .6rem;padding-left:1.3rem;font-size:.88rem;opacity:.85;">
<li>8GB RAM (Docomo / Rakuten Mobile versions) / <strong>12GB RAM (SIM-free version M06; Rakuten's spec table lists only 8GB/256GB)</strong></li>
<li>256GB storage (expandable up to 1TB with microSD)</li>
<li>About 6.6-inch OLED (Full HD+, 2,400x1,080)</li>
<li>Camera: 50.1MP wide (F1.8, optical image stabilization) + 8MP ultra-wide / 16.1MP front</li>
<li>5,000mAh battery / Snapdragon 7s Gen 2</li>
<li>Water resistance (IPX5/IPX8) and dust resistance (IP6X)</li>
<li>Osaifu-Keitai (mobile wallet) supported / fingerprint and face authentication</li>
<li>Pulse-wave sensor measures autonomic nervous system activity (We2 Plus only)</li>
<li>About 182g, about 162 x 75 x 8.5mm</li>
<li><strong>No wireless charging</strong> / headphone jack / Android 14</li>
</ul>
<p style="margin:.3rem 0 0;"><small>未使用品の価格・在庫は販売店ごとに異なります。最新の情報はご自身でご確認ください。</small></p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">Prices and stock of unused units vary by seller. Please check the latest information yourself.</p>
<p style="margin:.5rem 0 0;text-align:center;"><a href="https://www.google.com/search?q=%E5%AF%8C%E5%A3%AB%E9%80%9A+arrows+we2+plus+%E6%9C%AA%E4%BD%BF%E7%94%A8" target="_blank" rel="noopener noreferrer">🔎 「富士通 arrows we2 plus 未使用」を検索 / Search</a><br><a href="https://network.mobile.rakuten.co.jp/product/smartphone/arrows-we2-plus/" target="_blank" rel="noopener noreferrer">▶️ 楽天モバイル arrows We2 Plus(仕様) / Specs</a></p>
</section>
<section class="oppo-reno11-a" style="max-width:420px;margin:1rem auto;padding:.9rem 1rem;border:1px solid rgba(128,128,128,.5);border-radius:10px;text-align:left;line-height:1.6;">
<h3 style="font-size:1.1rem;margin:0 0 .4rem;text-align:center;">④ OPPO Reno11 A(未使用品)</h3>
<p style="text-align:center;margin:0 0 .5rem;"><img src="data:image/webp;base64,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" alt="OPPO Reno11 A 本体(コーラルパープル・ダークグリーン) / Coral Purple and Dark Green" width="280" style="max-width:100%;height:auto;"></p>
<p style="margin:.3rem 0 0;"><strong>メモリ8GB</strong>と、<strong>アウトカメラの解像度が高い(約6,400万画素)</strong>のが特徴です。</p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;"><strong>8GB RAM</strong> and a <strong>high-resolution rear camera (about 64MP)</strong> are the key features.</p>
<ul style="margin:.4rem 0 .1rem;padding-left:1.3rem;">
<li>メモリ8GB / ストレージ128GB(microSDで最大2TBまで対応)</li>
<li>アウトカメラ：約6,400万画素メイン + 約800万画素超広角(112°) + 約200万画素マクロ / インカメラ約3,200万画素</li>
<li>約6.7インチ 有機EL、120Hz</li>
<li>バッテリー5,000mAh、急速充電対応</li>
<li>防水・防塵 IPX5 / IP6X / おサイフケータイ(FeliCa)対応</li>
<li>MediaTek Dimensity 7050 / 約177g、厚さ約7.6mm</li>
<li><strong>ワイヤレス充電は非対応</strong> / Android 14</li>
<li>カラー：ダークグリーン、コーラルパープル</li>
</ul>
<ul lang="en" style="margin:0 0 .6rem;padding-left:1.3rem;font-size:.88rem;opacity:.85;">
<li>8GB RAM / 128GB storage (microSD up to 2TB)</li>
<li>Rear camera: about 64MP main + about 8MP ultra-wide (112 degrees) + about 2MP macro / 32MP front camera</li>
<li>About 6.7-inch OLED, 120Hz</li>
<li>5,000mAh battery, fast charging supported</li>
<li>IPX5 / IP6X water and dust resistance / Osaifu-Keitai (FeliCa) supported</li>
<li>MediaTek Dimensity 7050 / about 177g, about 7.6mm thick</li>
<li><strong>No wireless charging</strong> / Android 14</li>
<li>Colors: Dark Green, Coral Purple</li>
</ul>
<p style="margin:.3rem 0 0;"><small>未使用品の価格・在庫は販売店ごとに異なります。最新の情報はご自身でご確認ください。</small></p>
<p lang="en" style="font-size:.88rem;opacity:.85;margin:.1rem 0 .5rem;">Prices and stock of unused units vary by seller. Please check the latest information yourself.</p>
<p style="margin:.5rem 0 0;text-align:center;"><a href="https://www.google.com/search?q=OPPO+RENO11A+%E6%9C%AA%E4%BD%BF%E7%94%A8" target="_blank" rel="noopener noreferrer">🔎 「OPPO RENO11A 未使用」を検索 / Search</a><br><a href="https://www.oppo.com/jp/smartphones/series-reno/reno11-a/specs/" target="_blank" rel="noopener noreferrer">▶️ OPPO Reno11 A(仕様) / Specs</a></p>
</section>
<p style="text-align:center;margin:0 0 1rem;"><small>(価格・条件は2026-10-26時点の楽天モバイル公式ページ記載。最新は公式で確認。Pixel 11・10aの分)<br><span lang="en">Prices and terms as listed on the Rakuten Mobile official pages on 2026-10-26 (Pixel 11 and 10a). Please check the official pages for the latest.</span></small></p>
</div>
<section class="top-news" style="max-width:640px;margin:1rem auto;text-align:center;">
<p style="font-size:1.2rem;margin:1rem 0;">面白そうなカフェ　検索ワード<br><a href="https://ameblo.jp/www-aon/entry-12981863547.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981863547.html</a></p>
<p style="text-align:center;font-size:1.2rem;margin:1rem 0;"><br>民間ガン治療法<br><br>ボランティア情報<br><br><a href="https://ameblo.jp/www-aon/entry-12981774738.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981774738.html</a></p>
</section>"#;

/// ボランティア情報の下に載せる自己紹介・連絡先(2026-10-26、ユーザー指示)。波括弧は含めないこと。
const TOP_INTRO_HTML: &str = r#"<div class="top-intro" style="margin:1rem auto;padding:.8rem 1rem;max-width:42rem;text-align:left;line-height:1.8;font-size:1.1rem;">
    <p style="margin:0 0 .8rem;">CLAUDEと言うAIをマスターして使いこなしておりますので、<br>様々なプログラム言語とフレームワーク対応可能で、<br>様々なWEBサイトやスマホアプリ開発が可能で御座います。</p>
    <p style="margin:0;"><a href="mailto:norukia.jp@gmail.com">norukia.jp@gmail.com</a><br>
    石　塚　正　浩<br>
    郵便番号　197-0828<br>
    東京都あきる野市秋留5-3-11<br>
    <a href="tel:09075555011">090-7555-5011</a></p>
  </div>"#;

/// 「政府は、LINEアプリ化して…」の紹介(下の方から上へ移動、2026-10-26、ユーザー指示)。
const GOV_LINE_HTML: &str = r#"<p style="font-size:1.2rem;margin:1rem 0;">政府は、LINEアプリ化して、99％リストラして少子高齢化と年金と後期高齢者無償化予算確保。<br>
    <a href="https://ameblo.jp/www-aon/entry-12981307682.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981307682.html</a></p>"#;

/// 上の定数の続き(動画セクション)。ボランティア情報の下に自己紹介などを差し込むため分割した。
const TOP_VIDEO_HTML_B: &str = r#"<section class="top-news" style="max-width:640px;margin:1rem auto;text-align:center;">
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/shorts/j_PGMrbhkj0" target="_blank" rel="noopener noreferrer">"4時間待ち"それでも外国人が殺到する日本のアニメイベントとは？</a></p>
<div style="position:relative;width:100%;max-width:640px;aspect-ratio:9/16;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/j_PGMrbhkj0?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='"4時間待ち"それでも外国人が殺到する日本のアニメイベントとは？';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/j_PGMrbhkj0/hqdefault.jpg" alt=""4時間待ち"それでも外国人が殺到する日本のアニメイベントとは？" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/watch?v=FODDxEZDVsY" target="_blank" rel="noopener noreferrer">フランスのコミケで大号泣した</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/FODDxEZDVsY?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='フランスのコミケで大号泣した';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/FODDxEZDVsY/hqdefault.jpg" alt="フランスのコミケで大号泣した" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;margin:0 0 1.2rem;">ラピダスより重要OIST　EUV露光装置をASMLと比較　消費電力1/10　装置本体コスト1/4<br><a href="https://ameblo.jp/www-aon/entry-12982430784.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12982430784.html</a></p>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=WWL5t0t_tEs" target="_blank" rel="noopener noreferrer">【ファナック】なぜ世界の工場は日本製の「頭脳」を選ぶのか？｜CNCが利益を生む本当の理由</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/WWL5t0t_tEs?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='【ファナック】なぜ世界の工場は日本製の「頭脳」を選ぶのか？｜CNCが利益を生む本当の理由';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/WWL5t0t_tEs/hqdefault.jpg" alt="【ファナック】なぜ世界の工場は日本製の「頭脳」を選ぶのか？｜CNCが利益を生む本当の理由" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=L3ugAc-fgVY" target="_blank" rel="noopener noreferrer">【海外の反応】日本の製造業をなめるな！ファナック・安川電機がロボットアームで世界を動かす理由…エプソン・川崎重工・ヤマハ・安川電機【日本の技術】</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/L3ugAc-fgVY?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='【海外の反応】日本の製造業をなめるな！ファナック・安川電機がロボットアームで世界を動かす理由…エプソン・川崎重工・ヤマハ・安川電機【日本の技術】';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/L3ugAc-fgVY/hqdefault.jpg" alt="【海外の反応】日本の製造業をなめるな！ファナック・安川電機がロボットアームで世界を動かす理由…エプソン・川崎重工・ヤマハ・安川電機【日本の技術】" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://www.youtube.com/watch?v=5MFwZbfpKEk" target="_blank" rel="noopener noreferrer">この世界最大のタブー〝YHWH〟の正体がついに暴露されました。理想社会「弥勒の世」を実現する鍵は日本人が大切にしてきた〇〇だった！？この動画から時代がさらに動きます。</a></p>
<p style="font-size:1.2rem;margin:0 0 .6rem;">出口王仁三郎のひ孫の出口光（ひかる）さんがYHWHと一体になれる体験をビデオにまとめてくれております。</p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/5MFwZbfpKEk?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='YHWH';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/5MFwZbfpKEk/hqdefault.jpg" alt="YHWH" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:.9rem;margin:-.6rem 0 1.2rem;">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/1702321541402846?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1702321541402846?locale=ja_JP</a></p>
<p style="font-size:1.2rem;margin:1.2rem 0;">学校の教科書から マインクラフトを超えたオンライン3D GAMEを作る方法！<br><a href="https://ameblo.jp/www-aon/entry-12981886969.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981886969.html</a></p>
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/watch?v=TooMBNl0lAw" target="_blank" rel="noopener noreferrer">日本が「石油」を超えた！世界を変える新発明の正体</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto 1.2rem;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/TooMBNl0lAw?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='日本が「石油」を超えた！世界を変える新発明の正体';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/TooMBNl0lAw/hqdefault.jpg" alt="日本が「石油」を超えた！世界を変える新発明の正体" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;margin:1.2rem 0;">V6＆V8＆V10＆V12マルチステージターボエンジン開発を国家プロジェクトにするアイデア！<br><a href="https://ameblo.jp/www-aon/entry-12981432312.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981432312.html</a></p>
<p style="font-size:1.2rem;font-weight:700;margin:0 0 .6rem;"><a href="https://ameblo.jp/www-aon/entry-12981724246.html" target="_blank" rel="noopener noreferrer">今の上海は経済崩壊でシャッター街なの？</a></p>
<div style="position:relative;width:100%;max-width:640px;aspect-ratio:9/16;margin:0 auto;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/oPGV1n8niXw?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='今の上海は経済崩壊でシャッター街なの？';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/oPGV1n8niXw/hqdefault.jpg" alt="今の上海は経済崩壊でシャッター街なの？" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;margin:1.2rem 0;">TOYOTAの方針では、<br>中国と日本の工場の生産コストを同じにする為に、ロボットや3Dプリンターなどのハイテクと熟練職人を育てて高級車を担当して頂く事などのバランスを重要視していくと言われております。</p>
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/shorts/2lileYb0OnQ" target="_blank" rel="noopener noreferrer">ロボットの巨人！安川電機のIoT！精緻なデータ収集が可能！【IIFES】</a></p>
<div style="position:relative;width:100%;max-width:640px;aspect-ratio:9/16;margin:0 auto;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/2lileYb0OnQ?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='ロボットの巨人！安川電機のIoT！精緻なデータ収集が可能！【IIFES】';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/2lileYb0OnQ/hqdefault.jpg" alt="ロボットの巨人！安川電機のIoT！精緻なデータ収集が可能！【IIFES】" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/shorts/jW5qS1ZQbsg" target="_blank" rel="noopener noreferrer">中韓には一生不可能な技術を日本がサクっと開発</a></p>
<div style="position:relative;width:100%;max-width:640px;aspect-ratio:9/16;margin:0 auto;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/jW5qS1ZQbsg?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='中韓には一生不可能な技術を日本がサクっと開発';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/jW5qS1ZQbsg/hqdefault.jpg" alt="中韓には一生不可能な技術を日本がサクっと開発" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/watch?v=l5qJAHhvvBs" target="_blank" rel="noopener noreferrer">ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！</a></p>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:0 auto;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/l5qJAHhvvBs?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/l5qJAHhvvBs/hqdefault.jpg" alt="ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size:1.2rem;margin:1.2rem 0 .6rem;">オリエンタルモーターの低コスト産業用ロボット<br><a href="https://www.youtube.com/results?search_query=%E3%82%AA%E3%83%AA%E3%82%A8%E3%83%B3%E3%82%BF%E3%83%AB%E3%83%A2%E3%83%BC%E3%82%BF%E3%83%BC%E3%81%AE%E4%BD%8E%E3%82%B3%E3%82%B9%E3%83%88%E7%94%A3%E6%A5%AD%E7%94%A8%E3%83%AD%E3%83%9C%E3%83%83%E3%83%88" target="_blank" rel="noopener noreferrer">Youtube検索結果はこちら</a></p>
<p style="font-size:1.2rem;margin:1.2rem 0 .6rem;">Google検索ワード<br><a href="https://www.google.com/search?q=%E4%BA%BA%E5%B7%A5%E5%85%89%E5%90%88%E6%88%90%E3%82%92%E5%88%A9%E7%94%A8%E3%81%97%E3%81%A6%E4%BA%8C%E9%85%B8%E5%8C%96%E7%82%AD%E7%B4%A0%EF%BC%88CO2%EF%BC%89%E3%81%A8%E6%B0%B4%E3%81%8B%E3%82%89%E5%B8%B8%E6%B8%A9%E3%83%BB%E5%B8%B8%E5%9C%A7%E3%81%A7%E6%89%B1%E3%81%88%E3%81%A6%E3%80%81%E8%BC%B8%E9%80%81%E3%81%AB%E9%AB%98%E5%9C%A7%E3%83%9C%E3%83%B3%E3%83%99%E3%82%84%E6%A5%B5%E4%BD%8E%E6%B8%A9%E5%86%B7%E5%8D%B4%E3%81%8C%E5%BF%85%E8%A6%81%E7%84%A1%E3%81%84%E3%80%8C%E3%82%AE%E9%85%B8%EF%BC%88HCOOH%EF%BC%89%E3%80%8D%E3%82%92%E7%94%9F%E6%88%90%E3%81%99%E3%82%8B%E6%8A%80%E8%A1%93" target="_blank" rel="noopener noreferrer">人工光合成を利用して二酸化炭素（CO2）と水から常温・常圧で扱えて、輸送に高圧ボンベや極低温冷却が必要無い「ギ酸（HCOOH）」を生成する技術</a><br>上記リンクをクリック後、AIモードをクリックしてご覧下さいませ。</p>
<p style="font-size:1.2rem;margin:0;"><br>ギ酸は、水素キャリアになります。<br>常温核融合は、数年に一度、水素燃料を補充するだけで、発電し続ける様です。</p>
<p style="font-size:1.2rem;margin:0;"><br>Youtube検索ワード<br><a href="https://www.youtube.com/results?search_query=%E5%B8%B8%E6%B8%A9%E6%A0%B8%E8%9E%8D%E5%90%88%E3%82%B9%E3%83%88%E3%83%BC%E3%83%96++%E5%B8%B8%E6%B8%A9%E6%A0%B8%E8%9E%8D%E5%90%88%E6%9A%96%E6%88%BF%E6%A9%9F++%E5%B0%8F%E5%9E%8B%E6%A0%B8%E8%9E%8D%E5%90%88%E6%9A%96%E6%88%BF%E6%A9%9F+%E3%83%9F%E3%83%A5%E3%82%A6%E3%82%AA%E3%83%B3++%E3%80%90%E3%83%9F%E3%83%A5%E3%82%AA%E3%83%B3%E3%81%AB%E3%82%88%E3%82%8B%E5%B8%B8%E6%B8%A9%E6%A0%B8%E8%9E%8D%E5%90%88%E3%80%91" target="_blank" rel="noopener noreferrer">常温核融合ストーブ  常温核融合暖房機  小型核融合暖房機 ミュウオン  【ミュオンによる常温核融合】</a></p>
<p style="font-size:1.2rem;font-weight:700;margin:1.2rem 0 .6rem;"><a href="https://www.youtube.com/shorts/tcknEiNp-zg" target="_blank" rel="noopener noreferrer">これは最も異例のヘリコプターだ</a></p>
<div style="position:relative;width:100%;max-width:640px;aspect-ratio:9/16;margin:0 auto;background:#000;border-radius:8px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/tcknEiNp-zg?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='これは最も異例のヘリコプターだ';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/tcknEiNp-zg/hqdefault.jpg" alt="これは最も異例のヘリコプターだ" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
</section>
"#;

#[allow(dead_code)] // トップから「重複」の紹介を外したため未使用(2026-10-26)
const BLOG_POST_URL: &str = "https://ameblo.jp/www-aon/entry-12973252437.html";
#[allow(dead_code)] // トップから「重複」の紹介を外したため未使用(2026-10-26)
const BLOG_POST_TITLE_JA: &str = "プログラム言語やフレームワークなどの全てをRust(Poemやhyper)版に移植するメリット?";
/// ユーザー指示(2026-07-20)により日英両方で掲載。ブログ本文自体は
/// 日本語のみだが、リンクのラベルは英語話者にも内容が伝わるよう
/// 意訳したもの(URLは日英共通、リンク先は変えない)。
#[allow(dead_code)] // トップから「重複」の紹介を外したため未使用(2026-10-26)
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
/// 各種保険・酪農家黒字化の人工授精師の話・植林〈杉より高級木材〉・
/// 商工会議所&民生委員のSET創業支援&AI駆動プログラマー育成)の
/// 日本語原文+英訳(2026-09-09追加、ユーザー指示によりrunno.tokyoと同じ
/// 内容をaruaru.tokyoにも掲載)。
const PROPOSAL_TEXT_JA_1: &str = "新規提案として、Amazon倉庫、コンビニ、スーパーなどのコーヒーの自動販売機コーナーや、イートインコーナーなどで、新規企画で始められるなら不動産の売買、賃貸などの電子契約、建設会社＆AUDIOルーム、IMAXシアターや4DXシアターの家庭版や住宅の壁や天井にスピーカーを埋め込んで映画館の様にリフォームしたり、それを一戸建て注文住宅として建設する工務店サービス、それを本物の映画館やコンサートホールとして建設する建設会社のサービス。AFLACやNISSAYやSONY損保などの人間やペットや火災保険、地震保険、自動車保険、ちなみに海外の自動車保険のセールスマンは自動車の新車も中古車の売買も行ないますし、お買い得な売出し中の一戸建て注文住宅も、建売物件もご紹介可能らしいです。";

const PROPOSAL_TEXT_JA_2: &str = "ちなみ、先祖にレモン牛乳を作った関東牛乳の創業者がいますが、酪農家が黒字になる話として、まずは、人工授精師と言う資格の講座を受講して頂き、一生懸命ノートに講座の内容を書いてメモして、そしてTESTを受けてみごと合格して頂き、和牛の精子と卵子の受精卵を乳牛のメスに移植して妊娠すれば、乳牛が和牛と乳牛の両方の子牛を出産して牛乳とそれらの加工品と合わせて出荷する事で、酪農家が黒字なりやすいです。と言うお話もしておきます。ちなみに北海道くらい広い土地なら、なるべくフラットに平らに整地して、冬の豪雪が解けるタイミングで放牧の牛糞と雪が解けていっしょになると処理が大変な時に、あらかじめ、泥もじゃりも一緒に流し込めて、リサイクル土や肥料として出荷して販売してもらう前提で、排水溝や下水溝を建設する企画もご提案しておきます。";

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
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=fDAToBerhPE" target="_blank" rel="noopener noreferrer">福山雅治 - 道標 2022〈Another Story of Promise for the Future『裸の音』〉(Short ver.)</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/fDAToBerhPE?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='福山雅治 - 道標 2022〈Another Story of Promise for the Future『裸の音』〉(Short ver.)';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/fDAToBerhPE/hqdefault.jpg" alt="福山雅治 - 道標 2022〈Another Story of Promise for the Future『裸の音』〉(Short ver.)" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size: 0.85rem; color: var(--muted, #94a3b8);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/314453634730386?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a><br><a href="/video/fukuyama/fukuyama-michishirube-2022.mp4" target="_blank" rel="noopener noreferrer" onclick="var v=document.createElement('video');v.src=this.href;v.controls=true;v.autoplay=true;v.poster='https://i.ytimg.com/vi/fDAToBerhPE/hqdefault.jpg';v.playsInline=true;v.preload='auto';v.style.cssText='display:block;width:100%;max-width:640px;margin:.6rem auto;background:#000;border-radius:6px';this.parentNode.insertBefore(v,this.nextSibling);this.style.display='none';return false">MP4(クリックで再生 / click to play)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=GJMAjdd2amY" target="_blank" rel="noopener noreferrer">福山雅治 - あの夏も 海も 空も（FUKUYAMA MASAHARU DOME LIVE 2018 -暗闇の中で飛べ-）</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/GJMAjdd2amY?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='福山雅治 - あの夏も 海も 空も（FUKUYAMA MASAHARU DOME LIVE 2018 -暗闇の中で飛べ-）';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/GJMAjdd2amY/hqdefault.jpg" alt="福山雅治 - あの夏も 海も 空も（FUKUYAMA MASAHARU DOME LIVE 2018 -暗闇の中で飛べ-）" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size: 0.85rem; color: var(--muted, #94a3b8);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/1049901451439463?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a><br><a href="/video/fukuyama/fukuyama-ano-natsu-mo-umi-mo-sora-mo.mp4" target="_blank" rel="noopener noreferrer" onclick="var v=document.createElement('video');v.src=this.href;v.controls=true;v.autoplay=true;v.poster='https://i.ytimg.com/vi/GJMAjdd2amY/hqdefault.jpg';v.playsInline=true;v.preload='auto';v.style.cssText='display:block;width:100%;max-width:640px;margin:.6rem auto;background:#000;border-radius:6px';this.parentNode.insertBefore(v,this.nextSibling);this.style.display='none';return false">MP4(クリックで再生 / click to play)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=yjlsyMGJlvM" target="_blank" rel="noopener noreferrer">福山雅治 - ヒトツボシ（WE'RE BROS. TOUR 2022 “光” -HIKARI-）</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/yjlsyMGJlvM?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='福山雅治 - ヒトツボシ（WE'RE BROS. TOUR 2022 “光” -HIKARI-）';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/yjlsyMGJlvM/hqdefault.jpg" alt="福山雅治 - ヒトツボシ（WE'RE BROS. TOUR 2022 “光” -HIKARI-）" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size: 0.85rem; color: var(--muted, #94a3b8);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/2346373692826504?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a><br><a href="/video/fukuyama/fukuyama-hitotsuboshi.mp4" target="_blank" rel="noopener noreferrer" onclick="var v=document.createElement('video');v.src=this.href;v.controls=true;v.autoplay=true;v.poster='https://i.ytimg.com/vi/yjlsyMGJlvM/hqdefault.jpg';v.playsInline=true;v.preload='auto';v.style.cssText='display:block;width:100%;max-width:640px;margin:.6rem auto;background:#000;border-radius:6px';this.parentNode.insertBefore(v,this.nextSibling);this.style.display='none';return false">MP4(クリックで再生 / click to play)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=dA3kYLJ57NQ" target="_blank" rel="noopener noreferrer">福山雅治 - 蛍（福山☆夏の大創業祭 2015 稲佐山）</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/dA3kYLJ57NQ?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='福山雅治 - 蛍（福山☆夏の大創業祭 2015 稲佐山）';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/dA3kYLJ57NQ/hqdefault.jpg" alt="福山雅治 - 蛍（福山☆夏の大創業祭 2015 稲佐山）" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size: 0.85rem; color: var(--muted, #94a3b8);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/2168438207439523?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a><br><a href="/video/fukuyama/fukuyama-hotaru.mp4" target="_blank" rel="noopener noreferrer" onclick="var v=document.createElement('video');v.src=this.href;v.controls=true;v.autoplay=true;v.poster='https://i.ytimg.com/vi/dA3kYLJ57NQ/hqdefault.jpg';v.playsInline=true;v.preload='auto';v.style.cssText='display:block;width:100%;max-width:640px;margin:.6rem auto;background:#000;border-radius:6px';this.parentNode.insertBefore(v,this.nextSibling);this.style.display='none';return false">MP4(クリックで再生 / click to play)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=my4Dxmyx1n8" target="_blank" rel="noopener noreferrer">(再投稿) 福山雅治 桜坂 2005</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/my4Dxmyx1n8?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='(再投稿) 福山雅治 桜坂 2005';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/my4Dxmyx1n8/hqdefault.jpg" alt="(再投稿) 福山雅治 桜坂 2005" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size: 0.85rem; color: var(--muted, #94a3b8);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/1629015382578865?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a><br><a href="/video/fukuyama/fukuyama-sakurazaka-2005.mp4" target="_blank" rel="noopener noreferrer" onclick="var v=document.createElement('video');v.src=this.href;v.controls=true;v.autoplay=true;v.poster='https://i.ytimg.com/vi/my4Dxmyx1n8/hqdefault.jpg';v.playsInline=true;v.preload='auto';v.style.cssText='display:block;width:100%;max-width:640px;margin:.6rem auto;background:#000;border-radius:6px';this.parentNode.insertBefore(v,this.nextSibling);this.style.display='none';return false">MP4(クリックで再生 / click to play)</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=LG9zEl7ASVk" target="_blank" rel="noopener noreferrer">『桜坂』 EARTH THE KO Opening Ceremony at 嚴島神社 | 柴咲コウ【期間限定】</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/LG9zEl7ASVk?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='『桜坂』 EARTH THE KO Opening Ceremony at 嚴島神社 | 柴咲コウ【期間限定】';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/LG9zEl7ASVk/hqdefault.jpg" alt="『桜坂』 EARTH THE KO Opening Ceremony at 嚴島神社 | 柴咲コウ【期間限定】" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
<p style="font-size: 0.85rem; color: var(--muted, #94a3b8);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 / If the YouTube link breaks, you can also watch it via the following URL:<br><a href="https://www.facebook.com/reel/2044662199570657?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a><br><a href="/video/fukuyama/shibasaki-sakurazaka-itsukushima.mp4" target="_blank" rel="noopener noreferrer" onclick="var v=document.createElement('video');v.src=this.href;v.controls=true;v.autoplay=true;v.poster='https://i.ytimg.com/vi/LG9zEl7ASVk/hqdefault.jpg';v.playsInline=true;v.preload='auto';v.style.cssText='display:block;width:100%;max-width:640px;margin:.6rem auto;background:#000;border-radius:6px';this.parentNode.insertBefore(v,this.nextSibling);this.style.display='none';return false">MP4(クリックで再生 / click to play)</a></p>
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
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">東亜紀 童神<br><span style="color:var(--muted);">AKI  AZUMA  - Warabigami</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/FNUKj6qMQZw" title="東亜紀 童神" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=ebTaPQ-KvRw" target="_blank" rel="noopener noreferrer">아즈마아키(東亜樹) - Volare｜한일톱텐쇼 60회</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/ebTaPQ-KvRw?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='아즈마아키(東亜樹) - Volare｜한일톱텐쇼 60회';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/ebTaPQ-KvRw/hqdefault.jpg" alt="아즈마아키(東亜樹) - Volare｜한일톱텐쇼 60회" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=9uRNlZENWbA" target="_blank" rel="noopener noreferrer">[공연실황] 아즈마아키(東亜樹)-Que Sera Sera(ケセラセラ) I 스테이지엠 영화음악 콘서트</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/9uRNlZENWbA?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='[공연실황] 아즈마아키(東亜樹)-Que Sera Sera(ケセラセラ) I 스테이지엠 영화음악 콘서트';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/9uRNlZENWbA/hqdefault.jpg" alt="[공연실황] 아즈마아키(東亜樹)-Que Sera Sera(ケセラセラ) I 스테이지엠 영화음악 콘서트" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
</div>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=DKpwG6E5rok" target="_blank" rel="noopener noreferrer">歌心りえ　X　東亜樹　童神</a></h2>
<div style="position:relative;width:100%;aspect-ratio:16/9;margin:1rem 0;background:#000;border-radius:6px;overflow:hidden;">
<button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/DKpwG6E5rok?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='歌心りえ　X　東亜樹　童神';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/DKpwG6E5rok/hqdefault.jpg" alt="歌心りえ　X　東亜樹　童神" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
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
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">歌心りえ ＆ エノク　Casablanca<br><span style="color:var(--muted);">Utagokoro Rie &amp; Enoch - Casablanca</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/v52UNMJH7tg" title="歌心りえ ＆ エノク　Casablanca" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem;"><a href="https://www.youtube.com/watch?v=v52UNMJH7tg" target="_blank" rel="noopener noreferrer">▶️ YouTube</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">歌心りえ ＆ ソンテジン　Rainy Blue<br><span style="color:var(--muted);">Utagokoro Rie &amp; Sung Tae-jin - Rainy Blue</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/p70qdkpf6jw" title="歌心りえ ＆ ソンテジン　Rainy Blue" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem;"><a href="https://www.youtube.com/watch?v=p70qdkpf6jw" target="_blank" rel="noopener noreferrer">▶️ YouTube</a></p>
</div>

<div class="space-video" style="margin: 1.5rem 0; text-align: center;">
<h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;">歌心りえ ＆ シン·スンテ　川の流れのように（美空ひばりさんのカバー）<br><span style="color:var(--muted);">Utagokoro Rie &amp; Shin Seung-tae - Kawa no Nagare no Yo ni (cover of Misora Hibari)</span></h2>
<div style="position: relative; width: 100%; padding-top: 56.25%; margin: 1rem 0;">
<iframe width="100%" height="100%" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0; border-radius: 6px;" src="https://www.youtube.com/embed/vVzQVcnib1w" title="歌心りえ ＆ シン·スンテ　川の流れのように（美空ひばりさんのカバー）" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
</div>
<p style="font-size: 0.85rem;"><a href="https://www.youtube.com/watch?v=vVzQVcnib1w" target="_blank" rel="noopener noreferrer">▶️ YouTube</a></p>
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

fn render_related_sites(range: std::ops::Range<usize>) -> String {
    RELATED_SITES
        .iter()
        .enumerate()
        .filter(|(i, _)| range.contains(i))
        .map(|(_, s)| {
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
    let related_sites_top = render_related_sites(0..2);
    let related_sites = render_related_sites(2..usize::MAX);
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
{TOP_VIDEO_HTML}
{TOP_INTRO_HTML}
    {AI_DRIVEN_RESEARCH_HTML}
    {GOV_LINE_HTML}
    {ONLINE_LEARNING_HTML}
    <p style="font-size:1.2rem;margin:1rem 0;text-align:center;">
    <a href="https://www.youtube.com/watch?v=mrCAz7mU9Zo" target="_blank" rel="noopener noreferrer">▶️ 【1日密着】Claude Codeに取り憑かれたエンジニア｜その衝撃の開発手法に迫る</a>
    <br />
    <a href="https://www.facebook.com/reel/1753405299237847?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook Backup</a>
    <br />
    {claude_code_desktop_search}
    <br />
    {related_sites_top}
    </p>
{TOP_VIDEO_HTML_B}
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
    <p style="font-size:1.2rem;margin-top:2.5rem;">レジオネラ菌対策は、高濃度塩素の代わりに、過酸化水素（3%）やフリップ・フラップ(A+B剤)で<br>
    <a href="https://ameblo.jp/www-aon/entry-12981661256.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981661256.html</a></p>
    <p style="font-size:1.2rem;margin-top:2.5rem;">腰痛対策　ダイエット　腰振り<br>
    <a href="https://ameblo.jp/www-aon/entry-12981627662.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981627662.html</a></p>
    <p style="font-size:1.2rem;margin-top:2.5rem;">イエスキリスト(Jesus Christ) 気功療法 (Qigong therapy)</p>
    <p style="font-size:1.2rem;">気功。KIKOU.<br>
    <a href="https://ameblo.jp/www-aon/entry-12981249631.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981249631.html</a></p>
    <div class="space-video" style="margin: 2.5rem auto 0; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.2rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=auRYv7w1NOg" target="_blank" rel="noopener noreferrer">ロイヤル・レイモンド・ライフ博士の研究 Frequência Rife Mestre de Cura Geral (4 em 1) 10000 Hz, 880 Hz, 528 Hz &amp; 8 hz</a></h2>
    <div class="yt-click" data-id="auRYv7w1NOg" data-title="ロイヤル・レイモンド・ライフ博士の研究 Frequência Rife Mestre de Cura Geral (4 em 1) 10000 Hz, 880 Hz, 528 Hz &amp; 8 hz" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=auRYv7w1NOg" target="_blank" rel="noopener noreferrer" title="Frequência Rife Mestre de Cura Geral (4 em 1)" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/auRYv7w1NOg/hqdefault.jpg" alt="Frequência Rife Mestre de Cura Geral (4 em 1)" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    </div>
    <p style="font-size:1.2rem;margin-top:2.5rem;">鉄にクロム、ニッケル、高濃度リンで数千年錆びにくい合金のパテント　石　塚　正　浩<br>
    <a href="https://ameblo.jp/www-aon/entry-12981538464.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981538464.html</a></p>
    <p style="font-size:1.2rem;margin-top:2.5rem;">4 in 1 (A+B+C+D)＋E　High-Tech Fiber Hybrid Battery 　ハイテク・ファイバーハイブリッド電池　2026/10/11<br>
    <a href="https://ameblo.jp/www-aon/entry-12981294981.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/entry-12981294981.html</a></p>
    <p style="font-size:1.2rem;margin-top:2.5rem;">U.S PatentConcept: CNF Fiber Battery by Masahiro Ishizuka<br>
    <a href="https://aon.co.jp" target="_blank" rel="noopener noreferrer">https://aon.co.jp</a><br>
    <a href="https://ameblo.jp/www-aon/page-5.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/page-5.html</a></p>
    <p style="font-size:1.2rem;margin-top:2.5rem;">特許アイデア:CNFファイバー電池(木質電池+ファイバーハイブリッド電池)発案者:石 塚 正 浩<br>
    <a href="https://ameblo.jp/www-aon/page-6.html" target="_blank" rel="noopener noreferrer">https://ameblo.jp/www-aon/page-6.html</a></p>
    <p style="font-size:1.2rem;margin-top:2.5rem;">気功。KIKOU. 自ら"気"を感じながら相手に流す練習法(動画)。<br>
    <a href="https://youtu.be/WBUojmKng3M" target="_blank" rel="noopener noreferrer">https://youtu.be/WBUojmKng3M</a><br>
    <video controls preload="metadata" style="width:100%;max-width:640px;aspect-ratio:16/9;height:auto;background:#000;border-radius:6px;object-fit:contain;">
    <source src="/video/kikou-practice.mp4" type="video/mp4">
    お使いのブラウザは動画再生に対応していません。<a href="/video/kikou-practice.mp4">動画ファイルを直接開く</a>
    </video></p>
    <p style="font-size:1.2rem;">気功。KIKOU. Facebook Backup.<br>
    <a href="https://www.facebook.com/100000656454938/videos/566679491579232" target="_blank" rel="noopener noreferrer">https://www.facebook.com/100000656454938/videos/566679491579232</a></p>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=7BKeQN4HeDM" target="_blank" rel="noopener noreferrer">【簡単】気を強力に出して気功治療もできる【気功講座12】</a></h2>
    <div class="yt-click" data-id="7BKeQN4HeDM" data-title="【簡単】気を強力に出して気功治療もできる【気功講座12】" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=7BKeQN4HeDM" target="_blank" rel="noopener noreferrer" title="【簡単】気を強力に出して気功治療もできる【気功講座12】" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/7BKeQN4HeDM/hqdefault.jpg" alt="【簡単】気を強力に出して気功治療もできる【気功講座12】" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
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
    <p><a href="https://www.facebook.com/reel/4626852324298766?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/4626852324298766?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=zNqA9q9l5M0" target="_blank" rel="noopener noreferrer">【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】</a></h2>
    <div class="yt-click" data-id="zNqA9q9l5M0" data-title="【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=zNqA9q9l5M0" target="_blank" rel="noopener noreferrer" title="【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/zNqA9q9l5M0/hqdefault.jpg" alt="【経絡に気を流す方法】十二経脈を活性化しよう【気功講座31】" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/1029774840070283?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1029774840070283?locale=ja_JP</a></p>
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
    <p><a href="https://www.facebook.com/reel/1626743582451474?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1626743582451474?locale=ja_JP</a></p>
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
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/shorts/APtgWQrH1mE" target="_blank" rel="noopener noreferrer">ブリキ缶とバカにされた日本のトラクターが北米でとんでもないことに…</a></h2>
    <div class="yt-click" data-id="APtgWQrH1mE" data-title="ブリキ缶とバカにされた日本のトラクターが北米でとんでもないことに…" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/shorts/APtgWQrH1mE" target="_blank" rel="noopener noreferrer" title="ブリキ缶とバカにされた日本のトラクターが北米でとんでもないことに…" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/APtgWQrH1mE/hqdefault.jpg" alt="ブリキ缶とバカにされた日本のトラクターが北米でとんでもないことに…" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=l5qJAHhvvBs" target="_blank" rel="noopener noreferrer">ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！</a></h2>
    <div class="yt-click" data-id="l5qJAHhvvBs" data-title="ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=l5qJAHhvvBs" target="_blank" rel="noopener noreferrer" title="ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/l5qJAHhvvBs/hqdefault.jpg" alt="ロボットで一括りにするな！ロボット業界（ファナック・安川電機）を完全解説。業界研究・企業研究・志望動機、これ1本！" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/results?search_query=%E6%BA%B6%E6%8E%A5%E3%80%80%E9%87%91%E5%B1%9E%E3%80%803D%E3%83%97%E3%83%AA%E3%83%B3%E3%82%BF%E3%83%BC" target="_blank" rel="noopener noreferrer">溶接　金属　3Dプリンター</a></h2>
    <div class="yt-click" data-id="CXWH8UpQSdo" data-title="人の考えが聞こえる能力" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/shorts/CXWH8UpQSdo" target="_blank" rel="noopener noreferrer" title="人の考えが聞こえる能力" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/CXWH8UpQSdo/hqdefault.jpg" alt="人の考えが聞こえる能力" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/2017519715630842?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/2017519715630842?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=eOlpH_duoGw" target="_blank" rel="noopener noreferrer">【知らなきゃ損】身体の芯から力が湧き出る「腰割り」のやり方</a></h2>
    <div class="yt-click" data-id="eOlpH_duoGw" data-title="【知らなきゃ損】身体の芯から力が湧き出る「腰割り」のやり方" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=eOlpH_duoGw" target="_blank" rel="noopener noreferrer" title="【知らなきゃ損】身体の芯から力が湧き出る「腰割り」のやり方" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/eOlpH_duoGw/hqdefault.jpg" alt="【知らなきゃ損】身体の芯から力が湧き出る「腰割り」のやり方" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/1450275836916465?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1450275836916465?locale=ja_JP</a></p>
    </div>
    <div class="space-video" style="margin: 1.5rem auto; text-align: center; max-width: 640px;">
    <h2 style="font-size: 1.4rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=OAZd8yP5JWE" target="_blank" rel="noopener noreferrer">離婚しに来たのに妻の顔を見た瞬間</a></h2>
    <div class="yt-click" data-id="OAZd8yP5JWE" data-title="離婚しに来たのに妻の顔を見た瞬間" style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;cursor:pointer;">
    <a href="https://www.youtube.com/watch?v=OAZd8yP5JWE" target="_blank" rel="noopener noreferrer" title="離婚しに来たのに妻の顔を見た瞬間" style="display:block;width:100%;height:100%;">
    <img src="https://i.ytimg.com/vi/OAZd8yP5JWE/hqdefault.jpg" alt="離婚しに来たのに妻の顔を見た瞬間" loading="lazy" style="width:100%;height:100%;object-fit:cover;">
    <span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);font-size:4rem;color:#fff;text-shadow:0 0 12px #000;">▶</span>
    </a>
    </div>
    <p><a href="https://www.facebook.com/reel/1086159577540831?locale=ja_JP" target="_blank" rel="noopener noreferrer">https://www.facebook.com/reel/1086159577540831?locale=ja_JP</a></p>
    </div>
    <script>
    document.querySelectorAll('.yt-click').forEach(function(b){{
      var h=b.parentNode.querySelector('h2');
      if(!h||h.textContent.trim()!==b.dataset.title){{var t=document.createElement('p');t.textContent=b.dataset.title;b.parentNode.insertBefore(t,b.nextSibling);}}
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
  <h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 0;"><a href="https://www.youtube.com/watch?v=qxFGnpGRQiQ" target="_blank" rel="noopener noreferrer">櫻坂46山﨑天＆オダギリジョー、親子役で初共演　音楽はPUNPEEとハナレグミがコラボ　相鉄東急直通記念ムービー「父と娘の風景」</a></h2>
  <div style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;">
  <button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/qxFGnpGRQiQ?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='櫻坂46山﨑天＆オダギリジョー、親子役で初共演　音楽はPUNPEEとハナレグミがコラボ　相鉄東急直通記念ムービー「父と娘の風景」';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/qxFGnpGRQiQ/hqdefault.jpg" alt="櫻坂46山﨑天＆オダギリジョー、親子役で初共演　音楽はPUNPEEとハナレグミがコラボ　相鉄東急直通記念ムービー「父と娘の風景」" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
  </div>
  <p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
  If the YouTube link breaks, you can also watch it via the following URL:
  <a href="https://www.facebook.com/reel/1804979787176806?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
  <h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 1.8rem;"><a href="https://www.youtube.com/watch?v=_W8LUM7E860" target="_blank" rel="noopener noreferrer">櫻坂46・山﨑天ｘオダギリ ジョー、148秒ワンカットで描く父と娘の12年間　音楽はPUNPEEとハナレグミのコラボレーション</a></h2>
  <div style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;">
  <button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/_W8LUM7E860?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='櫻坂46・山﨑天ｘオダギリ ジョー、148秒ワンカットで描く父と娘の12年間　音楽はPUNPEEとハナレグミのコラボレーション';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/_W8LUM7E860/hqdefault.jpg" alt="櫻坂46・山﨑天ｘオダギリ ジョー、148秒ワンカットで描く父と娘の12年間　音楽はPUNPEEとハナレグミのコラボレーション" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
  </div>
  <p style="font-size: 0.85rem; color: var(--muted);">YouTubeのリンクが切れたら次のURLでもご視聴になれます。 /
  If the YouTube link breaks, you can also watch it via the following URL:
  <a href="https://www.facebook.com/reel/2211985123031669?locale=ja_JP" target="_blank" rel="noopener noreferrer">Facebook(予備 / backup)</a></p>
  <h2 style="font-size: 1.15rem; border-bottom: none; margin-top: 1.8rem;"><a href="https://www.youtube.com/watch?v=J_hH4zZ5Uvs" target="_blank" rel="noopener noreferrer">数多くの日本人を泣かせた伝説のCM</a></h2>
  <div style="position:relative;width:100%;max-width:640px;aspect-ratio:16/9;margin:1rem auto;background:#000;border-radius:6px;overflow:hidden;">
  <button type="button" aria-label="動画を再生 / Play video" style="all:unset;position:absolute;inset:0;cursor:pointer;" onclick="var f=document.createElement('iframe');f.src='https://www.youtube-nocookie.com/embed/J_hH4zZ5Uvs?autoplay=1&amp;playsinline=1&amp;rel=0';f.allow='autoplay; encrypted-media; picture-in-picture';f.allowFullscreen=true;f.title='数多くの日本人を泣かせた伝説のCM';f.style.cssText='position:absolute;inset:0;width:100%;height:100%;border:0';this.replaceWith(f)"><img src="https://i.ytimg.com/vi/J_hH4zZ5Uvs/hqdefault.jpg" alt="数多くの日本人を泣かせた伝説のCM" loading="lazy" style="width:100%;height:100%;object-fit:cover;display:block;"><span style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%);width:64px;height:64px;border-radius:50%;background:rgba(0,0,0,.65);color:#fff;font-size:30px;line-height:64px;text-align:center;">▶</span></button>
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
        .at("/video/fukuyama/fukuyama-michishirube-2022.mp4", StaticFileEndpoint::new("video/fukuyama/fukuyama-michishirube-2022.mp4"))
        .at("/video/fukuyama/fukuyama-ano-natsu-mo-umi-mo-sora-mo.mp4", StaticFileEndpoint::new("video/fukuyama/fukuyama-ano-natsu-mo-umi-mo-sora-mo.mp4"))
        .at("/video/fukuyama/fukuyama-hitotsuboshi.mp4", StaticFileEndpoint::new("video/fukuyama/fukuyama-hitotsuboshi.mp4"))
        .at("/video/fukuyama/fukuyama-hotaru.mp4", StaticFileEndpoint::new("video/fukuyama/fukuyama-hotaru.mp4"))
        .at("/video/fukuyama/fukuyama-sakurazaka-2005.mp4", StaticFileEndpoint::new("video/fukuyama/fukuyama-sakurazaka-2005.mp4"))
        .at("/video/fukuyama/shibasaki-sakurazaka-itsukushima.mp4", StaticFileEndpoint::new("video/fukuyama/shibasaki-sakurazaka-itsukushima.mp4"))
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
