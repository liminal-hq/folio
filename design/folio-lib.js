// Folio — pure helpers: rich-text runs, cover geometry, shape paths, page backgrounds, spread templates.
// Plain script; everything hangs off window.FOLIO_LIB. (c) Liminal HQ · Apache-2.0 OR MIT
(function () {
  const PAPER = '#fbfaf6';
  const CHAR_KEYS = ['b', 'i', 'u', 'st', 'sc', 'sup', 'sub', 'c', 'hl', 'href'];
  const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const same = (a, b) => CHAR_KEYS.every((k) => (a[k] || null) === (b[k] || null));

  // ── runs ──
  function runsOf(f) { return f.runs && f.runs.length ? f.runs : [{ t: f.text || '' }]; }
  function plain(runs) { return runs.map((r) => r.t).join(''); }
  function merge(runs) { const out = []; runs.forEach((r) => { if (!r.t) return; const l = out[out.length - 1]; if (l && same(l, r)) l.t += r.t; else out.push(Object.assign({}, r)); }); return out; }
  function lines(runs) {
    const out = [[]];
    runs.forEach((r) => { r.t.split('\n').forEach((part, i) => { if (i > 0) out.push([]); if (part) out[out.length - 1].push(Object.assign({}, r, { t: part })); }); });
    return out;
  }
  function runsToHtml(runs) {
    return runs.map((r) => {
      let h = esc(r.t).replace(/\n/g, '<br>');
      if (r.b) h = '<b>' + h + '</b>'; if (r.i) h = '<i>' + h + '</i>'; if (r.u) h = '<u>' + h + '</u>'; if (r.st) h = '<s>' + h + '</s>';
      if (r.sup) h = '<sup>' + h + '</sup>'; if (r.sub) h = '<sub>' + h + '</sub>';
      if (r.sc) h = '<span style="font-variant:small-caps">' + h + '</span>';
      if (r.c) h = '<span style="color:' + r.c + '">' + h + '</span>';
      if (r.hl) h = '<span style="background-color:' + r.hl + '">' + h + '</span>';
      if (r.href) h = '<a href="' + esc(r.href) + '">' + h + '</a>';
      return h;
    }).join('');
  }
  function domToRuns(root) {
    const out = [];
    const walk = (node, st) => {
      if (node.nodeType === 3) { out.push(Object.assign({}, st, { t: node.nodeValue.replace(/\u00a0/g, ' ') })); return; }
      if (node.nodeType !== 1) return;
      const tag = node.tagName, s = Object.assign({}, st), cs = node.style || {};
      if (tag === 'BR') { out.push({ t: '\n' }); return; }
      if ((tag === 'DIV' || tag === 'P') && out.length && !/\n$/.test(out[out.length - 1].t)) out.push({ t: '\n' });
      if (tag === 'B' || tag === 'STRONG' || parseInt(cs.fontWeight, 10) >= 600 || cs.fontWeight === 'bold') s.b = true;
      if (tag === 'I' || tag === 'EM' || cs.fontStyle === 'italic') s.i = true;
      const td = (cs.textDecoration || '') + ' ' + (cs.textDecorationLine || '');
      if (tag === 'U' || td.includes('underline')) s.u = true;
      if (tag === 'S' || tag === 'STRIKE' || tag === 'DEL' || td.includes('line-through')) s.st = true;
      if (tag === 'SUP') s.sup = true; if (tag === 'SUB') s.sub = true;
      if (cs.fontVariant === 'small-caps' || cs.fontVariantCaps === 'small-caps') s.sc = true;
      if (tag === 'FONT' && node.getAttribute('color')) s.c = node.getAttribute('color');
      if (cs.color) s.c = cs.color;
      if (cs.backgroundColor && cs.backgroundColor !== 'transparent' && cs.backgroundColor !== 'rgba(0, 0, 0, 0)') s.hl = cs.backgroundColor;
      if (tag === 'A' && node.getAttribute('href')) s.href = node.getAttribute('href');
      node.childNodes.forEach((c) => walk(c, s));
    };
    root.childNodes.forEach((c) => walk(c, {}));
    const m = merge(out);
    if (m.length) { const l = m[m.length - 1]; l.t = l.t.replace(/\n+$/, ''); }
    return merge(m);
  }
  function hasRich(runs) { return runs.some((r) => CHAR_KEYS.some((k) => r[k])); }

  // ── covers ──
  // paperback: bleed 0.125 · hardcover: wrap 0.591 + hinge 0.394 (approx — KDP template governs) · jacket: flaps 3.5 in
  function coverGeo(K, pages, paper, mode, perPage) {
    const spine = Math.max(0.06, pages * (perPage[paper] || perPage.white));
    if (mode === 'inside') return { W: 2 * K.w, H: K.h, edge: 0, spine, panels: [{ id: 'ifront', label: 'Inside front cover', x: 0, w: K.w }, { id: 'iback', label: 'Inside back cover', x: K.w, w: K.w }] };
    const hard = mode === 'hardcover', jacket = mode === 'jacket', edge = hard ? 0.591 : 0.125, hinge = hard ? 0.394 : 0, flap = jacket ? 3.5 : 0;
    const panels = []; let x = 0;
    if (flap) { panels.push({ id: 'bflap', label: 'Back flap', x, w: flap }); x += flap; }
    panels.push({ id: 'back', label: 'Back cover', x, w: K.w + hinge }); x += K.w + hinge;
    panels.push({ id: 'spine', label: 'Spine', x, w: spine }); x += spine;
    panels.push({ id: 'front', label: 'Front cover', x, w: K.w + hinge }); x += K.w + hinge;
    if (flap) { panels.push({ id: 'fflap', label: 'Front flap', x, w: flap }); x += flap; }
    const back = panels.find((p) => p.id === 'back');
    return { W: x, H: K.h, edge, hinge, spine, panels, spineText: pages >= 79, barcode: { x: back.x + back.w - hinge - 0.25 - 2, y: K.h - 0.25 - 1.2, w: 2, h: 1.2 } };
  }
  function coverFrames(K, kind, title) {
    const w = K.w, h = K.h, T = (id, panel, x, y, fw, fh, s, text, more) => Object.assign({ id, t: 'text', panel, x, y, w: fw, h: fh, s, text }, more || {});
    const I = (id, panel, x, y, fw, fh, tint, label, more) => Object.assign({ id, t: 'image', panel, x, y, w: fw, h: fh, tint, label }, more || {});
    const big = kind === 'picture' || kind === 'illustrated' || kind === 'memoir';
    return [
      big ? I('cv-art', 'front', 0, -0.125, w + 0.125, h + 0.25, 'p', 'Front cover artwork — full bleed') : I('cv-art', 'front', w * 0.12, h * 0.36, w * 0.76, h * 0.36, 'o', 'Cover illustration'),
      T('cv-title', 'front', w * 0.08, h * 0.08, w * 0.84, h * 0.2, big ? 'display' : 'title', title),
      T('cv-author', 'front', w * 0.08, h * 0.86, w * 0.84, 0.5, 'byline', 'Mom'),
      T('cv-spine', 'spine', 0, h * 0.06, 0, h * 0.88, 'spine', title + '   ·   Mom'),
      T('cv-blurb', 'back', 0.6, 0.8, w - 1.2, h * 0.38, 'bodyL', 'A short, warm description of the book goes here — the few lines that make someone pick it up and turn it over.'),
      I('cv-photo', 'back', 0.6, h * 0.52, 1.3, 1.3, 'g', 'Author photo', { corners: 'circle' }),
      T('cv-bio', 'back', 2.1, h * 0.52, w - 2.7, 1.3, 'caption', 'Mom writes at the kitchen table, usually with tea going cold beside her.'),
    ];
  }
  function flapFrames(K) {
    return [{ id: 'fl-blurb', t: 'text', panel: 'fflap', x: 0.35, y: 0.8, w: 2.8, h: K.h * 0.6, s: 'bodyL', text: 'The front flap carries a longer description of the story — room for a second paragraph and a quote from an early reader.' },
      { id: 'fl-bio', t: 'text', panel: 'bflap', x: 0.35, y: K.h * 0.45, w: 2.8, h: K.h * 0.35, s: 'caption', text: 'About the author. A few lines, a town, a cat.' },
      { id: 'fl-photo', t: 'image', panel: 'bflap', x: 0.85, y: 0.8, w: 1.8, h: 1.8, tint: 'g', label: 'Author photo', corners: 'circle' }];
  }

  // ── shapes (px space) ──
  function shapePath(kind, w, h) {
    const r = Math.min(w, h);
    if (kind === 'rect') return 'M0 0H' + w + 'V' + h + 'H0Z';
    if (kind === 'round') { const q = r * 0.18; return 'M' + q + ' 0H' + (w - q) + 'Q' + w + ' 0 ' + w + ' ' + q + 'V' + (h - q) + 'Q' + w + ' ' + h + ' ' + (w - q) + ' ' + h + 'H' + q + 'Q0 ' + h + ' 0 ' + (h - q) + 'V' + q + 'Q0 0 ' + q + ' 0Z'; }
    if (kind === 'ellipse') return 'M' + w / 2 + ' 0A' + w / 2 + ' ' + h / 2 + ' 0 1 1 ' + w / 2 + ' ' + h + 'A' + w / 2 + ' ' + h / 2 + ' 0 1 1 ' + w / 2 + ' 0Z';
    if (kind === 'line') return 'M0 ' + h / 2 + 'H' + w;
    if (kind === 'speech') { const bh = h * 0.78, q = Math.min(w, bh) * 0.3; return 'M' + q + ' 0H' + (w - q) + 'Q' + w + ' 0 ' + w + ' ' + q + 'V' + (bh - q) + 'Q' + w + ' ' + bh + ' ' + (w - q) + ' ' + bh + 'H' + w * 0.34 + 'L' + w * 0.16 + ' ' + h + 'L' + w * 0.22 + ' ' + bh + 'H' + q + 'Q0 ' + bh + ' 0 ' + (bh - q) + 'V' + q + 'Q0 0 ' + q + ' 0Z'; }
    if (kind === 'thought') { const bh = h * 0.76, e = (cx, cy, rx, ry) => 'M' + (cx - rx) + ' ' + cy + 'a' + rx + ' ' + ry + ' 0 1 0 ' + 2 * rx + ' 0a' + rx + ' ' + ry + ' 0 1 0 ' + -2 * rx + ' 0Z'; return e(w / 2, bh / 2, w / 2, bh / 2) + e(w * 0.2, bh + h * 0.08, r * 0.07, r * 0.07) + e(w * 0.1, h - r * 0.04, r * 0.04, r * 0.04); }
    if (kind === 'shout') { const pts = [], n = 14; for (let i = 0; i < n * 2; i++) { const a = (i / (n * 2)) * Math.PI * 2, rr = i % 2 ? 0.78 : 1; pts.push((w / 2 + Math.cos(a) * w / 2 * rr).toFixed(1) + ' ' + (h / 2 + Math.sin(a) * h / 2 * rr).toFixed(1)); } return 'M' + pts.join('L') + 'Z'; }
    return 'M0 0H' + w + 'V' + h + 'H0Z';
  }

  // ── backgrounds ──
  const TEXTURES = {
    linen: { n: 'Linen', css: { backgroundColor: '#f3eee4', backgroundImage: 'repeating-linear-gradient(0deg, rgba(120,100,70,0.06) 0 1px, transparent 1px 3px), repeating-linear-gradient(90deg, rgba(120,100,70,0.05) 0 1px, transparent 1px 4px)' } },
    laid: { n: 'Laid paper', css: { backgroundColor: '#f7f3ea', backgroundImage: 'repeating-linear-gradient(0deg, rgba(90,80,60,0.07) 0 1px, transparent 1px 5px), repeating-linear-gradient(90deg, rgba(90,80,60,0.06) 0 1px, transparent 1px 60px)' } },
    kraft: { n: 'Kraft', css: { backgroundColor: '#cfae82', backgroundImage: 'radial-gradient(rgba(90,60,30,0.12) 1px, transparent 1.5px), radial-gradient(rgba(255,255,255,0.12) 1px, transparent 1.5px)', backgroundSize: '7px 7px, 11px 11px' } },
    wash: { n: 'Watercolour wash', css: { backgroundColor: '#f6f1ea', backgroundImage: 'radial-gradient(ellipse at 20% 25%, rgba(110,205,220,0.35), transparent 55%), radial-gradient(ellipse at 80% 70%, rgba(255,180,84,0.3), transparent 50%), radial-gradient(ellipse at 60% 15%, rgba(226,93,117,0.18), transparent 45%)' } },
    dots: { n: 'Dot grid', css: { backgroundColor: PAPER, backgroundImage: 'radial-gradient(rgba(60,60,60,0.22) 1px, transparent 1.4px)', backgroundSize: '14px 14px' } },
  };
  const GRADIENTS = { dusk: ['Dusk', 'linear-gradient(180deg, #2b2d55, #a35d8f 60%, #f1a66a)'], meadow: ['Meadow', 'linear-gradient(180deg, #d9f0ff, #e8f6d4 70%, #b9dc8f)'], sea: ['Sea', 'linear-gradient(180deg, #e6f6f8, #8fd0dc)'], peach: ['Peach', 'linear-gradient(135deg, #ffe3c8, #ffc0b0)'], night: ['Night', 'radial-gradient(ellipse at 50% 20%, #3b4677, #121633 70%)'] };
  const COLOURS = ['#fbfaf6', '#f6efe1', '#fde2c8', '#e8f1df', '#dceef3', '#ece4f7', '#f9dfe4', '#2b2a33'];
  function bgCss(bg) {
    if (!bg || bg.type === 'none') return { background: PAPER };
    if (bg.type === 'colour') return { background: bg.v };
    if (bg.type === 'gradient') return { background: (GRADIENTS[bg.v] || GRADIENTS.dusk)[1] };
    if (bg.type === 'texture') return Object.assign({}, (TEXTURES[bg.v] || TEXTURES.linen).css);
    if (bg.type === 'picture') return { background: 'color-mix(in srgb, #a78bfa 22%, ' + PAPER + ')', backgroundImage: 'repeating-linear-gradient(45deg, rgba(255,255,255,0.18) 0 10px, transparent 10px 20px)' };
    return { background: PAPER };
  }

  // ── picture treatments ──
  const FILTERS = { none: ['Original', 'none'], warm: ['Warm', 'sepia(0.25) saturate(1.3) hue-rotate(-8deg)'], cool: ['Cool', 'saturate(0.9) hue-rotate(14deg) brightness(1.03)'], bw: ['Black & white', 'grayscale(1) contrast(1.05)'], sepia: ['Sepia', 'sepia(0.85)'], faded: ['Faded', 'contrast(0.82) brightness(1.08) saturate(0.7)'] };
  const BORDERS = { none: ['No border', null], hair: ['Hairline', '1px solid #44403c'], thick: ['Thick', '6px solid #1c1917'], polaroid: ['White mount', 'polaroid'] };
  const CORNERS = { square: ['Square', 0], soft: ['Soft', 0.06], round: ['Round', 0.16], circle: ['Circle', 'circle'] };
  const ORNAMENTS = { fleuron: ['Fleuron', '❦'], stars: ['Three stars', '⁂'], dots: ['Three dots', '•  •  •'], wave: ['Wave', '〰〰〰'], rule: ['Simple rule', '———'] };

  // ── spread templates (spread-relative inches; P = 2) ──
  const TEMPLATES = [
    { id: 'pb-bleed', group: 'Picture book', n: 'Full-bleed spread', build: (K) => [img('a', -K.b, -K.b, 2 * K.w + 2 * K.b, K.h + 2 * K.b, 'c', 'One picture across both pages'), txt('t', K.w + K.m, K.h - K.m - 1.6, K.w - 2 * K.m, 1.5, 'story', 'And the words sit quietly in the corner of the picture.')] },
    { id: 'pb-split', group: 'Picture book', n: 'Picture + words', build: (K) => [img('a', -K.b, -K.b, K.w + K.b, K.h + 2 * K.b, 'o', 'Full-page picture'), txt('t', K.w + K.m, K.h * 0.35, K.w - 2 * K.m, K.h * 0.3, 'storyC', 'Words on their own page, centred and calm.')] },
    { id: 'pb-vignette', group: 'Picture book', n: 'Vignettes', build: (K) => [img('a', K.w * 0.2, K.h * 0.16, K.w * 0.6, K.w * 0.6, 'g', 'Vignette', { corners: 'circle' }), txt('t', K.m, K.h * 0.16 + K.w * 0.66, K.w - 2 * K.m, 1.2, 'storyC', 'A small moment.'), img('b', K.w + K.w * 0.2, K.h * 0.16, K.w * 0.6, K.w * 0.6, 'r', 'Vignette', { corners: 'circle' }), txt('u', K.w + K.m, K.h * 0.16 + K.w * 0.66, K.w - 2 * K.m, 1.2, 'storyC', 'And another.')] },
    { id: 'pb-panorama', group: 'Picture book', n: 'Panorama + words below', build: (K) => [img('a', -K.b, -K.b, 2 * K.w + 2 * K.b, K.h * 0.69 + K.b, 'r', 'One wide picture across the top of both pages'), txt('t', K.m + 0.25, K.h * 0.74, K.w - 2 * K.m - 0.25, K.h * 0.19, 'story', 'Words for the left page sit under the picture.'), txt('u', K.w + K.m, K.h * 0.74, K.w - 2 * K.m - 0.25, K.h * 0.19, 'story', 'And the story carries on over here.')] },
    { id: 'pb-panorama-low', group: 'Picture book', n: 'Words above panorama', build: (K) => [txt('t', K.m + 0.25, K.m, K.w - 2 * K.m - 0.25, K.h * 0.2, 'story', 'Words up top on the left…'), txt('u', K.w + K.m, K.m, K.w - 2 * K.m - 0.25, K.h * 0.2, 'story', '…and on the right.'), img('a', -K.b, K.h * 0.3, 2 * K.w + 2 * K.b, K.h * 0.7 + K.b, 'g', 'One wide picture across the bottom of both pages')] },
    { id: 'pb-words-left', group: 'Picture book', n: 'Words + picture', build: (K) => [txt('t', K.m, K.h * 0.33, K.w - 2 * K.m, K.h * 0.3, 'storyC', 'Words on the left, the picture gets the right-hand page.'), img('a', K.w, -K.b, K.w + K.b, K.h + 2 * K.b, 'g', 'Full-page picture')] },
    { id: 'pb-overlay', group: 'Picture book', n: 'Words over the picture', build: (K) => [img('a', -K.b, -K.b, 2 * K.w + 2 * K.b, K.h + 2 * K.b, 'p', 'One picture across both pages'), txt('t', K.w + K.m, K.m, K.w - 2 * K.m, K.h * 0.24, 'callout', 'Words sit in a soft box on top of the picture, where the sky is quiet.')] },
    { id: 'pb-two', group: 'Picture book', n: 'Two pictures, two captions', build: (K) => [img('a', K.m, K.m, K.w - 2 * K.m, K.h * 0.62, 'o', 'Picture'), txt('t', K.m, K.m + K.h * 0.66, K.w - 2 * K.m, K.h * 0.2, 'story', 'Something happens here.'), img('b', K.w + K.m, K.m, K.w - 2 * K.m, K.h * 0.62, 'c', 'Picture'), txt('u', K.w + K.m, K.m + K.h * 0.66, K.w - 2 * K.m, K.h * 0.2, 'story', 'And then this.')] },
    { id: 'pb-panels', group: 'Picture book', n: 'Comic panels', build: (K) => { const g = 0.15, pw = (K.w - 2 * K.m - g) / 2, ph = (K.h - 2 * K.m - g) / 2, out = []; [0, 1].forEach((p) => [0, 1].forEach((r) => [0, 1].forEach((c) => { const i = p * 4 + r * 2 + c; out.push(img('p' + i, p * K.w + K.m + c * (pw + g), K.m + r * (ph + g), pw, ph, ['c', 'o', 'p', 'g', 'r', 'c', 'o', 'p'][i], 'Panel ' + (i + 1), { border: 'thick' })); }))); out.push(Object.assign(img('x', 0, 0, 0, 0, 'p', ''), { t: 'shape', sh: 'speech', x: K.m + 0.2, y: K.m + 0.15, w: pw * 0.7, h: ph * 0.4, fill: '#ffffff', stroke: '#1c1b20', sw: 1.5, s: 'bubble', text: 'Oh!' })); return out; } },
    { id: 'pb-text', group: 'Picture book', n: 'Words only (opening or ending)', build: (K) => [txt('t', K.w + K.m, K.h * 0.38, K.w - 2 * K.m, K.h * 0.24, 'storyC', 'The end.'), txt('o', K.w + K.m, K.h * 0.64, K.w - 2 * K.m, 0.5, 'ornament', '❦')] },
    { id: 'recipe', group: 'Cookbook', n: 'Recipe page', build: (K) => [img('a', -K.b, -K.b, K.w + K.b, K.h + 2 * K.b, 'o', 'Photo of the dish'), txt('l', K.w + K.m, K.m, K.w - 2 * K.m, 0.35, 'label', 'Season · Course'), txt('t', K.w + K.m, K.m + 0.4, K.w - 2 * K.m, 0.9, 'h1big', 'Recipe name'), txt('m', K.w + K.m, K.m + 1.35, K.w - 2 * K.m, 0.35, 'caption', 'Serves 4 · 45 minutes'), txt('ih', K.w + K.m, K.m + 2, (K.w - 2 * K.m) * 0.4, 0.35, 'h2', "You'll need"), txt('i', K.w + K.m, K.m + 2.4, (K.w - 2 * K.m) * 0.4, 4, 'bodyL', '1 cup this\n2 tbsp that\nA pinch of salt', { list: 'bullet' }), txt('mh', K.w + K.m + (K.w - 2 * K.m) * 0.46, K.m + 2, (K.w - 2 * K.m) * 0.54, 0.35, 'h2', 'Method'), txt('me', K.w + K.m + (K.w - 2 * K.m) * 0.46, K.m + 2.4, (K.w - 2 * K.m) * 0.54, 5, 'bodyL', 'Heat the oven.\nMix everything.\nBake until golden.', { list: 'number' })] },
    { id: 'recipe-two', group: 'Cookbook', n: 'Two recipes', build: (K) => [0, 1].reduce((a, p) => a.concat([img('a' + p, p * K.w + K.m, K.m, K.w - 2 * K.m, K.h * 0.34, p ? 'g' : 'o', 'Photo'), txt('t' + p, p * K.w + K.m, K.m + K.h * 0.37, K.w - 2 * K.m, 0.6, 'h1', p ? 'Second recipe' : 'First recipe'), txt('m' + p, p * K.w + K.m, K.m + K.h * 0.37 + 0.65, K.w - 2 * K.m, 0.3, 'caption', 'Serves 4 · 20 minutes'), txt('i' + p, p * K.w + K.m, K.m + K.h * 0.37 + 1.1, (K.w - 2 * K.m) * 0.4, K.h * 0.4, 'bodyL', 'Butter\nFlour\nSugar', { list: 'bullet' }), txt('s' + p, p * K.w + K.m + (K.w - 2 * K.m) * 0.45, K.m + K.h * 0.37 + 1.1, (K.w - 2 * K.m) * 0.55, K.h * 0.4, 'bodyL', 'Mix.\nBake.\nShare.', { list: 'number' })]), []) },
    { id: 'lesson-2col', group: 'Textbook', n: 'Two-column reading', build: (K) => { const cw = (K.w - 2 * K.m - 0.25) / 2; return [txt('h', K.m, K.m, K.w - 2 * K.m, 0.8, 'h1', 'Section title'), txt('a', K.m, K.m + 1, cw, K.h - 2 * K.m - 1, 'body', 'A long reading set in two columns, so lines stay short and easy to follow.'), txt('b', K.m + cw + 0.25, K.m + 1, cw, K.h - 2 * K.m - 1, 'body', 'The second column carries on.'), txt('n', K.w + K.m, K.m, K.w - 2 * K.m, 1.4, 'callout', 'Key idea · a one-line summary of the section.'), img('f', K.w + K.m, K.m + 1.7, K.w - 2 * K.m, K.h * 0.4, 'c', 'Diagram'), txt('c', K.w + K.m, K.m + 1.8 + K.h * 0.4, K.w - 2 * K.m, 0.5, 'caption', 'Figure · what the diagram shows.')]; } },
    { id: 'lesson', group: 'Textbook', n: 'Lesson + worksheet', build: (K) => [txt('l', K.m, K.m, K.w - 2 * K.m, 0.35, 'label', 'Lesson 4'), txt('h', K.m, K.m + 0.4, K.w - 2 * K.m, 0.9, 'h1big', 'Lesson title'), txt('o', K.m, K.m + 1.45, K.w - 2 * K.m, 1.1, 'callout', "You'll learn to… · one thing · another thing"), txt('b', K.m, K.m + 2.8, K.w - 2 * K.m, 4, 'bodyL', 'Introduce the idea in a paragraph or two.'), txt('h2', K.w + K.m, K.m, K.w - 2 * K.m, 0.45, 'h2', 'Try it yourself'), tbl('w', K.w + K.m, K.m + 0.55, K.w - 2 * K.m, 3, [['Step', 'What you did', 'What happened'], ['1', '', ''], ['2', '', ''], ['3', '', '']], 'grid'), img('f', K.w + K.m, K.m + 3.9, K.w - 2 * K.m, 3, 'c', 'Figure'), txt('c', K.w + K.m, K.m + 7, K.w - 2 * K.m, 0.5, 'caption', 'Figure 4.1 · A caption explains the picture.')] },
    { id: 'collage', group: 'Photos', n: 'Photo collage', build: (K) => { const g = 0.15, cw = (2 * K.w - 2 * K.m - 2 * g) / 3, ch = (K.h - 2 * K.m - g) / 2; return [img('a', K.m, K.m, cw * 2 + g, ch, 'g', 'Photo'), img('b', K.m + 2 * (cw + g), K.m, cw, ch, 'o', 'Photo'), img('c', K.m, K.m + ch + g, cw, ch, 'p', 'Photo'), img('d', K.m + cw + g, K.m + ch + g, cw, ch, 'c', 'Photo'), img('e', K.m + 2 * (cw + g), K.m + ch + g, cw, ch * 0.7, 'r', 'Photo'), txt('t', K.m + 2 * (cw + g), K.m + ch + g + ch * 0.74, cw, ch * 0.26, 'caption', 'Summer, the year the porch went up.')]; } },
    { id: 'memoir-photo', group: 'Photos', n: 'Photo page + story', build: (K) => [img('a', -K.b, -K.b, K.w + K.b, K.h * 0.7 + K.b, 'g', 'Photo'), txt('c', K.m, K.h * 0.72, K.w - 2 * K.m, 0.7, 'caption', 'A caption: who, where, which summer.'), txt('b', K.w + K.m, K.m, K.w - 2 * K.m, K.h - 2 * K.m, 'body', 'The story that goes with the photo.')] },
    { id: 'memoir-polaroids', group: 'Photos', n: 'Scattered snapshots', build: (K) => [img('a', K.m + 0.2, K.m + 0.3, K.w * 0.5, K.w * 0.5, 'o', 'Snapshot', { border: 'polaroid', rot: -6 }), img('b', K.w * 0.42, K.h * 0.45, K.w * 0.48, K.w * 0.48, 'c', 'Snapshot', { border: 'polaroid', rot: 5 }), img('c', K.w + K.m + 0.3, K.m + 0.2, K.w * 0.5, K.w * 0.5, 'p', 'Snapshot', { border: 'polaroid', rot: 4 }), txt('t', K.w + K.m, K.h * 0.62, K.w - 2 * K.m, K.h * 0.25, 'bodyL', 'A few lines about the day.', { rot: -1 })] },
    { id: 'poem-facing', group: 'Poetry', n: 'Two poems facing', build: (K) => [0, 1].reduce((a, p) => a.concat([txt('t' + p, p * K.w + K.m, K.h * 0.16, K.w - 2 * K.m, 0.6, 'h1', p ? 'Second poem' : 'First poem'), txt('p' + p, p * K.w + K.m, K.h * 0.16 + 0.9, K.w - 2 * K.m, K.h * 0.62, 'poem', 'A first line\nand the next,\n\na second stanza\nto close.')]), []) },
    { id: 'poem', group: 'Poetry', n: 'Poem, centred', build: (K) => [txt('o', K.m, K.h * 0.45, K.w - 2 * K.m, 0.6, 'ornament', '❦'), txt('t', K.w + K.m, K.h * 0.18, K.w - 2 * K.m, 0.6, 'h1', 'Title', { al: 'center' }), txt('p', K.w + K.m, K.h * 0.18 + 0.9, K.w - 2 * K.m, K.h * 0.6, 'poem', 'The first line\nand the turn of the second,\n\nthen a quiet stanza\nto end on.', { al: 'center' })] },
    { id: 'op-classic', group: 'Chapter openers', n: 'Classic opener', build: (K) => [txt('l', K.w + K.m, K.h * 0.2, K.w - 2 * K.m, 0.35, 'label', 'Chapter two', { al: 'center' }), txt('t', K.w + K.m, K.h * 0.2 + 0.4, K.w - 2 * K.m, 0.7, 'h1', 'Chapter title', { al: 'center' }), txt('o', K.w + K.m, K.h * 0.2 + 1.15, K.w - 2 * K.m, 0.4, 'ornament', '⁂'), txt('b', K.w + K.m, K.h * 0.2 + 1.7, K.w - 2 * K.m, K.h * 0.8 - K.m - 1.7, 'body', 'The chapter begins here, with a drop cap to welcome the reader in.', { dc: true })] },
    { id: 'op-photo', group: 'Chapter openers', n: 'Photo opener', build: (K) => [img('a', -K.b, -K.b, K.w + K.b, K.h + 2 * K.b, 'p', 'Chapter photo'), txt('n', K.w + K.m, K.m + 0.4, K.w - 2 * K.m, 1.2, 'numeral', '3'), txt('t', K.w + K.m, K.m + 1.7, K.w - 2 * K.m, 0.9, 'h1big', 'Chapter title'), txt('b', K.w + K.m, K.m + 2.9, K.w - 2 * K.m, K.h - 2 * K.m - 2.9, 'body', 'Opening paragraph.')] },
    { id: 'op-part', group: 'Chapter openers', n: 'Part title', build: (K) => [txt('l', K.w + K.m, K.h * 0.36, K.w - 2 * K.m, 0.4, 'label', 'Part two', { al: 'center' }), txt('t', K.w + K.m, K.h * 0.36 + 0.45, K.w - 2 * K.m, 1, 'h1big', 'The long summer', { al: 'center' }), txt('o', K.w + K.m, K.h * 0.36 + 1.55, K.w - 2 * K.m, 0.5, 'ornament', '⁂')] },
    { id: 'body-plain', group: 'Text', n: 'Plain text spread', build: (K) => [txt('a', K.m, K.m, K.w - 2 * K.m, K.h - 2 * K.m, 'body', 'Body text on the left page.'), txt('b', K.w + K.m, K.m, K.w - 2 * K.m, K.h - 2 * K.m, 'body', 'Body text continues on the right.')] },
    { id: 'body-quote', group: 'Text', n: 'Text with pull quote', build: (K) => [txt('a', K.m, K.m, K.w - 2 * K.m, K.h - 2 * K.m, 'body', 'Body text on the left page.'), txt('b', K.w + K.m, K.m, K.w - 2 * K.m, K.h * 0.4, 'body', 'Body text continues…'), txt('q', K.w + K.m + 0.3, K.h * 0.45, K.w - 2 * K.m - 0.6, K.h * 0.2, 'epigraph', '“A line worth repeating, set large and quiet.”'), txt('c', K.w + K.m, K.h * 0.68, K.w - 2 * K.m, K.h * 0.32 - K.m, 'body', '…and after the quote.')] },
    { id: 'op-minimal', group: 'Chapter openers', n: 'Minimal opener', build: (K) => [txt('n', K.w + K.m, K.h * 0.42, K.w - 2 * K.m, 1.4, 'numeral', '4', { al: 'left' }), txt('t', K.w + K.m, K.h * 0.42 + 1.4, K.w - 2 * K.m, 0.6, 'h1', 'Chapter title'), txt('b', K.w + K.m, K.h * 0.42 + 2.2, K.w - 2 * K.m, K.h * 0.58 - K.m - 2.2, 'body', 'Opening paragraph.')] },
  ];
  function txt(id, x, y, w, h, s, text, more) { return Object.assign({ id, t: 'text', x, y, w, h, s, text }, more || {}); }
  function img(id, x, y, w, h, tint, label, more) { return Object.assign({ id, t: 'image', x, y, w, h, tint, label }, more || {}); }
  function tbl(id, x, y, w, h, rows, ts) { return { id, t: 'table', x, y, w, h, rows, head: true, ts: ts || 'lined' }; }

  // ── ISBN / EAN-13 Bookland barcode with optional EAN-5 price add-on ──
  const EL = ['0001101', '0011001', '0010011', '0111101', '0100011', '0110001', '0101111', '0111011', '0110111', '0001011'];
  const EG = ['0100111', '0110011', '0011011', '0100001', '0011101', '0111001', '0000101', '0010001', '0001001', '0010111'];
  const ER = ['1110010', '1100110', '1101100', '1000010', '1011100', '1001110', '1010000', '1000100', '1001000', '1110100'];
  const PAR13 = ['LLLLLL', 'LLGLGG', 'LLGGLG', 'LLGGGL', 'LGLLGG', 'LGGLLG', 'LGGGLL', 'LGLGLG', 'LGLGGL', 'LGGLGL'];
  const PAR5 = ['GGLLL', 'GLGLL', 'GLLGL', 'GLLLG', 'LGGLL', 'LLGGL', 'LLLGG', 'LGLGL', 'LGLLG', 'LLGLG'];
  const digits = (s) => String(s || '').replace(/[^0-9Xx]/g, '').toUpperCase();
  const check13 = (d12) => { let s = 0; for (let i = 0; i < 12; i++) s += +d12[i] * (i % 2 ? 3 : 1); return (10 - (s % 10)) % 10; };
  function isbnInfo(raw) {
    let d = digits(raw);
    if (d.length === 10) { const core = '978' + d.slice(0, 9); return Object.assign(isbnInfo(core + check13(core)), { converted: true }); }
    if (d.length !== 13 || /X/.test(d)) return { ok: false, digits: d, msg: d.length ? 'An ISBN has 13 digits (or 10 for older ones) — this has ' + d.length + '.' : 'Type or paste your ISBN.' };
    if (!/^97[89]/.test(d)) return { ok: false, digits: d, msg: 'Book ISBNs start with 978 or 979.' };
    const want = check13(d.slice(0, 12));
    if (+d[12] !== want) return { ok: false, digits: d, fixable: d.slice(0, 12) + want, msg: 'The last digit should be ' + want + ' — a typo somewhere, or a mistyped check digit.' };
    return { ok: true, digits: d, pretty: d.slice(0, 3) + '-' + d[3] + '-' + d.slice(4, 9) + '-' + d.slice(9, 12) + '-' + d[12] };
  }
  function ean13(d) {
    const par = PAR13[+d[0]]; let p = '101';
    for (let i = 1; i <= 6; i++) p += (par[i - 1] === 'L' ? EL : EG)[+d[i]];
    p += '01010'; for (let i = 7; i <= 12; i++) p += ER[+d[i]]; return p + '101';
  }
  function ean5(a) { const d = String(a).padStart(5, '0').slice(0, 5), c = (3 * (+d[0] + +d[2] + +d[4]) + 9 * (+d[1] + +d[3])) % 10, par = PAR5[c]; let p = '01011'; for (let i = 0; i < 5; i++) { if (i) p += '01'; p += (par[i] === 'L' ? EL : EG)[+d[i]]; } return p; }
  function priceCode(cur, amount) { if (cur === 'none') return '90000'; const n = Math.round(parseFloat(amount || 0) * 100); if (!n || n > 9999) return '90000'; return (cur === 'usd' ? '5' : '6') + String(n).padStart(4, '0'); }
  // returns module geometry: bars [{x,w,guard}] in modules; total width; add-on start
  function barcodeGeo(d, addon) {
    const main = ean13(d), bars = [], guardIdx = new Set(); [0, 1, 2, 45, 46, 47, 48, 49, 92, 93, 94].forEach((i) => guardIdx.add(i));
    const push = (pat, off, isMain) => { let i = 0; while (i < pat.length) { if (pat[i] === '1') { let j = i; while (j < pat.length && pat[j] === '1') j++; bars.push({ x: off + i, w: j - i, guard: isMain && guardIdx.has(i), add: !isMain }); i = j; } else i++; } };
    const quietL = 11; push(main, quietL, true);
    let total = quietL + 95 + 7, addAt = null;
    if (addon) { addAt = total + 2; push(ean5(addon), addAt, false); total = addAt + 47 + 5; }
    return { bars, total, quietL, addAt, digits: d, addon };
  }

  window.FOLIO_LIB = { isbnInfo, ean13, ean5, priceCode, barcodeGeo, CHAR_KEYS, runsOf, plain, merge, lines, runsToHtml, domToRuns, hasRich, coverGeo, coverFrames, flapFrames, shapePath, TEXTURES, GRADIENTS, COLOURS, bgCss, FILTERS, BORDERS, CORNERS, ORNAMENTS, TEMPLATES, txt, img, tbl };
})();
