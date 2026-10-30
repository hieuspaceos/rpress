/* ATRIUM.js — Ferris the press crab + dock helpers.
   First-party, additive only: reads no secrets, makes no network calls.
   Singleton-guarded: htmx fragment swaps re-execute page scripts. */
(function () {
  if (window.__atriumInit) return;
  window.__atriumInit = true;

  var reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var mobileQ = window.matchMedia('(max-width: 782px)');
  var touchQ = window.matchMedia('(hover: none)');

  /* ---------------- dock helpers ---------------- */

  var menu = document.getElementById('adminmenu');

  // "+" tile (repurposed collapse button) → new post
  var collapseBtn = document.getElementById('collapse-button');
  if (collapseBtn) {
    collapseBtn.addEventListener('click', function (e) {
      e.preventDefault();
      e.stopPropagation();
      window.location.href = '/admin/new/post';
    }, true);
  }

  // "More" tile on the mobile tab bar — items past index 4 tuck behind it.
  var moreLi = null;
  function applyMore() {
    if (!menu) return;
    var items = Array.prototype.filter.call(menu.children, function (li) {
      return li.id !== 'collapse-menu' && !li.classList.contains('more-tile');
    });
    if (mobileQ.matches && items.length > 5) {
      menu.classList.add('more-mode');
      if (!moreLi) {
        moreLi = document.createElement('li');
        moreLi.className = 'more-tile menu-top';
        moreLi.innerHTML =
          '<a class="menu-top" href="#"><div class="wp-menu-image dashicons-before dashicons-ellipsis"><br></div>' +
          '<div class="wp-menu-name">More</div></a>';
        moreLi.addEventListener('click', function (e) {
          e.preventDefault();
          menu.classList.toggle('show-all');
        });
      }
      if (!moreLi.parentNode) {
        // insert as the 5th slot — hidden items follow it
        var after = menu.children[4] ? menu.children[4].nextSibling : null;
        menu.insertBefore(moreLi, after);
      }
    } else {
      menu.classList.remove('more-mode');
      menu.classList.remove('show-all');
      if (moreLi && moreLi.parentNode) moreLi.parentNode.removeChild(moreLi);
    }
  }
  applyMore();
  if (mobileQ.addEventListener) mobileQ.addEventListener('change', applyMore);

  /* current submenu link: the server only marks the top-level tile, so
     mirror WP and pin `li.current` on the flyout item matching the path */
  if (menu) {
    var path = location.pathname.replace(/\/+$/, '');
    Array.prototype.forEach.call(menu.querySelectorAll('.wp-submenu a'), function (a) {
      var href = (a.getAttribute('href') || '').replace(/\/+$/, '');
      if (href && href === path) {
        a.parentNode.classList.add('current');
        a.setAttribute('aria-current', 'page');
      }
    });
  }

  // touch: first tap on a parent tile opens its flyout, second navigates
  if (touchQ.matches && menu) {
    menu.addEventListener('click', function (e) {
      var li = e.target.closest('li.wp-has-submenu');
      if (!li || !li.parentNode || li.parentNode.id !== 'adminmenu') return;
      if (!li.classList.contains('opensub')) {
        e.preventDefault();
        Array.prototype.forEach.call(menu.querySelectorAll('li.opensub'), function (o) {
          o.classList.remove('opensub');
        });
        li.classList.add('opensub');
      }
    }, false);
    document.addEventListener('click', function (e) {
      if (!e.target.closest('#adminmenu')) {
        Array.prototype.forEach.call(menu.querySelectorAll('li.opensub'), function (o) {
          o.classList.remove('opensub');
        });
      }
    }, false);
  }

  /* ---------------- Ferris — Tom & Jerry ---------------- */
  if (reduced || !document.body) return; // absent under reduced-motion

  var ferris = document.createElement('div');
  ferris.id = 'ferris';
  ferris.title = 'Ferris, the press crab';
  ferris.innerHTML =
    '<span class="say" role="status"></span>' +
    '<span class="tag">ferris · press shift</span>' +
    '<svg viewBox="0 0 64 52" fill="none"><g class="inner">' +
    '<g stroke="oklch(48% 0.17 36)" stroke-width="3" stroke-linecap="round">' +
    '<path class="leg l1" d="M20 34 L10 40"/><path class="leg l2" d="M24 38 L16 46"/>' +
    '<path class="leg l3" d="M44 34 L54 40"/><path class="leg l4" d="M40 38 L48 46"/></g>' +
    '<path d="M14 26 Q14 12 32 12 Q50 12 50 26 Q50 36 32 36 Q14 36 14 26 Z" fill="oklch(60% 0.2 38)"/>' +
    '<path d="M18 22 Q18 15 32 15" stroke="oklch(70% 0.18 42)" stroke-width="2.5" stroke-linecap="round" fill="none" opacity=".8"/>' +
    '<g class="eye" style="transform-origin: 26px 14px">' +
    '<rect x="24" y="6" width="3" height="8" rx="1.5" fill="oklch(48% 0.17 36)"/>' +
    '<circle cx="25.5" cy="6" r="3" fill="oklch(24% 0.012 60)"/></g>' +
    '<g class="eye" style="transform-origin: 38px 14px; animation-delay:.1s">' +
    '<rect x="37" y="6" width="3" height="8" rx="1.5" fill="oklch(48% 0.17 36)"/>' +
    '<circle cx="38.5" cy="6" r="3" fill="oklch(24% 0.012 60)"/></g>' +
    '<g class="claw"><path d="M14 24 L4 18" stroke="oklch(48% 0.17 36)" stroke-width="3" stroke-linecap="round"/>' +
    '<path d="M2 12 Q-2 18 3 22 Q8 26 11 21 Q13 17 9 14 Q5 11 2 12 Z" fill="oklch(55% 0.19 38)"/></g>' +
    '<g class="claw" style="transform-origin: 70% 70%">' +
    '<path d="M50 24 L60 18" stroke="oklch(48% 0.17 36)" stroke-width="3" stroke-linecap="round"/>' +
    '<path d="M62 12 Q66 18 61 22 Q56 26 53 21 Q51 17 55 14 Q59 11 62 12 Z" fill="oklch(55% 0.19 38)"/></g>' +
    '<rect x="24" y="28" width="16" height="5" rx="1" fill="oklch(48% 0.17 36)" opacity=".9"/>' +
    '<path d="M26 30.5h12" stroke="oklch(96% 0.016 88)" stroke-width="1.4" stroke-linecap="round"/>' +
    '</g></svg>';
  document.body.appendChild(ferris);

  var say = ferris.querySelector('.say');
  var inner = ferris.querySelector('.inner');

  var W = 64, H = 52;
  var x = innerWidth + 90, y = innerHeight - 170;
  var dir = 1, mode = 'enter';
  var tx = x, ty = y, speed = 0.55;
  var modeUntil = 0, lastCx = 0, lastCy = 0;
  var vx = 0, vy = 0;
  var rand = function (a, b) { return a + Math.random() * (b - a); };
  var clamp = function (v, a, b) { return Math.min(b, Math.max(a, v)); };

  ferris.style.transform = 'translate(' + (innerWidth + 90) + 'px, ' + y + 'px)';

  function puff(px, py, n, spread) {
    spread = spread || 26;
    for (var i = 0; i < (n || 5); i++) {
      var el = document.createElement('i');
      el.className = 'puff';
      el.style.left = px + rand(-8, 8) + 'px';
      el.style.top = py + rand(-6, 6) + 'px';
      el.style.setProperty('--dx', rand(-spread, spread) + 'px');
      el.style.setProperty('--dy', rand(-spread, -10) + 'px');
      document.body.appendChild(el);
      (function (el) { setTimeout(function () { el.remove(); }, 600); })(el);
    }
  }
  var streakCooldown = 0;
  function streak(px, py, fly, tilt) {
    if (performance.now() < streakCooldown) return;
    streakCooldown = performance.now() + 60;
    var el = document.createElement('i');
    el.className = 'streak';
    el.style.left = px + 'px';
    el.style.top = py + rand(-4, 4) + 'px';
    el.style.rotate = (tilt || 0) + 'deg';
    el.style.setProperty('--fly', fly + 'px');
    document.body.appendChild(el);
    setTimeout(function () { el.remove(); }, 450);
  }

  function edgeSpot() {
    var m = 90;
    var spots = [
      [rand(innerWidth * .55, innerWidth * .9), rand(80, innerHeight * .3)],
      [rand(m, innerWidth * .2), rand(90, innerHeight * .4)],
      [rand(m, innerWidth * .3), rand(innerHeight * .45, innerHeight * .75)],
      [rand(innerWidth * .4, innerWidth * .85), rand(innerHeight * .65, innerHeight * .82)],
      [rand(innerWidth * .6, innerWidth - m), rand(innerHeight * .2, innerHeight * .55)]
    ];
    return spots[Math.floor(Math.random() * spots.length)];
  }
  function headTo(nx, ny, s) {
    tx = clamp(nx, 8, innerWidth - W - 8);
    ty = clamp(ny, 40, innerHeight - H - 8);
    speed = s || 0.55;
  }
  function face(dx) {
    if (Math.abs(dx) > 0.5) dir = dx < 0 ? -1 : 1;
    inner.style.transform = 'scaleX(' + dir + ')';
  }

  var cx = -999, cy = -999, moveAt = 0;
  addEventListener('pointermove', function (e) {
    cx = e.clientX; cy = e.clientY;
    if (Math.hypot(cx - lastCx, cy - lastCy) > 12) {
      lastCx = cx; lastCy = cy; moveAt = performance.now();
    }
  }, { passive: true });

  var last = performance.now();
  function tick(now) {
    if (!ferris.isConnected) return; // htmx swap removed the node — stop the loop
    x = clamp(x, -90, innerWidth + 90); y = clamp(y, 40, innerHeight + H);
    if (mobileQ.matches) { requestAnimationFrame(tick); return; } // tab bar owns the space
    var dt = Math.min(50, now - last); last = now;
    var dcx = (x + W / 2) - cx, dcy = (y + H / 2) - cy;
    var distCursor = Math.hypot(dcx, dcy);
    var dx = tx - x, dy = ty - y, dist = Math.hypot(dx, dy);

    if (mode === 'enter') {
      headTo(innerWidth * 0.82, innerHeight * 0.16, 1.2);
      if (dist < 40) {
        mode = 'wander'; modeUntil = now + rand(800, 1600);
        var s0 = edgeSpot(); headTo(s0[0], s0[1]);
      }
    } else if (mode === 'wander' || mode === 'wait') {
      if (distCursor < 80) {
        mode = 'dodge'; modeUntil = now + 380;
        var away = Math.atan2(dcy, dcx);
        vx = Math.cos(away) * 1.9; vy = Math.sin(away) * 1.9;
        face(Math.cos(away));
        puff(x + W / 2, y + H, 6);
        streak(x + W / 2, y + H / 2, -dir * 70);
      } else if (cx > 0 && now - moveAt > 900 && distCursor < 500) {
        mode = 'rev'; modeUntil = now + 480;
        ferris.classList.add('rev');
        inner.style.transform = 'scaleX(' + dir + ') scaleY(.82)';
        puff(x + W / 2, y + H, 3, 16);
      } else if (mode === 'wander' && dist < 24) {
        mode = 'wait'; modeUntil = now + rand(600, 2000);
      } else if (mode === 'wait' && now > modeUntil) {
        mode = 'wander';
        var s1 = edgeSpot(); headTo(s1[0], s1[1]);
      }
    } else if (mode === 'rev') {
      if (distCursor > 560 || now - moveAt < 250) {
        ferris.classList.remove('rev');
        inner.style.transform = 'scaleX(' + dir + ')';
        mode = 'wander';
        var s2 = edgeSpot(); headTo(s2[0], s2[1]);
      } else if (now > modeUntil) {
        ferris.classList.remove('rev');
        inner.style.transform = 'scaleX(' + dir + ')';
        mode = 'charge'; modeUntil = now + 1400;
        face(-dcx);
        puff(x + W / 2, y + H, 8);
        var aim = Math.atan2(-dcy, -dcx);
        vx = Math.cos(aim) * 2.6; vy = Math.sin(aim) * 2.6;
      }
    } else if (mode === 'charge') {
      if (distCursor < 46 || now > modeUntil) {
        mode = 'wander'; modeUntil = now + 700;
        var s3 = edgeSpot(); headTo(s3[0], s3[1], 0.5);
        puff(x + W / 2, y + H, 4);
      }
    } else if (mode === 'dodge') {
      if (now > modeUntil) {
        mode = 'wander';
        var s4 = edgeSpot(); headTo(s4[0], s4[1]);
      }
    }

    if (mode === 'charge' || mode === 'dodge') {
      x = clamp(x + vx * dt, 8, innerWidth - W - 8);
      y = clamp(y + vy * dt, 40, innerHeight - H - 8);
      streak(x + W / 2 - vx * 8, y + H / 2, -vx * 46, Math.atan2(vy, vx) * 180 / Math.PI);
      if (Math.random() < .35) puff(x + W / 2, y + H, 1, 14);
    } else {
      var step = speed * dt;
      if (dist > 0.5) {
        var k = Math.min(1, step / dist);
        x += dx * k; y += dy * k;
        face(dx);
      }
    }

    ferris.style.transform = 'translate(' + x + 'px, ' + y + 'px)';
    requestAnimationFrame(tick);
  }
  requestAnimationFrame(tick);

  var lines = [
    'fresh off the press. <span class="em">*</span>',
    'queries: zero. <span class="em">claws: two.</span>',
    'static files only, pal.',
    'I do the publishing around here.',
    'print it. ship it. <span class="em">done.</span>',
    'no PHP was harmed.',
    'crab on duty — keep it brief.',
    'the cache is warm today.',
    'wp-admin? never heard of it.',
    'stamp it <span class="em">harder.</span>',
    'can\'t catch me. <span class="em">try again.</span>'
  ];
  var sayTimer;
  ferris.addEventListener('click', function () {
    say.innerHTML = lines[Math.floor(Math.random() * lines.length)];
    say.classList.add('show');
    clearTimeout(sayTimer);
    sayTimer = setTimeout(function () { say.classList.remove('show'); }, 2400);
  });
})();
