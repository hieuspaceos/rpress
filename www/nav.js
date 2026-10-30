// Zero-dep nav hint: speculation-rules prefetch so internal links
// feel instant without a framework.
//
// This file used to also do SPA-style fetch/swap navigation. That is
// gone: jQuery-era WP themes (superfish, sticky nav, parallax, mmenu,
// flexslider) bind listeners in jQuery(document).ready and are NOT
// idempotent — a swapped DOM has no bindings (dead menus, missing
// backgrounds), and re-executing the theme files stacks duplicate
// window-level handlers (traveltour's sticky-nav was observed
// re-inserting a cloned header → two nav bars). No amount of
// element-level patching fixes that class of bug, so navigation is
// left to the browser.
//
// Prefetch keeps most of the win: on hover/intent the HTML is warmed
// into the HTTP cache, so the real navigation renders near-instantly
// and every script still runs exactly once per document.
(function () {
  if (window.__navSwapInit) return;
  window.__navSwapInit = true;
  // Prefetch, NOT prerender. Prerender runs the page's JS in a hidden
  // context where layout APIs (rects, viewport) report 0s; on
  // activation those stale measurements stick and the page renders
  // broken. Prefetch warms bytes only — scripts still run in a
  // visible document on nav. Firefox/Safari ignore the script type;
  // harmless.
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
})();
