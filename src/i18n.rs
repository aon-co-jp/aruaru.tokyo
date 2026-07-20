//! `/open-aruaru-runo-iLumi`(エイリアス`/open-aruaru-runo`)ページの
//! 多言語切り替え(クエリパラメータ`?lang=`方式)。
//!
//! 対応言語は`e-gov.info`(姉妹リポジトリ)の`src/i18n.rs`で定めた基本13言語
//! セットと同じ(日本語・英語(米)・英語(英)・中国語簡体字・中国語繁体字・
//! 韓国語・イタリア語・フランス語・ドイツ語・アラビア語・ペルシャ語・
//! ロシア語・ウクライナ語)。
//!
//! **既定言語は日本語**——このサイト(`aruaru-tokyo`)の既存TOPページが
//! `lang="ja"`を既定としているため、それに揃える(e-gov.infoが英語既定
//! なのとは異なる、サイトごとの既存方針に合わせた判断)。
//!
//! **正直な開示(スコープの限界)**: ページの見出し・ラベル・ボタン等の
//! UI文言はこの13言語すべてに翻訳しているが、各プロジェクトの役割説明
//! (`meta_index.rs`の`PROJECTS`)自体は日本語のみで記載しており、13言語へは
//! 展開していない(16プロジェクト分の説明文を13言語へ翻訳すると分量が
//! 膨大になり、翻訳精度の検証も難しくなるため。各プロジェクトのREADMEへの
//! リンクを辿れば、README自体が多言語対応しているプロジェクトについては
//! そちらで確認できる)。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ja,
    En,
    EnGb,
    ZhCn,
    ZhTw,
    Ko,
    It,
    Fr,
    De,
    Ar,
    Fa,
    Ru,
    Uk,
}

impl Lang {
    pub const ALL: &'static [Lang] = &[
        Lang::Ja, Lang::En, Lang::EnGb, Lang::ZhCn, Lang::ZhTw, Lang::Ko, Lang::It, Lang::Fr, Lang::De, Lang::Ar,
        Lang::Fa, Lang::Ru, Lang::Uk,
    ];

    /// URLクエリパラメータで使うコード(`?lang=xx`)。
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ja => "ja",
            Lang::En => "en",
            Lang::EnGb => "en-gb",
            Lang::ZhCn => "zh-cn",
            Lang::ZhTw => "zh-tw",
            Lang::Ko => "ko",
            Lang::It => "it",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::Ar => "ar",
            Lang::Fa => "fa",
            Lang::Ru => "ru",
            Lang::Uk => "uk",
        }
    }

    /// `<html lang="...">`属性用(BCP 47)。
    pub fn html_lang(self) -> &'static str {
        match self {
            Lang::Ja => "ja",
            Lang::En => "en",
            Lang::EnGb => "en-GB",
            Lang::ZhCn => "zh-CN",
            Lang::ZhTw => "zh-TW",
            Lang::Ko => "ko",
            Lang::It => "it",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::Ar => "ar",
            Lang::Fa => "fa",
            Lang::Ru => "ru",
            Lang::Uk => "uk",
        }
    }

    /// 言語切替ナビに表示する現地語名。
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Ja => "日本語",
            Lang::En => "English",
            Lang::EnGb => "English (UK)",
            Lang::ZhCn => "简体中文",
            Lang::ZhTw => "繁體中文",
            Lang::Ko => "한국어",
            Lang::It => "Italiano",
            Lang::Fr => "Français",
            Lang::De => "Deutsch",
            Lang::Ar => "العربية",
            Lang::Fa => "فارسی",
            Lang::Ru => "Русский",
            Lang::Uk => "Українська",
        }
    }

    pub fn is_rtl(self) -> bool {
        matches!(self, Lang::Ar | Lang::Fa)
    }

    /// クエリパラメータの値から`Lang`を決定する。未知の値・未指定の
    /// 場合は既定言語の日本語にフォールバックする。
    pub fn parse(code: Option<&str>) -> Lang {
        match code.map(str::to_lowercase).as_deref() {
            Some("en") => Lang::En,
            Some("en-gb") | Some("en_gb") | Some("engb") => Lang::EnGb,
            Some("zh-cn") | Some("zh_cn") | Some("zhcn") | Some("zh") => Lang::ZhCn,
            Some("zh-tw") | Some("zh_tw") | Some("zhtw") => Lang::ZhTw,
            Some("ko") => Lang::Ko,
            Some("it") => Lang::It,
            Some("fr") => Lang::Fr,
            Some("de") => Lang::De,
            Some("ar") => Lang::Ar,
            Some("fa") => Lang::Fa,
            Some("ru") => Lang::Ru,
            Some("uk") => Lang::Uk,
            Some("ja") => Lang::Ja,
            _ => Lang::Ja,
        }
    }
}

