/* rpress page-builder shim: the plugin JS expects WordPress core scripts
   (jquery, jquery-ui sortable/datepicker/dialog, jquery-transit) already
   loaded. rpress loads jquery-ui.min.js and color-picker/iris separately;
   this shim only fills the small gaps the plugin code touches:

   - `$.fn.sortable`/`datepicker`/`dialog` come from jquery-ui.min.js (loaded
     before this file); nothing to do.
   - jQuery Transit is bundled at the end of the plugin's page-builder.js,
     so it self-registers; nothing to do.
   - `window.page_builder_var` is emitted inline by rpress (wp_localize_script
     equivalent) before the plugin scripts run; nothing to do.

   What IS needed: the plugin's `$(document).on('ready', ...)` handlers run
   immediately when the script tag is inline after the metabox HTML — fine —
   but `$(window).on('load')` handlers that reference optional globals
   (YoastSEO, tinymce) must not throw before the optional checks run. Guard
   the two globals the bundled code dereferences without typeof checks:

   - `window.YoastSEO` : plugin calls YoastSEO.app.registerPlugin inside a
     `typeof YoastSEO != 'undefined'` guard — no shim needed.
   - `tinymce`/`tinyMCEPreInit` : html-option.js guards with typeof — no
     shim needed.

   So this file intentionally does almost nothing: it defines `send_to_editor`
   (WP core global) which html-option.js calls WITHOUT a typeof guard when the
   "Add Media" button inserts into a textarea, and stubs `QTags._buttonsInit`
   shape so a stray call can't crash (quicktags itself is only invoked under
   `typeof quicktags != 'undefined'` guards). */
(function () {
  'use strict';

  if (typeof window.send_to_editor !== 'function') {
    /* Minimal WP core behaviour: insert raw HTML/string at caret in the
       active wp editor textarea (rpress tracks it via window.wpActiveEditor). */
    window.send_to_editor = function (html) {
      var id = window.wpActiveEditor;
      var ta = id ? document.getElementById(id) : null;
      if (!ta || typeof ta.selectionStart !== 'number') return;
      var s = ta.selectionStart, e = ta.selectionEnd;
      var text = typeof html === 'string' ? html : String(html);
      ta.value = ta.value.slice(0, s) + text + ta.value.slice(e);
      ta.selectionStart = ta.selectionEnd = s + text.length;
      ta.focus();
    };
  }

  if (typeof window.QTags === 'undefined') {
    window.QTags = function () {};
    window.QTags._buttonsInit = function () {};
  }
})();
