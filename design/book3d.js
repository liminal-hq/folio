// Folio — 3D book for the About window. Same approach as Jax: simple shapes, extruded and layered.
// Usage: const b = await mountBook(el, { size: 300 }); b.toggle(); b.dispose();
const THREE_URL = 'https://unpkg.com/three@0.160.0/build/three.module.js';
let tp = null; const loadThree = () => (tp ||= import(THREE_URL));

function coverTex(THREE, w, h, front) {
  const c = document.createElement('canvas'); c.width = w; c.height = h; const g = c.getContext('2d');
  const gr = g.createLinearGradient(0, 0, w, h); gr.addColorStop(0, '#2b1a3a'); gr.addColorStop(0.55, '#4a1f33'); gr.addColorStop(1, '#7a3416'); g.fillStyle = gr; g.fillRect(0, 0, w, h);
  const glow = g.createRadialGradient(w * 0.5, h * 0.92, 10, w * 0.5, h * 0.92, h * 0.7); glow.addColorStop(0, 'rgba(255,170,64,0.55)'); glow.addColorStop(1, 'rgba(255,170,64,0)'); g.fillStyle = glow; g.fillRect(0, 0, w, h);
  g.strokeStyle = 'rgba(255,214,160,0.35)'; g.lineWidth = 3; g.strokeRect(22, 22, w - 44, h - 44);
  if (front) {
    g.save(); g.translate(w / 2, h * 0.36); g.strokeStyle = '#ffaa40'; g.fillStyle = 'rgba(255,170,64,0.22)'; g.lineWidth = 9; g.lineJoin = 'round'; g.lineCap = 'round';
    const s = 7; g.beginPath(); g.moveTo(-5 * s, -5.5 * s); g.lineTo(3 * s, -5.5 * s); g.quadraticCurveTo(5 * s, -5.5 * s, 5 * s, -3.5 * s); g.lineTo(5 * s, 6 * s); g.lineTo(-3 * s, 6 * s); g.quadraticCurveTo(-5 * s, 6 * s, -5 * s, 4 * s); g.closePath(); g.fill(); g.stroke();
    g.beginPath(); g.moveTo(-5 * s, 3.5 * s); g.quadraticCurveTo(-5 * s, 1.5 * s, -3 * s, 1.5 * s); g.lineTo(5 * s, 1.5 * s); g.stroke(); g.restore();
    g.fillStyle = '#fff4e6'; g.textAlign = 'center'; g.font = '700 ' + Math.round(h * 0.11) + 'px "Space Grotesk", system-ui, sans-serif'; g.fillText('Folio', w / 2, h * 0.66);
    g.fillStyle = 'rgba(255,230,200,0.75)'; g.font = 'italic ' + Math.round(h * 0.034) + 'px Literata, Georgia, serif'; g.fillText('Books, laid out by hand', w / 2, h * 0.73);
    g.fillStyle = 'rgba(255,230,200,0.55)'; g.font = Math.round(h * 0.026) + 'px system-ui, sans-serif'; g.fillText('liminal hq', w / 2, h * 0.9);
  }
  const t = new THREE.CanvasTexture(c); t.colorSpace = THREE.SRGBColorSpace; t.anisotropy = 4; return t;
}
function pageTex(THREE, w, h) {
  const c = document.createElement('canvas'); c.width = w; c.height = h; const g = c.getContext('2d');
  g.fillStyle = '#fbfaf6'; g.fillRect(0, 0, w, h);
  g.fillStyle = '#1c1b20'; g.textAlign = 'center'; g.font = '600 ' + Math.round(h * 0.06) + 'px Literata, Georgia, serif'; g.fillText('Chapter one', w / 2, h * 0.2);
  g.fillStyle = '#a8a29e'; for (let i = 0; i < 16; i++) { const y = h * 0.3 + i * h * 0.04, len = i === 15 ? 0.4 : 0.72 + (i % 3) * 0.04; g.fillRect(w * 0.14, y, w * len * 0.9, Math.max(2, h * 0.006)); }
  g.fillStyle = '#c2410c'; g.font = '600 ' + Math.round(h * 0.09) + 'px Literata, serif'; g.textAlign = 'left'; g.fillText('O', w * 0.12, h * 0.33);
  const t = new THREE.CanvasTexture(c); t.colorSpace = THREE.SRGBColorSpace; return t;
}
function edgeTex(THREE) {
  const c = document.createElement('canvas'); c.width = 64; c.height = 256; const g = c.getContext('2d'); g.fillStyle = '#f3ede0'; g.fillRect(0, 0, 64, 256);
  for (let x = 0; x < 64; x += 2) { g.fillStyle = x % 4 ? 'rgba(120,100,70,0.10)' : 'rgba(120,100,70,0.22)'; g.fillRect(x, 0, 1, 256); }
  const t = new THREE.CanvasTexture(c); t.colorSpace = THREE.SRGBColorSpace; return t;
}

