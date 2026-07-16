# PORTING.md — ملفات قابلة للنقل

قائمة بأنماط التنفيذ القابلة للنقل كما هي (أو بتعديلات طفيفة) إلى مشاريع أخرى.

## `markdown_to_rs()` (src/main.rs)

دالة تحوّل كل سطر من Markdown إلى أسلوب تعليق rustdoc مسبوق بـ`//!`. قابلة للاستخدام العام في README.md/CLAUDE.md/PORTING.md على حد سواء. يمكن نسخها كما هي إلى مستودعات أخرى تتبنى مفهوم readme-to-rs.

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

مساعد يجلب محتوى GitHub الخام بدون مصادقة (مع رجوع احتياطي من main إلى master). قابل لإعادة الاستخدام لميزات تكامل GitHub الأخرى بمجرد استبدال اسم المؤسسة وقيمة المهلة الزمنية.

## نمط nginx "أولوية conf.d"

يتم تضمين `/etc/nginx/conf.d/*.conf` في `nginx.conf` قبل `/etc/nginx/sites-enabled/*.conf`، لذا فإن وضع تكوين بنفس `server_name` في `conf.d/` يمنحه الأولوية على التكوينات التي تُنشئها تلقائيًا أدوات الواجهة (مثل aruaru-easyweb) في `sites-enabled/`. تقنية قابلة لإعادة الاستخدام عند حدوث تعارض مماثل في نطاق آخر (راجع `CLAUDE.md` للتفاصيل).

## واجهة برمجة تطبيقات GitHub: الفرق بين نقاط النهاية الخاصة بالمؤسسة والحساب الشخصي

تعمل `GET /orgs/{name}/repos` فقط مع مؤسسات GitHub؛ استدعاؤها لحساب شخصي (User) يُرجع خطأ 404 (تم التأكد منه على بنية تحتية فعلية). للحصول على القائمة الكاملة لمستودعات حساب شخصي، استخدم بدلاً من ذلك `GET /users/{name}/repos`. `aon-co-jp` هو حساب شخصي، وليس مؤسسة.

```rust
// ✗ لا يعمل مع حساب شخصي (يُرجع 404)
let url = format!("https://api.github.com/orgs/{name}/repos?per_page=100");
// ✓ صحيح لحساب شخصي
let url = format!("https://api.github.com/users/{name}/repos?per_page=100");
```

عند نقل ميزات تكامل GitHub إلى مشاريع أخرى، تحقق مسبقًا عبر `GET /users/{name}` (ما إذا كان حقل `"type"` هو `"Organization"` أو `"User"`)، أو فكّر في حل احتياطي يجرّب كلا نقطتي النهاية.

## `markdown_to_github_style_html()` + `pulldown-cmark` (src/main.rs)

ميزة تعرض فعليًا ملف README خارجي بأسلوب GitHub للعرض. نمط لدمج هذا مع عرض تحويل `.rs` (`markdown_to_rs`) وجعلهما قابلين للتبديل عبر علامات التبويب.

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

إعداد CSS بأسلوب `.markdown-body` في وجهة العرض (تسطير العناوين، كتل الأكواد، حدود الجداول، إلخ) يقرّب المظهر من مظهر GitHub.

## الخروج من حاوية `main` ضيقة (التمركز بناءً على vw)

عندما تحتاج الصفحة بأكملها إلى البقاء ضيقة من أجل القابلية للقراءة (مثل `max-width: 780px`)، لكن قسمًا معينًا (مثل عرض README) يحتاج إلى التمدد ليشغل عرض الشاشة بالكامل:

```css
section.wide {
  width: 94vw;
  max-width: 1400px;
  position: relative;
  left: 50%;
  transform: translateX(-50%);
}
```

هذا ينشئ قسمًا عريضًا متمركزًا بالنسبة لمنفذ العرض، بغض النظر عن `max-width` الخاص بالحاوية الأصل.

## التحقق من صحة القوائم المجلوبة ديناميكيًا عبر فحص التنسيق (وليس عبر مطابقة قائمة بيضاء)

عندما ترغب في قبول كل من قائمة مُدرجة بشكل ثابت عند بدء تشغيل الخادم (مثل `GITHUB_REPOS`) وقائمة يتم جلبها ديناميكيًا من واجهة برمجة تطبيقات أثناء التشغيل، فإن التحقق من صحة الإدخال عبر "المطابقة مع القائمة الثابتة" سيرفض القيم الجديدة الموجودة فقط في القائمة الديناميكية. بدلاً من ذلك، تحقق عبر "هل التنسيق صحيح" (أحرف وأرقام/شرطة/شرطة سفلية/نقطة فقط، حدود الطول، إلخ).

## نمط "موقع المرآة" (توكيل داخلي عبر إعادة كتابة رأس Host)

عندما ترغب في التوكيل داخليًا فقط لمسارات معينة ضمن نطاق واحد إلى المحتوى الفعلي لنطاق آخر:

```nginx
location /some-path/ {
    proxy_pass http://127.0.0.1:80/some-path/;
    proxy_set_header Host other-domain.example;
    proxy_set_header X-Real-IP $remote_addr;
}
```

قابل لإعادة الاستخدام كلما تعايشت عدة نطاقات على نفس VPS ورغبت في جعل المحتوى قابلًا للعرض من نطاق آخر دون تكراره.
