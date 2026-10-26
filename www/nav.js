// Zero-dep nav: speculation-rules prefetch on hover + same-origin
// fetch/swap navigation so internal links feel instant without a
// framework. Progressive enhancement — if any step fails the browser
// just does a normal full load.
(function () {
  // nav.js sits inside the swapped content region, so every SPA nav
  // re-executes it. Without this guard each swap stacks another copy
  // of the click + popstate listeners — eventually five handlers race
  // to fetch/swap the same link and the page state goes sideways.
  if (window.__navSwapInit) return;
  window.__navSwapInit = true;
  // 1. Speculation rules — prefetch, NOT prerender. Prerender runs the
  //    page's JS in a hidden context where layout APIs (rects, viewport)
  //    report 0s; on activation those stale measurements stick and the
  //    page renders broken. Prefetch warms bytes only — scripts still
  //    run in a visible document on nav. Firefox/Safari ignore the
  //    script type; harmless.
  if (HTMLScriptElement.supports && HTMLScriptElement.supports('speculationrules')) {
    var s = document.createElement('script');
    s.type = 'speculationrules';
    s.textContent = JSON.stringify({
      prefetch: [{
        source: 'document',
        where: { and: [{ href_matches: '/*' }, { not: { href_matches: ['/admin/*', '/wp-login*', '/*?*'] } }] },
        eagerness: 'moderate'
      }]
    });
    document.head.appendChild(s);
  }

  // 2. Same-origin nav swap: fetch → replace <title> + the outer content
  //    wrapper → pushState.
  //
  //    Site-agnostic by design: we locate the outermost element that
  //    wraps everything between <header> and <footer> — the shape is
  //    always "one container holding everything but the chrome". Falls
  //    back to "every body child that isn't header/footer/script/style"
  //    for themes without a single wrapper.
  //
  //    Earlier versions swapped a per-theme selector list by index —
  //    broke the moment two pages had different block counts (home vs
  //    post: extra page-title-wrap) and required a hand-tuned list per
  //    theme.

  function sameDoc(url) {
    try {
      var u = new URL(url, location.href);
      return u.origin === location.origin;
    } catch (_) { return false; }
  }

  function contentRoot(doc) {
    // Try the most specific known wrappers first, then any single
    // element that contains both a <header> and a <footer>.
    var CANDIDATES = ['.body-wrapper', '.site-main', '#content', 'main', '#main'];
    for (var i = 0; i < CANDIDATES.length; i++) {
      var el = doc.querySelector(CANDIDATES[i]);
      if (el && el.querySelector('header') && el.querySelector('footer')) return el;
    }
    // Generic: find the element containing both landmarks
    var all = doc.querySelectorAll('body > *');
    for (var i = 0; i < all.length; i++) {
      var el = all[i];
      if (el.querySelector('header') && el.querySelector('footer')) return el;
    }
    return null;
  }

  function reviveScripts(container) {
    container.querySelectorAll('script').forEach(function (old) {
      var s = document.createElement('script');
      if (old.src) s.src = old.src; else s.textContent = old.textContent;
      old.replaceWith(s);
    });
  }

  function mergeHeadTags(newDoc) {
    // Post-kind pages inject assets the shared head lacks: TOC css,
    // comment/review styles, rmwr styles. Swap them by element id —
    // every plugin tag carries one — so a blog→post navigation picks
    // them up and a post→page navigation drops them.
    var SEL = 'style[id], script[id]';
    var nextIds = new Set();
    newDoc.querySelectorAll(SEL).forEach(function (el) {
      nextIds.add(el.id);
      var live = document.getElementById(el.id);
      if (live) live.outerHTML = el.outerHTML;
      else document.head.appendChild(el);
      document.getElementById(el.id).dataset.navMerged = '1';
    });
    // Drop tags this page doesn't declare — but only ones that came
    // from a previous swap merge, never the base template's own.
    document.querySelectorAll('head ' + SEL).forEach(function (el) {
      if (!nextIds.has(el.id) && el.dataset.navMerged) el.remove();
    });
  }

  // NOTE: themes that transform menu markup at ready() (dropdown
  // builders, hover binding, superfish-style wrappers) must re-run
  // their init after a swap — the fetched page ships raw markup and
  // the new DOM has no listeners. Call them from a `navswap` hook or
  // inside `navigate()` below; nothing theme-specific lives here.

  function navigate(url, push) {
    fetch(url, { credentials: 'same-origin' })
      .then(function (r) {
        if (!r.ok) { location.href = url; return null; }
        return r.text();
      })
      .then(function (txt) {
        if (txt === null) return;
        var doc = new DOMParser().parseFromString(txt, 'text/html');
        var next = contentRoot(doc);
        var cur = contentRoot(document);
        if (!next || !cur) { location.href = url; return; }
        document.title = doc.title;
        // Merge head <style>/<script> by id — post-kind pages inject
        // assets the shared head lacks (TOC css, comment/review styles),
        // and swapping only the content leaves them unstyled/dead.
        mergeHeadTags(doc);
        cur.replaceWith(next);
        reviveScripts(next);
        // Theme JS that bound to nodes inside the swapped region is
        // dead now — emit a hook so themes can re-init menus, sliders,
        // galleries. Nothing theme-specific lives in nav.js itself.
        document.dispatchEvent(new CustomEvent('pressrs:swapped', { detail: { url: url } }));
        if (push) history.pushState(null, '', url);
        lastNavUrl = url;
        window.scrollTo(0, 0);
      })
      .catch(function () { location.href = url; });
  }

  document.addEventListener('click', function (e) {
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    var a = e.target.closest('a');
    if (!a) return;
    var href = a.getAttribute('href');
    if (!href || a.target || a.hasAttribute('download') || a.dataset.noNav !== undefined) return;
    // Same-page anchor (#id) — let the browser scroll to the element
    // instead of fetch+swap+scrollTo(0,0).
    if (href.charAt(0) === '#') return;
    var dest = new URL(href, location.href);
    if (dest.pathname === location.pathname && dest.hash) return;
    if (!sameDoc(href)) return;
    // Let the speculation prefetch handle it when available — a warm
    // full nav beats fetch+swap and runs scripts correctly.
    if (HTMLScriptElement.supports && HTMLScriptElement.supports('speculationrules')) return;
    e.preventDefault();
    navigate(a.href, true);
  });

  var lastNavUrl = location.href;
  window.addEventListener('popstate', function () {
    // Same-document hash hops (TOC clicks, fragment links) shouldn't
    // re-fetch the page — they only move scroll position.
    var cur = new URL(lastNavUrl);
    var now = new URL(location.href);
    if (cur.pathname === now.pathname && cur.search === now.search) { lastNavUrl = location.href; return; }
    lastNavUrl = location.href;
    navigate(location.href, false);
  });
})();
