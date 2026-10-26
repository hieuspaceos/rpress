#!/usr/bin/env python3
"""Scope frontend stylesheets to `.gdlr-core-pb-livemode` so the page-builder's
Live Mode canvas renders like the public site without leaking theme rules
into wp-admin chrome.

Usage: scope-css.py <in.css> <out.css>   (run from cms/ dir)
Regenerate: for each file in templates/front-css/, write <name>.scoped.css.
"""
import re
import sys

SCOPE = ".gdlr-core-pb-livemode"


def strip_comments(css: str) -> str:
    return re.sub(r"/\*.*?\*/", "", css, flags=re.S)


def scope_selector(sel: str) -> str:
    sel = sel.strip()
    if not sel:
        return sel
    low = sel.lower()
    if low == "html":
        return SCOPE
    for p in ("html ", "body ", "body.", "body#", "body:", "body["):
        if low.startswith(p):
            return SCOPE + sel[len(p) - len(p) + (len(p) - len(p)) :] if False else SCOPE + sel[len(p) - 1 :]
    if low == "body" or low.startswith("body "):
        return SCOPE + sel[4:]
    if low.startswith("body"):
        return SCOPE + sel[4:]
    if low in (":root",):
        return SCOPE
    return f"{SCOPE} {sel}"


def scope_block(css: str, i: int) -> tuple:
    """Parse a rule starting at i; return (text, next_i)."""
    # find next '{' or ';' or '}'
    m = re.search(r"[{};]", css[i:])
    if not m:
        return css[i:], len(css)
    j = i + m.start()
    head = css[i:j].strip()
    ch = css[j]
    if ch == ";":
        # at-rule like @import — keep as-is
        return css[i : j + 1], j + 1
    if ch == "}":
        return css[i:j], j
    # ch == '{'
    # find matching close
    depth = 0
    k = j
    while k < len(css):
        if css[k] == "{":
            depth += 1
        elif css[k] == "}":
            depth -= 1
            if depth == 0:
                break
        k += 1
    inner = css[j + 1 : k]
    end = k + 1
    if head.startswith("@"):
        name = head[1:].split()[0].lower() if head[1:].split() else ""
        if name in ("media", "supports", "layer"):
            scoped = scope_css(inner)
            return f"{head}{{{scoped}}}", end
        if name in ("font-face", "page", "property", "viewport"):
            return css[i:end], end
        if name in ("keyframes", "-webkit-keyframes", "-moz-keyframes"):
            return css[i:end], end
        if name in ("charset", "import", "namespace"):
            return "", end
        # unknown at-rule: keep verbatim
        return css[i:end], end
    # style rule
    sels = [scope_selector(s) for s in head.split(",")]
    return ",\n".join(sels) + "{" + inner + "}", end


def scope_css(css: str) -> str:
    css = strip_comments(css)
    out = []
    i = 0
    n = len(css)
    while i < n:
        m = re.search(r"[{};]", css[i:])
        if not m:
            tail = css[i:].strip()
            if tail:
                out.append(tail)
            break
        text, i = scope_block(css, i)
        if text.strip():
            out.append(text)
    return "\n".join(out)


if __name__ == "__main__":
    src, dst = sys.argv[1], sys.argv[2]
    scoped = scope_css(open(src, encoding="utf-8").read())
    open(dst, "w", encoding="utf-8").write(scoped)
    print(f"{dst}: {len(scoped)} bytes")
