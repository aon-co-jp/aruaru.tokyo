# aruaru-tokyo-server

الصفحة الرئيسية لموقع [aruaru.tokyo](https://aruaru.tokyo/). مكتوبة بلغة Rust + [Poem](https://github.com/poem-web/poem)، بدون اعتماد على قاعدة بيانات، وملف تنفيذي واحد فقط.

موقع شقيق لـ`audiocafe.tokyo` (PHP) على نطاق مختلف وبتقنية مختلفة، تم تنفيذه وفق اتفاقية نظام poem-cosmo-tauri البيئي (استخدام hyper/Poem مباشرة، دون أطر عمل ثقيلة أو اعتماد على قاعدة بيانات).

## الميزات

- محتوى "أروارو" (لحظات يومية يتفق عليها الجميع) موزعة على 5 فئات، مع عرض عشوائي
- رابط سريع إلى [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb)
- رابط إلى الصفحة الرئيسية لحساب GitHub (aon-co-jp)
- زر "🔄 جلب أحدث قائمة مستودعات" يحدّث خيارات المستودعات ديناميكيًا عبر واجهة برمجة تطبيقات GitHub
- رابط مباشر إلى صفحة GitHub الخاصة بالمستودع المحدد
- عرض README.md وCLAUDE.md وPORTING.md إما كـHTML مُنسّق بأسلوب GitHub أو كنص بأسلوب تعليقات rustdoc (`//!`) على شكل `.rs`، مع إمكانية التبديل عبر علامات تبويب (تطبيق لفكرة readme-to-rs). تشغل منطقة العرض معظم عرض الشاشة (94vw، حتى 1400px).

## البناء والتشغيل

```bash
cargo build --release
ARUARU_TOKYO_BIND=0.0.0.0:4100 ./target/release/aruaru-tokyo-server
```

يستمع على `0.0.0.0:4100` في حال عدم تحديد `ARUARU_TOKYO_BIND`.

## إعداد الإنتاج (مرجعي)

يعمل على VPS كخدمة systemd مرتبطة بـ`127.0.0.1:4100`، حيث ينهي nginx اتصال TLS على المنفذ 443 ويقوم بالتوكيل العكسي. تُعكس المسارات `/aruaru/` و`/aruaru-lady/` و`/rakuten-mobile/` إلى المحتوى الفعلي على `audiocafe.tokyo` (PHP) عبر كتل `location` مخصصة ضمن نفس ملف vhost الخاص بـnginx (راجع [CLAUDE.md](CLAUDE.md) للتفاصيل).

## المشاريع ذات الصلة

- [open-runo](https://github.com/aon-co-jp/open-runo) — النواة الأساسية لنظام open-runo البيئي (Rust→WASM/tokio+hyper)
- [poem-cosmo-tauri](https://github.com/aon-co-jp/poem-cosmo-tauri) — مصدر اتفاقية تنفيذ Poem/Tauri
- [open-web-server](https://github.com/aon-co-jp/open-web-server) — بوابة خادم ويب عامة
- [aruaru-db](https://github.com/aon-co-jp/aruaru-db) — طبقة قاعدة البيانات (هذا المستودع مستقل عنها)
- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — المصدر المرجعي لقواعد التطوير
- [aruaru-easyweb](https://github.com/aon-co-jp/aruaru-easyweb) — خادم مصادقة OTP وإدارة المواقع (tokio+hyper)
- [audiocafe.tokyo](https://github.com/aon-co-jp/audiocafe-tokyo) (PHP)