export async function mountBook(host, opts = {}) {
  const THREE = await loadThree(), size = opts.size || 300, reduce = matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches;
  const r = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true }); r.setPixelRatio(Math.min(devicePixelRatio || 1, 2)); r.setSize(size, size); r.domElement.style.display = 'block'; host.appendChild(r.domElement);
  const scene = new THREE.Scene(), cam = new THREE.PerspectiveCamera(28, 1, 0.1, 50); cam.position.set(0, 0.6, 9.2); cam.lookAt(0, 0, 0);
  scene.add(new THREE.HemisphereLight(0xfff1e0, 0x2a2030, 1.5)); const key = new THREE.DirectionalLight(0xffe2c0, 2.4); key.position.set(3, 4, 5); scene.add(key); const rim = new THREE.DirectionalLight(0xa78bfa, 1.6); rim.position.set(-4, 2, -3); scene.add(rim);
  const W = 2.2, H = 3.1, T = 0.55, BT = 0.07, round = (w, h, rr) => { const s = new THREE.Shape(); s.moveTo(-w / 2 + rr, -h / 2); s.lineTo(w / 2 - rr, -h / 2); s.quadraticCurveTo(w / 2, -h / 2, w / 2, -h / 2 + rr); s.lineTo(w / 2, h / 2 - rr); s.quadraticCurveTo(w / 2, h / 2, w / 2 - rr, h / 2); s.lineTo(-w / 2 + rr, h / 2); s.quadraticCurveTo(-w / 2, h / 2, -w / 2, h / 2 - rr); s.lineTo(-w / 2, -h / 2 + rr); s.quadraticCurveTo(-w / 2, -h / 2, -w / 2 + rr, -h / 2); return s; };
  const geos = [], mats = [];
  const board = (front) => { const g = new THREE.ExtrudeGeometry(round(W + 0.08, H + 0.1, 0.06), { depth: BT, bevelEnabled: true, bevelThickness: 0.015, bevelSize: 0.015, bevelSegments: 2, curveSegments: 8 }); g.translate(0, 0, -BT / 2);
    const pos = g.attributes.position, uv = g.attributes.uv; for (let i = 0; i < pos.count; i++) uv.setXY(i, (pos.getX(i) + (W + 0.08) / 2) / (W + 0.08), (pos.getY(i) + (H + 0.1) / 2) / (H + 0.1)); uv.needsUpdate = true;
    const m = new THREE.MeshStandardMaterial({ map: coverTex(THREE, 512, 720, front), roughness: 0.55, metalness: 0.05 }); geos.push(g); mats.push(m); return new THREE.Mesh(g, m); };
  const book = new THREE.Group(); scene.add(book);
  const back = board(false); back.position.set(0, 0, -T / 2 - BT / 2); book.add(back);
  const pagesM = new THREE.MeshStandardMaterial({ map: edgeTex(THREE), roughness: 0.9 }), pageFaceM = new THREE.MeshStandardMaterial({ map: pageTex(THREE, 512, 720), roughness: 0.85 }); mats.push(pagesM, pageFaceM);
  const pg = new THREE.BoxGeometry(W - 0.06, H - 0.08, T); geos.push(pg); const pages = new THREE.Mesh(pg, [pagesM, pagesM, pagesM, pagesM, pageFaceM, pagesM]); pages.position.x = 0.03; book.add(pages);
  const SR = T / 2 + BT + 0.004, spG = new THREE.CylinderGeometry(SR, SR, H + 0.1, 40, 1, false); geos.push(spG); const spM = new THREE.MeshStandardMaterial({ color: 0x3a1a2c, roughness: 0.6 }); mats.push(spM);
  const spine = new THREE.Mesh(spG, spM); spine.position.x = -W / 2 - 0.02; spine.scale.x = 0.38; book.add(spine);
  const bandG = new THREE.CylinderGeometry(SR + 0.006, SR + 0.006, 0.06, 40, 1, false); geos.push(bandG); const bandM = new THREE.MeshStandardMaterial({ color: 0xffaa40, roughness: 0.4, metalness: 0.3, side: THREE.DoubleSide }); mats.push(bandM);
  [0.95, -0.95].forEach((y) => { const bd = new THREE.Mesh(bandG, bandM); bd.position.set(-W / 2 - 0.02, y * (H / 2), 0); bd.scale.x = 0.4; book.add(bd); });
  const hinge = new THREE.Group(); hinge.position.set(-W / 2 - 0.02, 0, T / 2 + BT / 2); book.add(hinge);
  const front = board(true); front.position.x = W / 2 + 0.02; hinge.add(front);
  const ribbon = new THREE.Mesh(new THREE.PlaneGeometry(0.12, 0.9), new THREE.MeshStandardMaterial({ color: 0xe25d75, roughness: 0.5, side: THREE.DoubleSide })); geos.push(ribbon.geometry); mats.push(ribbon.material); ribbon.position.set(0.5, -H / 2 - 0.3, 0); book.add(ribbon);
  book.rotation.set(-0.12, 0.55, 0.04);
  let open = 0, target = 0, raf = 0, vis = true, t0 = performance.now(), drag = null, spin = 0, spinV = 0;
  const io = new IntersectionObserver((es) => { vis = es[0].isIntersecting; if (vis && !raf) loop(); }); io.observe(host);
  const el = r.domElement; el.style.cursor = 'grab';
  const down = (e) => { drag = { x: e.clientX, s: spin, moved: false }; el.style.cursor = 'grabbing'; el.setPointerCapture && el.setPointerCapture(e.pointerId); };
  const move = (e) => { if (!drag) return; const dx = e.clientX - drag.x; if (Math.abs(dx) > 3) drag.moved = true; spinV = (drag.s + dx * 0.012) - spin; spin = drag.s + dx * 0.012; };
  const up = () => { if (drag && !drag.moved) target = target ? 0 : 1; drag = null; el.style.cursor = 'grab'; };
  el.addEventListener('pointerdown', down); el.addEventListener('pointermove', move); el.addEventListener('pointerup', up); el.addEventListener('pointerleave', () => { if (drag) up(); });
  function frame(t) {
    if (!drag) { spinV *= 0.94; spin += spinV; spin += (0 - spin) * 0.012; }
    open += (target - open) * 0.07;
    book.rotation.y = 0.55 + Math.sin(t * 0.45) * 0.28 + spin - open * 0.45; book.rotation.x = -0.12 + Math.sin(t * 0.6) * 0.04; book.position.y = Math.sin(t * 1.2) * 0.06; book.position.x = open * 0.9;
    hinge.rotation.y = -open * 2.35; ribbon.rotation.z = Math.sin(t * 1.4) * 0.08;
    r.render(scene, cam);
  }
  function loop() { raf = 0; if (!vis) return; frame((performance.now() - t0) / 1000); if (!reduce) raf = requestAnimationFrame(loop); }
  loop();
  return { toggle() { target = target ? 0 : 1; if (reduce) { open = target; frame(0); } }, isOpen: () => target === 1, dispose() { cancelAnimationFrame(raf); io.disconnect(); geos.forEach((g) => g.dispose()); mats.forEach((m) => { if (m.map) m.map.dispose(); m.dispose(); }); r.dispose(); r.forceContextLoss && r.forceContextLoss(); el.remove(); } };
}
