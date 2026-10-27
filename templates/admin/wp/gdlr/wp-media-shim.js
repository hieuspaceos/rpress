/* rpress page-builder shim: `wp.media` uploader.
   The plugin calls `wp.media({title, button:{text}, multiple})`, then
   `.on('select', fn)` and reads `frame.state().get('selection').first().toJSON()`
   expecting {id, url}. We can't re-create the WP media grid inside rpress, so
   the shim exposes a small attachment picker backed by rpress's media library:
   GET /api/v1/gdlr/media returns [{id, url}] (JSON); clicking a tile fires
   the frame's 'select' callback with a Backbone-free selection object. */
(function () {
  'use strict';

  function Fire(on) {
    return {
      on: function (ev, cb) { on[ev] = on[ev] || []; on[ev].push(cb); return this; },
      fire: function (ev, arg) { (on[ev] || []).forEach(function (cb) { cb(arg); }); },
    };
  }

  function el(tag, cls, html) {
    var d = document.createElement(tag);
    if (cls) d.className = cls;
    if (html != null) d.innerHTML = html;
    return d;
  }

  function openPicker(frame, handlers) {
    var ov = el('div', 'rpress-media-overlay');
    ov.style.cssText =
      'position:fixed;inset:0;background:rgba(0,0,0,.55);z-index:160001;' +
      'display:flex;align-items:center;justify-content:center;';
    var box = el('div', 'rpress-media-box');
    box.style.cssText =
      'background:#fff;width:720px;max-height:80vh;display:flex;flex-direction:column;' +
      'box-shadow:0 5px 30px rgba(0,0,0,.4);';
    var head = el('div', '', '<span style="font-weight:600">' +
      (frame.title || 'Select image') + '</span>');
    head.style.cssText = 'padding:12px 16px;border-bottom:1px solid #ddd;display:flex;justify-content:space-between;';
    var x = el('span', '', '&times;');
    x.style.cssText = 'cursor:pointer;font-size:20px;line-height:1;';
    x.onclick = function () { document.body.removeChild(ov); };
    head.appendChild(x);
    var grid = el('div');
    grid.style.cssText =
      'padding:14px;overflow:auto;display:grid;grid-template-columns:repeat(auto-fill,110px);gap:10px;';
    var urlRow = el('div');
    urlRow.style.cssText = 'padding:12px 16px;border-top:1px solid #ddd;display:flex;gap:8px;';
    var inp = el('input');
    inp.placeholder = '…or paste image URL';
    inp.style.cssText = 'flex:1;padding:6px;';
    var btn = el('button', 'button', 'Select');
    btn.style.cssText = 'padding:6px 14px;';
    function pick(att) {
      var sel = { first: function () { return { toJSON: function () { return att; } }; } };
      handlers.fire('select', sel);
      document.body.removeChild(ov);
    }
    btn.onclick = function () {
      if (!inp.value.trim()) return;
      pick({ id: 0, url: inp.value.trim() });
    };
    urlRow.appendChild(inp); urlRow.appendChild(btn);
    box.appendChild(head); box.appendChild(grid); box.appendChild(urlRow);
    ov.appendChild(box);
    document.body.appendChild(ov);

    fetch('/api/v1/gdlr/media', { credentials: 'same-origin' })
      .then(function (r) { return r.ok ? r.json() : []; })
      .catch(function () { return []; })
      .then(function (list) {
        (list || []).forEach(function (m) {
          var c = el('div');
          c.style.cssText =
            'width:110px;height:110px;background:#f0f0f1 center/cover no-repeat;' +
            'cursor:pointer;border:2px solid transparent;';
          c.style.backgroundImage = 'url(' + (m.url || '') + ')';
          c.title = m.url || '';
          c.onmouseover = function () { c.style.borderColor = '#0073aa'; };
          c.onmouseout = function () { c.style.borderColor = 'transparent'; };
          c.onclick = function () { pick(m); };
          grid.appendChild(c);
        });
        if (!(list || []).length) {
          grid.appendChild(el('div', '', 'No media yet — upload via Media, or paste a URL below.'));
          grid.firstChild.style.cssText = 'grid-column:1/-1;color:#666;padding:20px;';
        }
      });
  }

  if (typeof window.wp !== 'object' || window.wp === null) window.wp = {};
  window.wp.media = function (opts) {
    var handlers = Fire({});
    var frame = {
      title: opts && opts.title,
      open: function () { openPicker(frame, handlers); return frame; },
      on: function (ev, cb) { handlers.on(ev, cb); return frame; },
      state: function () { return { get: function () { return null; } }; },
    };
    return frame;
  };
})();
