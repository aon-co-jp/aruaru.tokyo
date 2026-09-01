// /open-aruaru-runo-iLumi (エイリアス /open-aruaru-runo) ページの
// クライアントサイド挙動。旧 src/meta_index.rs 内のインラインscriptを
// そのまま抽出したもの。挙動は変更していない。
//
// per-request/言語依存の文字列(取得中…等のi18nラベル)は、`<script
// type="application/json" id="page-data">` のJSONデータアイランド経由で
// 受け取る。
(function () {
  'use strict';

  const dataEl = document.getElementById('page-data');
  const pageData = dataEl ? JSON.parse(dataEl.textContent) : {};

  document.querySelectorAll('.live-fetch-btn').forEach(btn => {
    const repo = btn.getAttribute('data-repo');
    const resultEl = document.querySelector('.live-fetch-result[data-repo-result="' + repo + '"]');
    btn.addEventListener('click', async () => {
      resultEl.textContent = pageData.loading;
      try {
        const res = await fetch('https://api.github.com/repos/' + repo, {
          headers: { 'Accept': 'application/vnd.github+json' }
        });
        if (!res.ok) throw new Error('status ' + res.status);
        const data = await res.json();
        const stars = data.stargazers_count;
        const updated = data.pushed_at || data.updated_at;
        const branch = data.default_branch;
        resultEl.innerHTML = '<span class="ok">' + pageData.stars + ': ' + stars +
          ' / ' + pageData.updated + ': ' + new Date(updated).toLocaleDateString() +
          ' / ' + pageData.branch + ': ' + branch + '</span>';
      } catch (e) {
        resultEl.textContent = pageData.fail;
      }
    });
  });
})();