/// `/open-aruaru-runo-iLumi`ページのUI文言(見出し・ラベル・ボタン等)。
pub struct IndexStrings {
    pub title: &'static str,
    pub h1: &'static str,
    pub intro: &'static str,
    pub org_link_label: &'static str,
    pub label_readme: &'static str,
    pub label_porting: &'static str,
    /// CLAUDE.mdのラベル。「WEBアプリ設計思想、開発方針、開発環境ルール」
    /// という日本語表記は、ユーザー指示によりリンクテキストとしてこの
    /// 日本語名で表示することが明示されているため、言語に関わらず常に含める。
    pub label_claude: &'static str,
    pub label_none: &'static str,
    pub btn_fetch_live: &'static str,
    pub fetch_loading: &'static str,
    pub fetch_fail: &'static str,
    pub field_stars: &'static str,
    pub field_updated: &'static str,
    pub field_default_branch: &'static str,
    pub back_to_top: &'static str,
}

const LABEL_CLAUDE_JA: &str = "CLAUDE.md(WEBアプリ設計思想、開発方針、開発環境ルール)";

pub fn index_strings(lang: Lang) -> IndexStrings {
    match lang {
        Lang::Ja => IndexStrings {
            title: "プロジェクトシリーズ索引 | aruaru.tokyo",
            h1: "📚 プロジェクトシリーズ索引",
            intro: "aon-co-jp organization配下、open-cosmoエコシステム全体のプロジェクト一覧です(メタリポジトリ aon〔旧称: open-aruaru-runo-iLumi〕と同内容)。各プロジェクト名をクリックすると、その場でGitHub APIから最新情報(⭐Star数・最終更新日時・既定ブランチ)を取得して表示します。",
            org_link_label: "🏢 GitHub organization: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(このリポジトリには存在しません)",
            btn_fetch_live: "🔄 GitHubから最新情報を取得",
            fetch_loading: "取得中…",
            fetch_fail: "❌ 取得できませんでした(レート制限またはネットワーク到達不能)。静的情報のみ表示しています。",
            field_stars: "⭐ Stars",
            field_updated: "最終更新",
            field_default_branch: "既定ブランチ",
            back_to_top: "← TOP",
        },
        Lang::En => IndexStrings {
            title: "Project Series Index | aruaru.tokyo",
            h1: "📚 Project Series Index",
            intro: "A list of every project under the aon-co-jp GitHub organization / open-cosmo ecosystem (same content as the aon meta repository, formerly named open-aruaru-runo-iLumi). Click a project's live-fetch button to pull its latest info (stars, last updated, default branch) from the GitHub API on the spot.",
            org_link_label: "🏢 GitHub organization: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(not present in this repository)",
            btn_fetch_live: "🔄 Fetch latest from GitHub",
            fetch_loading: "Fetching…",
            fetch_fail: "❌ Could not fetch (rate limit or network unreachable). Showing static info only.",
            field_stars: "⭐ Stars",
            field_updated: "Last updated",
            field_default_branch: "Default branch",
            back_to_top: "← TOP",
        },
        Lang::EnGb => IndexStrings {
            title: "Project Series Index | aruaru.tokyo",
            h1: "📚 Project Series Index",
            intro: "A list of every project under the aon-co-jp GitHub organisation / open-cosmo ecosystem (same content as the aon meta repository, formerly named open-aruaru-runo-iLumi). Click a project's live-fetch button to pull its latest information (stars, last updated, default branch) from the GitHub API on the spot.",
            org_link_label: "🏢 GitHub organisation: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(not present in this repository)",
            btn_fetch_live: "🔄 Fetch latest from GitHub",
            fetch_loading: "Fetching…",
            fetch_fail: "❌ Could not fetch (rate limit or network unreachable). Showing static information only.",
            field_stars: "⭐ Stars",
            field_updated: "Last updated",
            field_default_branch: "Default branch",
            back_to_top: "← TOP",
        },
        Lang::ZhCn => IndexStrings {
            title: "项目系列索引 | aruaru.tokyo",
            h1: "📚 项目系列索引",
            intro: "aon-co-jp organization下、open-cosmo生态系统的全部项目一览(与元仓库aon〔原名open-aruaru-runo-iLumi〕内容相同)。点击项目的实时获取按钮，即可当场从GitHub API获取最新信息(⭐Star数・最后更新时间・默认分支)。",
            org_link_label: "🏢 GitHub organization: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(此仓库中不存在)",
            btn_fetch_live: "🔄 从GitHub获取最新信息",
            fetch_loading: "获取中…",
            fetch_fail: "❌ 获取失败(达到速率限制或网络不可达)。仅显示静态信息。",
            field_stars: "⭐ Star数",
            field_updated: "最后更新",
            field_default_branch: "默认分支",
            back_to_top: "← 首页",
        },
        Lang::ZhTw => IndexStrings {
            title: "專案系列索引 | aruaru.tokyo",
            h1: "📚 專案系列索引",
            intro: "aon-co-jp organization底下、open-cosmo生態系統的全部專案一覽(與元儲存庫aon〔原名open-aruaru-runo-iLumi〕內容相同)。點擊專案的即時取得按鈕，即可當場從GitHub API取得最新資訊(⭐Star數・最後更新時間・預設分支)。",
            org_link_label: "🏢 GitHub organization: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(此儲存庫中不存在)",
            btn_fetch_live: "🔄 從GitHub取得最新資訊",
            fetch_loading: "取得中…",
            fetch_fail: "❌ 取得失敗(達到速率限制或網路無法連線)。僅顯示靜態資訊。",
            field_stars: "⭐ Star數",
            field_updated: "最後更新",
            field_default_branch: "預設分支",
            back_to_top: "← 首頁",
        },
        Lang::Ko => IndexStrings {
            title: "프로젝트 시리즈 색인 | aruaru.tokyo",
            h1: "📚 프로젝트 시리즈 색인",
            intro: "aon-co-jp organization 산하, open-cosmo 생태계 전체 프로젝트 목록입니다(메타 저장소 aon〔이전 명칭: open-aruaru-runo-iLumi〕와 동일한 내용). 각 프로젝트의 실시간 조회 버튼을 클릭하면 GitHub API에서 최신 정보(⭐Star 수・최종 업데이트・기본 브랜치)를 즉시 가져와 표시합니다.",
            org_link_label: "🏢 GitHub organization: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(이 저장소에는 존재하지 않습니다)",
            btn_fetch_live: "🔄 GitHub에서 최신 정보 가져오기",
            fetch_loading: "가져오는 중…",
            fetch_fail: "❌ 가져오지 못했습니다(속도 제한 또는 네트워크 연결 불가). 정적 정보만 표시합니다.",
            field_stars: "⭐ Star 수",
            field_updated: "최종 업데이트",
            field_default_branch: "기본 브랜치",
            back_to_top: "← TOP",
        },
        Lang::It => IndexStrings {
            title: "Indice della serie di progetti | aruaru.tokyo",
            h1: "📚 Indice della serie di progetti",
            intro: "Un elenco di tutti i progetti dell'organizzazione GitHub aon-co-jp / ecosistema open-cosmo (stesso contenuto del meta-repository aon, ex open-aruaru-runo-iLumi). Fai clic sul pulsante di aggiornamento in tempo reale di un progetto per recuperare al volo le informazioni più recenti (stelle, ultimo aggiornamento, branch predefinito) dall'API di GitHub.",
            org_link_label: "🏢 Organizzazione GitHub: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(non presente in questo repository)",
            btn_fetch_live: "🔄 Recupera le ultime info da GitHub",
            fetch_loading: "Recupero in corso…",
            fetch_fail: "❌ Recupero non riuscito (limite di frequenza o rete non raggiungibile). Vengono mostrate solo le informazioni statiche.",
            field_stars: "⭐ Stelle",
            field_updated: "Ultimo aggiornamento",
            field_default_branch: "Branch predefinito",
            back_to_top: "← TOP",
        },
        Lang::Fr => IndexStrings {
            title: "Index de la série de projets | aruaru.tokyo",
            h1: "📚 Index de la série de projets",
            intro: "Une liste de tous les projets de l'organisation GitHub aon-co-jp / de l'écosystème open-cosmo (même contenu que le méta-dépôt aon, anciennement open-aruaru-runo-iLumi). Cliquez sur le bouton de récupération en direct d'un projet pour obtenir sur-le-champ ses informations les plus récentes (étoiles, dernière mise à jour, branche par défaut) depuis l'API GitHub.",
            org_link_label: "🏢 Organisation GitHub : aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(absent de ce dépôt)",
            btn_fetch_live: "🔄 Récupérer les dernières infos depuis GitHub",
            fetch_loading: "Récupération…",
            fetch_fail: "❌ Échec de la récupération (limite de débit ou réseau injoignable). Seules les informations statiques sont affichées.",
            field_stars: "⭐ Étoiles",
            field_updated: "Dernière mise à jour",
            field_default_branch: "Branche par défaut",
            back_to_top: "← TOP",
        },
        Lang::De => IndexStrings {
            title: "Projektserien-Index | aruaru.tokyo",
            h1: "📚 Projektserien-Index",
            intro: "Eine Liste aller Projekte der GitHub-Organisation aon-co-jp / des open-cosmo-Ökosystems (gleicher Inhalt wie das Meta-Repository aon, vormals open-aruaru-runo-iLumi). Klicken Sie auf die Live-Abruf-Schaltfläche eines Projekts, um dessen aktuelle Informationen (Sterne, letzte Aktualisierung, Standard-Branch) sofort über die GitHub-API abzurufen.",
            org_link_label: "🏢 GitHub-Organisation: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(in diesem Repository nicht vorhanden)",
            btn_fetch_live: "🔄 Aktuelle Infos von GitHub abrufen",
            fetch_loading: "Wird abgerufen…",
            fetch_fail: "❌ Abruf fehlgeschlagen (Ratenlimit oder Netzwerk nicht erreichbar). Es werden nur statische Informationen angezeigt.",
            field_stars: "⭐ Sterne",
            field_updated: "Letzte Aktualisierung",
            field_default_branch: "Standard-Branch",
            back_to_top: "← TOP",
        },
        Lang::Ar => IndexStrings {
            title: "فهرس سلسلة المشاريع | aruaru.tokyo",
            h1: "📚 فهرس سلسلة المشاريع",
            intro: "قائمة بجميع المشاريع ضمن منظمة GitHub التابعة لـ aon-co-jp / نظام open-cosmo البيئي (نفس محتوى المستودع الوصفي aon، المعروف سابقاً باسم open-aruaru-runo-iLumi). انقر على زر الجلب المباشر لأي مشروع لسحب أحدث معلوماته (النجوم، آخر تحديث، الفرع الافتراضي) من واجهة GitHub API فوراً.",
            org_link_label: "🏢 منظمة GitHub: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(غير موجود في هذا المستودع)",
            btn_fetch_live: "🔄 جلب أحدث المعلومات من GitHub",
            fetch_loading: "جارٍ الجلب…",
            fetch_fail: "❌ تعذّر الجلب (حد المعدل أو تعذّر الوصول للشبكة). يتم عرض المعلومات الثابتة فقط.",
            field_stars: "⭐ النجوم",
            field_updated: "آخر تحديث",
            field_default_branch: "الفرع الافتراضي",
            back_to_top: "← الرئيسية",
        },
        Lang::Fa => IndexStrings {
            title: "نمایه سری پروژه‌ها | aruaru.tokyo",
            h1: "📚 نمایه سری پروژه‌ها",
            intro: "فهرستی از تمام پروژه‌های سازمان GitHub با نام aon-co-jp / اکوسیستم open-cosmo (همان محتوای مخزن فراداده aon، با نام سابق open-aruaru-runo-iLumi). روی دکمه دریافت زنده هر پروژه کلیک کنید تا اطلاعات جدید آن (ستاره‌ها، آخرین به‌روزرسانی، شاخه پیش‌فرض) بلافاصله از API گیت‌هاب دریافت شود.",
            org_link_label: "🏢 سازمان GitHub: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(در این مخزن وجود ندارد)",
            btn_fetch_live: "🔄 دریافت آخرین اطلاعات از GitHub",
            fetch_loading: "در حال دریافت…",
            fetch_fail: "❌ دریافت انجام نشد (محدودیت نرخ یا عدم دسترسی به شبکه). فقط اطلاعات ایستا نمایش داده می‌شود.",
            field_stars: "⭐ ستاره‌ها",
            field_updated: "آخرین به‌روزرسانی",
            field_default_branch: "شاخه پیش‌فرض",
            back_to_top: "← صفحه اصلی",
        },
        Lang::Ru => IndexStrings {
            title: "Индекс серии проектов | aruaru.tokyo",
            h1: "📚 Индекс серии проектов",
            intro: "Список всех проектов организации GitHub aon-co-jp / экосистемы open-cosmo (то же содержимое, что и в мета-репозитории aon, ранее известном как open-aruaru-runo-iLumi). Нажмите кнопку живого получения данных у проекта, чтобы тут же получить актуальную информацию (звёзды, последнее обновление, ветка по умолчанию) через GitHub API.",
            org_link_label: "🏢 Организация GitHub: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(отсутствует в этом репозитории)",
            btn_fetch_live: "🔄 Получить актуальные данные с GitHub",
            fetch_loading: "Загрузка…",
            fetch_fail: "❌ Не удалось получить данные (лимит запросов или сеть недоступна). Показана только статическая информация.",
            field_stars: "⭐ Звёзды",
            field_updated: "Последнее обновление",
            field_default_branch: "Ветка по умолчанию",
            back_to_top: "← TOP",
        },
        Lang::Uk => IndexStrings {
            title: "Індекс серії проєктів | aruaru.tokyo",
            h1: "📚 Індекс серії проєктів",
            intro: "Список усіх проєктів організації GitHub aon-co-jp / екосистеми open-cosmo (той самий вміст, що й у мета-репозиторії aon, раніше відомому як open-aruaru-runo-iLumi). Натисніть кнопку живого отримання даних проєкту, щоб одразу отримати актуальну інформацію (зірки, останнє оновлення, гілка за замовчуванням) через GitHub API.",
            org_link_label: "🏢 Організація GitHub: aon-co-jp",
            label_readme: "README.md",
            label_porting: "PORTING.md",
            label_claude: LABEL_CLAUDE_JA,
            label_none: "(відсутній у цьому репозиторії)",
            btn_fetch_live: "🔄 Отримати останні дані з GitHub",
            fetch_loading: "Завантаження…",
            fetch_fail: "❌ Не вдалося отримати дані (ліміт запитів або мережа недоступна). Показано лише статичну інформацію.",
            field_stars: "⭐ Зірки",
            field_updated: "Останнє оновлення",
            field_default_branch: "Гілка за замовчуванням",
            back_to_top: "← TOP",
        },
    }
}
