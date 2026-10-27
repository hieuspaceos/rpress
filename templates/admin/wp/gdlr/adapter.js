/* rpress page-builder adapter: glue between the GoodLayers plugin JS and the
   rpress post form. Runs before framework/js/page-builder.js.

   - The plugin's in-canvas Save button POSTs `gdlr_core_save_page_builder_data`
     to `page_builder_var.text.ajaxurl`; rpress serves that action at
     /admin/api/gdlr/ajax (or the bridge endpoint when `editor_assets` swaps
     the URL). No adapter work needed there.

   - The classic post-form submit is a second save path (title/status/etc.):
     the canvas must be serialized into the hidden `#gdlr-json` input
     (`body_json`) — this is a direct port of the plugin's
     `pb_update_tempalte_val` walker (same attribute names, same inner-
     container selectors, same `.data('value')` reads). */
(function () {
  'use strict';

  function walker(el) {
    var $ = window.jQuery;
    var cur = $(el);
    var item = {
      template: cur.attr('data-template'),
      type: cur.attr('data-type'),
    };
    if (cur.attr('data-column')) item.column = cur.attr('data-column');
    if (cur.attr('data-sync-template')) item['sync-template'] = cur.attr('data-sync-template');
    var v = cur.data('value');
    if (v !== undefined && v !== null) item.value = v;

    if (item.template === 'wrapper') {
      var inner;
      if (item.type === 'background') {
        inner = cur.children('.gdlr-core-page-builder-wrapper-content')
          .children('.gdlr-core-page-builder-wrapper-content-margin')
          .children('.gdlr-core-page-builder-wrapper-content-wrap')
          .children()
          .children('.gdlr-core-page-builder-wrapper-container');
      } else if (item.type === 'sidebar') {
        inner = cur.children()
          .children('.gdlr-core-page-builder-wrapper-sidebar-container')
          .children('.gdlr-core-page-builder-wrapper-sidebar-content')
          .children();
      } else if (item.type === 'flexbox') {
        inner = cur.children('.gdlr-core-page-builder-flexbox-content')
          .children('.gdlr-core-page-builder-flexbox-content-margin')
          .children('.gdlr-core-page-builder-flexbox-content-wrap')
          .children();
      } else {
        inner = cur.children('.gdlr-core-page-builder-column-content')
          .children('.gdlr-core-page-builder-column-content-margin')
          .children('.gdlr-core-page-builder-column-content-wrap')
          .children('.gdlr-core-page-builder-column-container');
      }
      inner.children('[data-template]').each(function () {
        (item.items = item.items || []).push(walker(this));
      });
    }
    return item;
  }

  function serializeCanvas() {
    var container = document.getElementById('gdlr-core-page-builder-container');
    if (!container || !window.jQuery) return '[]';
    var out = [];
    /* Root is `#gdlr-core-page-builder-container` children with data-template
       (mirrors the plugin's own `pb_update`), not the body div. */
    Array.prototype.forEach.call(
      container.querySelectorAll(':scope > [data-template]'),
      function (n) { out.push(walker(n)); }
    );
    return JSON.stringify(out);
  }

  /* Init watchdog: page-builder.js registers `new gdlrCorePageBuilder()` as a
     jQuery document-ready handler. With rpress's inline script placement the
     ready promise can resolve before/around the bind, leaving the handler
     registered but never invoked (no canvas bindings = dead UI). On window
     load, if the Save button has no click handler yet, run the registered
     ready handlers ourselves — identical effect to jQuery firing them. */
  function ensurePbInit() {
    try {
      var update = document.querySelector('.gdlr-core-page-builder-head-nav-update');
      if (!update) return;
      var bound = window.jQuery._data(update, 'events');
      if (bound && bound.click) return;
      var ready = (window.jQuery._data(document, 'events') || {}).ready || [];
      ready.forEach(function (h) {
        try { h.handler.call(document, window.jQuery); } catch (e) {}
      });
    } catch (e) {}
  }
  window.addEventListener('load', ensurePbInit);
  function hookForm() {
    var hidden = document.getElementById('gdlr-json');
    if (!hidden) return;
    var form = hidden.closest ? hidden.closest('form') : null;
    if (!form || form.__rpressPbHooked) return;
    form.__rpressPbHooked = true;
    form.addEventListener('submit', function () {
      try { hidden.value = serializeCanvas(); } catch (e) { /* keep classic submit */ }
    }, true);
  }

  document.addEventListener('DOMContentLoaded', hookForm);
})();
