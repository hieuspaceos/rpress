/* Menu editor (nav-menus.php) — vendored-style, no dependencies.
 *
 * Behaviors (progressive enhancement; every action has a POST fallback):
 *  - .item-edit expands/collapses the item settings block.
 *  - Move buttons (data-dir: up|down|under|out|top) rewrite the hidden
 *    ord-{id}/parent-{id} inputs and re-sort the DOM — mirroring
 *  - Remove marks del-{id}=1 and hides the block; deletion commits on Save.
 *  - Select All toggles the metabox checkboxes.
 *  - Inner tabs (Most Recent | View All | Search) switch .tabs-panel panels;
 *    the Search input filters its panel's list client-side.
 *  - Screen Options: the tab toggles the prefs panel; .hide-field-tog boxes
 *    flip every `.field-{value}` block live and .hide-acc-tog flips the
 *    `.add-{value}` accordion section (server persists on Apply).
 *  - Any field change sets a dirty flag → beforeunload guard.
 */
(function () {
	'use strict';

	var dirty = false;

	function markDirty() { dirty = true; }

	function items() {
		return Array.prototype.slice.call(
			document.querySelectorAll('#menu-to-edit > li.menu-item')
		);
	}

	function idOf(li) { return li.getAttribute('data-item-id'); }

	function hid(li, name) {
		return li.querySelector('input[name="' + name + '-' + idOf(li) + '"]');
	}

	function depthOf(li) {
		var m = /menu-item-depth-(\d+)/.exec(li.className || '');
		return m ? parseInt(m[1], 10) : 0;
	}

	function setDepth(li, d) {
		li.className = li.className.replace(/menu-item-depth-\d+/, 'menu-item-depth-' + d);
		var title = li.querySelector('.item-title');
		if (!title) { return; }
		var badge = title.querySelector('.is-submenu');
		if (d > 0) {
			if (!badge) {
				badge = document.createElement('span');
				badge.className = 'is-submenu';
				badge.appendChild(document.createTextNode('sub item'));
				title.appendChild(badge);
			}
		} else if (badge) {
			badge.parentNode.removeChild(badge);
		}
	}

	// li plus the descendant blocks that follow it (deeper depth, DFS order).
	function blockOf(li) {
		var all = items();
		var i = all.indexOf(li);
		var d = depthOf(li);
		var out = [li];
		for (var j = i + 1; j < all.length; j++) {
			if (depthOf(all[j]) > d) { out.push(all[j]); } else { break; }
		}
		return out;
	}

	function titleOf(li) {
		var t = li && li.querySelector('.menu-item-title');
		return t ? (t.textContent || '').replace(/^\s+|\s+$/g, '') : '';
	}

	function byId(id) { return document.getElementById('menu-item-' + id); }

	function parentOf(li) {
		var p = hid(li, 'parent');
		return p ? parseInt(p.value, 10) || 0 : 0;
	}

	// Renumber ord inputs to match DOM order (1..n) and refresh move links.
	function refresh() {
		var all = items();
		all.forEach(function (li, i) {
			var ord = hid(li, 'ord');
			if (ord) { ord.value = String(i + 1); }
			var pid = parentOf(li);
			var prev = all[i - 1] || null;
			var set = function (dir, show, label) {
				var b = li.querySelector('.field-move [data-dir="' + dir + '"]');
				if (!b) { return; }
				b.hidden = !show;
				if (label != null) { b.textContent = label; }
			};
			set('up', i > 0);
			set('down', i < all.length - 1);
			set('under', !!prev, 'Under ' + (prev ? titleOf(prev) : ''));
			var par = pid ? byId(pid) : null;
			set('out', pid > 0, 'Out from under ' + titleOf(par));
			set('top', i > 0);
		});
	}

	function shiftDepth(li, delta) {
		if (!delta) { return; }
		blockOf(li).forEach(function (n) { setDepth(n, depthOf(n) + delta); });
	}

	function move(li, dir) {
		var list = document.getElementById('menu-to-edit');
		var all = items();
		var i = all.indexOf(li);
		var id = idOf(li);
		switch (dir) {
		case 'up':
			if (li.previousElementSibling) {
				list.insertBefore(li, li.previousElementSibling);
			}
			break;
		case 'down':
			if (li.nextElementSibling) {
				list.insertBefore(li.nextElementSibling, li);
			}
			break;
		case 'top': {
			if (i <= 0) { break; }
			var old = depthOf(li);
			var blk = blockOf(li);
			// First element that is not part of the moving block.
			var first = all.filter(function (n) { return blk.indexOf(n) === -1; })[0];
			blk.forEach(function (n) { list.insertBefore(n, first); });
			hid(li, 'parent').value = '0';
			shiftDepth(li, -old);
			break;
		}
		case 'under': {
			var prev = all[i - 1];
			if (!prev) { break; }
			hid(li, 'parent').value = idOf(prev);
			shiftDepth(li, depthOf(prev) + 1 - depthOf(li));
			break;
		}
		case 'out': {
			var pid = parentOf(li);
			var par = pid ? byId(pid) : null;
			if (!par) { break; }
			var gp = parentOf(par);
			var pd = depthOf(par);
			// Land as sibling of parent, right after its subtree.
			var anchor = blockOf(par);
			var last = anchor[anchor.length - 1];
			blockOf(li).forEach(function (n) {
				list.insertBefore(n, last.nextSibling);
				last = n;
			});
			hid(li, 'parent').value = String(gp);
			shiftDepth(li, pd - depthOf(li));
			break;
		}
		default:
			return;
		}
		refresh();
		markDirty();
	}

	function closest(el, sel) {
		while (el && el !== document) {
			if (el.matches && el.matches(sel)) { return el; }
			el = el.parentNode;
		}
		return null;
	}

	document.addEventListener('click', function (e) {
		// Accordion metabox open/close.
		var hnd = closest(e.target, '.accordion-section-title');
		if (hnd) {
			var sec = closest(hnd, '.accordion-section');
			if (sec) {
				var open = sec.className.indexOf('open') !== -1;
				sec.className = open ? sec.className.replace(' open', '') : sec.className + ' open';
				var trig = sec.querySelector('.accordion-trigger');
				if (trig) trig.setAttribute('aria-expanded', open ? 'false' : 'true');
			}
			return;
		}
		var t = e.target;

		// Expand / collapse item settings.
		var edit = closest(t, 'a.item-edit');
		if (edit) {
			e.preventDefault();
			var li0 = closest(edit, 'li.menu-item');
			if (li0) {
				var open = li0.className.indexOf('menu-item-edit-active') !== -1;
				li0.className = li0.className.replace(
					open ? 'menu-item-edit-active' : 'menu-item-edit-inactive',
					open ? 'menu-item-edit-inactive' : 'menu-item-edit-active'
				);
			}
			return;
		}

		// Cancel collapses without saving field edits (server still holds old).
		var cancel = closest(t, 'a.item-cancel');
		if (cancel) {
			e.preventDefault();
			var li1 = closest(cancel, 'li.menu-item');
			if (li1) {
				li1.className = li1.className.replace('menu-item-edit-active', 'menu-item-edit-inactive');
			}
			return;
		}

		// Remove: mark deleted + hide; commits on Save Menu.
		var del = closest(t, '.item-delete');
		if (del) {
			e.preventDefault();
			var li2 = closest(del, 'li.menu-item');
			if (li2) {
				var dh = hid(li2, 'del');
				if (dh) { dh.value = '1'; }
				li2.style.display = 'none';
				refresh();
				markDirty();
			}
			return;
		}

		// Move links → client-side reorder (no POST).
		var mv = closest(t, '.field-move [data-dir]');
		if (mv) {
			e.preventDefault();
			var li3 = closest(mv, 'li.menu-item');
			if (li3) { move(li3, mv.getAttribute('data-dir')); }
			return;
		}

		// Select All inside a metabox.
		var selAll = closest(t, 'a.select-all');
		if (selAll) {
			e.preventDefault();
			var box = closest(selAll, '.inside');
			if (box) {
				box.querySelectorAll('input.menu-item-checkbox').forEach(function (c) {
					c.checked = true;
				});
			}
			return;
		}

		// Inner metabox tabs (Most Recent | View All | Search).
		var tab = closest(t, 'a.nav-tab-link');
		if (tab) {
			e.preventDefault();
			var div = closest(tab, '.posttypediv');
			if (!div) { return; }
			div.querySelectorAll('.tabs-panel').forEach(function (p) {
				var on = p.id === tab.getAttribute('data-type');
				p.className = p.className.replace(
					on ? 'tabs-panel-inactive' : 'tabs-panel-active',
					on ? 'tabs-panel-active' : 'tabs-panel-inactive'
				);
			});
			div.querySelectorAll('.add-menu-item-tabs > li').forEach(function (n) {
				n.className = n.className.replace(/\s*\btabs\b/, '');
			});
			var tabLi = tab.parentNode;
			if (tabLi.className.indexOf('tabs') === -1) {
				tabLi.className = (tabLi.className + ' tabs').replace(/^\s+/, '');
			}
			return;
		}

		// Screen Options tab toggle (WP screenMeta equivalent).
		if (t.id === 'show-settings-link') {
			e.preventDefault();
			var wrap = document.getElementById('screen-options-wrap');
			if (wrap) {
				var hid2 = wrap.classList.toggle('hidden');
				t.setAttribute('aria-expanded', hid2 ? 'false' : 'true');
			}
			return;
		}
	});

	// Quick Search: filter the panel's checklist by data-title.
	document.addEventListener('input', function (e) {
		var t = e.target;
		if (!t.classList || !t.classList.contains('quick-search')) { return; }
		var panel = closest(t, '.tabs-panel');
		if (!panel) { return; }
		var v = (t.value || '').toLowerCase();
		panel.querySelectorAll('li[data-title]').forEach(function (li) {
			li.style.display = li.getAttribute('data-title').indexOf(v) !== -1 ? '' : 'none';
		});
	});

	// Dirty flag → beforeunload. Any edit inside the save form counts.
	document.addEventListener('input', function (e) {
		if (closest(e.target, '#update-nav-menu')) { markDirty(); }
	});
	document.addEventListener('change', function (e) {
		if (closest(e.target, '#update-nav-menu')) { markDirty(); }
	});

	// Screen Options toggles — flip fields/accordion sections live; the Apply
	// POST persists the choice in settings.menus_screen_{user}.
	document.addEventListener('change', function (e) {
		var t = e.target;
		if (!t || !t.classList) { return; }
		var sel = null;
		if (t.classList.contains('hide-field-tog')) {
			sel = '.field-' + t.value;
		} else if (t.classList.contains('hide-acc-tog')) {
			sel = '.add-' + t.value;
		}
		if (!sel) { return; }
		document.querySelectorAll(sel).forEach(function (el) {
			el.classList.toggle('hidden', !t.checked);
		});
	});
	document.addEventListener('submit', function (e) {
		// Submitting (Save/add/delete/move POST) is a deliberate navigation —
		// clear the guard so it doesn't nag on the redirect.
		if (closest(e.target, '#menu-del, #menu-move')) { dirty = false; }
	});

	window.addEventListener('beforeunload', function (e) {
		if (dirty) {
			e.preventDefault();
			e.returnValue = '';
			return '';
		}
	});


	// jQuery UI sortable — real drag reorder when the vendored build has it.
	// Horizontal intent: >28px right = nest under the item above, >28px left
	// = outdent one level; vertical position decides insert order. Falls back
	// to the Move buttons when sortable is absent.
	if (window.jQuery && typeof jQuery.fn.sortable === 'function') {
		var dragX0 = 0;
		jQuery('#menu-to-edit').sortable({
			items: '> li.menu-item',
			handle: '.menu-item-handle',
			placeholder: 'menu-item sortable-placeholder',
			tolerance: 'pointer',
			start: function (e) { dragX0 = e.clientX || 0; },
			update: function (e, ui) {
				var li = ui.item && ui.item[0];
				if (!li) { return; }
				var dx = (e.clientX || 0) - dragX0;
				var all = items();
				var i = all.indexOf(li);
				var prev = i > 0 ? all[i - 1] : null;
				if (dx > 28 && prev) {
					hid(li, 'parent').value = idOf(prev);
					shiftDepth(li, depthOf(prev) + 1 - depthOf(li));
				} else if (dx < -28) {
					var pid = parentOf(li);
					var par = pid ? byId(pid) : null;
					hid(li, 'parent').value = par ? String(parentOf(par)) : '0';
					shiftDepth(li, (par ? depthOf(par) : 0) - depthOf(li));
				}
				refresh();
				markDirty();
			}
		});
	}

	refresh();
})();

