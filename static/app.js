// aruaru.tokyo TOPページ ( / ) のクライアントサイド挙動。
// 旧 src/main.rs 内のインラインscript(2ブロック: shuffleボタン + repo一覧
// 取得/view-toggle/YouTube facade)をそのまま抽出したもの。挙動は変更していない。
//
// サーバー側からの per-request な値は、`<script type="application/json"
// id="page-data">` のJSONデータアイランド経由で受け取る(現時点では
// あるある一覧のシャッフル用データ`items`のみ)。
(function () {
  'use strict';

  const dataEl = document.getElementById('page-data');
  const pageData = dataEl ? JSON.parse(dataEl.textContent) : {};
  const items = pageData.items || [];

  // 🎲 ランダムに1つ表示ボタン。
  const btn = document.getElementById('shuffle-btn');
  const result = document.getElementById('shuffle-result');
  if (btn && result) {
    btn.addEventListener('click', () => {
      const pick = items[Math.floor(Math.random() * items.length)];
      result.querySelector('.cat').textContent = pick.category;
      result.querySelector('.txt').textContent = pick.text;
      result.style.display = 'block';
    });
  }

  // ボタンを押した瞬間にGitHub組織の最新リポジトリ一覧をAPI経由で取得し、
  // <select>を動的に差し替える。
  const fetchBtn = document.getElementById('fetch-repos-btn');
  const fetchStatus = document.getElementById('fetch-repos-status');
  const repoSelect = document.getElementById('repo-select');
  if (fetchBtn && fetchStatus && repoSelect) {
    fetchBtn.addEventListener('click', async () => {
      fetchStatus.textContent = '取得中… / Fetching…';
      try {
        const res = await fetch('/api/repos');
        const data = await res.json();
        if (data.error) { fetchStatus.textContent = '❌ ' + data.error; return; }
        const repos = data.repos || [];
        const current = repoSelect.value;
        repoSelect.innerHTML = '<option value="">リポジトリを選択…</option>' +
          repos.map(r => `<option value="${r}">${r}</option>`).join('');
        if (repos.includes(current)) repoSelect.value = current;
        fetchStatus.textContent = `✅ ${repos.length}件取得しました。`;
      } catch (e) {
        fetchStatus.textContent = '❌ 取得に失敗しました。';
      }
    });
  }

  // GitHub風表示 / .rs形式 の切替タブ配線。
  document.querySelectorAll('.view-toggle').forEach(toggle => {
    const tabId = toggle.getAttribute('data-tab');
    const ghEl = document.getElementById('gh-' + tabId);
    const rsEl = document.getElementById('rs-' + tabId);
    toggle.querySelectorAll('.view-toggle-btn').forEach(b => {
      b.addEventListener('click', () => {
        toggle.querySelectorAll('.view-toggle-btn').forEach(x => x.classList.remove('active'));
        b.classList.add('active');
        const view = b.getAttribute('data-view');
        if (view === 'gh') { ghEl.classList.remove('hidden'); rsEl.classList.add('hidden'); }
        else { ghEl.classList.add('hidden'); rsEl.classList.remove('hidden'); }
      });
    });
  });

  // YouTube埋め込みのプログレッシブ表示化(2026-08-12追加、ユーザー指示
  // 「連続でYoutube動画埋め込みをしているとスクロールが早いと表示が
  // 遅れる事があるので、サムネイルの埋め込み動画の写真を素早く表示する
  // ようにプログレッシブな感じで」への対応)。
  //
  // 正直な開示: ページ内の全<iframe src="https://www.youtube.com/embed/...">
  // を、実際のiframe(重いYouTubeプレイヤー本体)ではなく、YouTube公式の
  // サムネイルCDN(img.youtube.com)から取得した軽量な静止画(再生ボタン
  // オーバーレイ付き)へ自動的に差し替える(YouTube Facadeパターン、
  // lite-youtube-embed等で広く使われている既知の手法)。サムネイル画像は
  // ブラウザ標準の`loading="lazy"`で画面内に近づいた時のみ読み込まれる
  // ため、22本の動画を一度に全部読み込む従来方式より大幅に軽くなる。
  // クリックした時点で初めて実際のiframe(自動再生付き)へ差し替え、
  // その動画だけを再生する——動画自体の視聴体験は変わらない。
  document.querySelectorAll('iframe[src*="youtube.com/embed/"]').forEach(iframe => {
    const src = iframe.getAttribute('src');
    const match = src.match(/embed\/([a-zA-Z0-9_-]+)/);
    if (!match) return;
    const videoId = match[1];
    const title = iframe.getAttribute('title') || 'YouTube video';
    const wrapper = iframe.parentElement;
    if (!wrapper) return;

    const facade = document.createElement('div');
    facade.className = 'yt-facade';
    facade.style.cssText = 'position:absolute; inset:0; cursor:pointer; background:#000;';
    facade.setAttribute('role', 'button');
    facade.setAttribute('aria-label', '▶ ' + title);

    const img = document.createElement('img');
    img.src = 'https://img.youtube.com/vi/' + videoId + '/hqdefault.jpg';
    img.loading = 'lazy';
    img.alt = title;
    img.style.cssText = 'width:100%; height:100%; object-fit:cover; display:block;';
    facade.appendChild(img);

    const playBtn = document.createElement('div');
    playBtn.textContent = '▶';
    playBtn.style.cssText = 'position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); font-size:3rem; color:#fff; text-shadow:0 0 12px rgba(0,0,0,.8); pointer-events:none;';
    facade.appendChild(playBtn);

    facade.addEventListener('click', () => {
      const realIframe = document.createElement('iframe');
      realIframe.width = '100%';
      realIframe.height = '100%';
      realIframe.style.cssText = 'position:absolute; top:0; left:0; width:100%; height:100%; border:0; border-radius:6px;';
      realIframe.src = 'https://www.youtube.com/embed/' + videoId + '?autoplay=1';
      realIframe.title = title;
      realIframe.setAttribute('frameborder', '0');
      realIframe.setAttribute('allow', 'accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture');
      realIframe.allowFullscreen = true;
      wrapper.replaceChild(realIframe, facade);
    }, { once: true });

    wrapper.replaceChild(facade, iframe);
  });
})();
