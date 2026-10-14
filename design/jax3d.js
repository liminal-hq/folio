// Jax 3D — the Liminal HQ otter, built the way Jar builds its fish:
// flat 2D silhouettes extruded into thin bevelled slabs and layered in z.
// Usage: const jax = await mountJax(el, { size: 120 }); jax.setMood('party'); jax.dispose();
// (c) Liminal HQ · Apache-2.0 OR MIT

const THREE_URL = 'https://unpkg.com/three@0.160.0/build/three.module.js';
let threePromise = null;
const loadThree = () => (threePromise ||= import(THREE_URL));

const COL = {
  fur: 0xb8743a, furDark: 0x8a5428, cream: 0xf6dcb4, nose: 0x2a1a14,
  eye: 0x1a1210, shine: 0xffffff, blush: 0xe25d75, book: 0xe25d75, pages: 0xfbfaf6, amber: 0xffb454,
};

export async function mountJax(host, opts = {}) {
  const THREE = await loadThree();
  const size = opts.size || 120;
  const reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, opts.maxDpr || 2));
  renderer.setSize(size, size);
  renderer.domElement.style.display = 'block';
  host.appendChild(renderer.domElement);

  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(30, 1, 0.1, 50);
  camera.position.set(0, -0.35, 8.6);
  camera.lookAt(0, -0.45, 0);
  scene.add(new THREE.HemisphereLight(0xfff1e0, 0x3a2a40, 1.6));
  const key = new THREE.DirectionalLight(0xffe2c0, 2.2); key.position.set(2.5, 3, 5); scene.add(key);
  const rim = new THREE.DirectionalLight(0xa78bfa, 1.4); rim.position.set(-4, 1, -3); scene.add(rim);

  const mats = {};
  const mat = (c) => (mats[c] ||= new THREE.MeshStandardMaterial({ color: c, roughness: 0.78, metalness: 0 }));
  const ellipse = (rx, ry) => { const s = new THREE.Shape(); s.absellipse(0, 0, rx, ry, 0, Math.PI * 2, false, 0); return s; };
  // extrude a silhouette into a soft slab, centred on z
  const slab = (shape, depth, colour, x = 0, y = 0, z = 0, bevel = 0.06) => {
    const g = new THREE.ExtrudeGeometry(shape, { depth, bevelEnabled: true, bevelThickness: bevel, bevelSize: bevel, bevelSegments: 3, curveSegments: 28 });
    g.translate(0, 0, -depth / 2);
    const m = new THREE.Mesh(g, mat(colour)); m.position.set(x, y, z); return m;
  };

  const jax = new THREE.Group();
  const body = new THREE.Group(); jax.add(body);

  // tail — tapered paddle behind the body
  const tailShape = new THREE.Shape();
  tailShape.moveTo(0, 0.18); tailShape.quadraticCurveTo(0.9, 0.22, 1.35, -0.05); tailShape.quadraticCurveTo(0.9, -0.12, 0, -0.18); tailShape.closePath();
  const tail = new THREE.Group(); tail.position.set(0.45, -1.75, -0.35);
  const tailMesh = slab(tailShape, 0.22, COL.furDark); tailMesh.rotation.z = -0.35; tail.add(tailMesh); body.add(tail);

  body.add(slab(ellipse(0.78, 1.02), 0.72, COL.fur, 0, -1.12, 0));          // torso
  body.add(slab(ellipse(0.5, 0.72), 0.1, COL.cream, 0, -1.18, 0.42, 0.04)); // belly
  const footL = slab(ellipse(0.26, 0.14), 0.4, COL.furDark, -0.38, -2.12, 0.18); body.add(footL);
  const footR = slab(ellipse(0.26, 0.14), 0.4, COL.furDark, 0.38, -2.12, 0.18); body.add(footR);

  // book held against the belly
  const book = new THREE.Group(); book.position.set(0, -1.02, 0.6); book.rotation.x = -0.25;
  book.add(slab(new THREE.Shape([new THREE.Vector2(-0.5, -0.32), new THREE.Vector2(0.5, -0.32), new THREE.Vector2(0.5, 0.32), new THREE.Vector2(-0.5, 0.32)]), 0.08, COL.book, 0, 0, 0, 0.02));
  book.add(slab(new THREE.Shape([new THREE.Vector2(-0.46, -0.28), new THREE.Vector2(0.46, -0.28), new THREE.Vector2(0.46, 0.28), new THREE.Vector2(-0.46, 0.28)]), 0.04, COL.pages, 0, 0, 0.07, 0.01));
  book.add(slab(new THREE.Shape([new THREE.Vector2(-0.015, -0.28), new THREE.Vector2(0.015, -0.28), new THREE.Vector2(0.015, 0.28), new THREE.Vector2(-0.015, 0.28)]), 0.02, COL.furDark, 0, 0, 0.1, 0.005));
  body.add(book);

  // arms — pivot at the shoulder so they can wave
  const arm = (side) => { const p = new THREE.Group(); p.position.set(side * 0.55, -0.55, 0.35); const a = slab(ellipse(0.16, 0.42), 0.26, COL.fur, side * -0.12, -0.34, 0); a.rotation.z = side * -0.4; p.add(a); const paw = slab(ellipse(0.15, 0.12), 0.26, COL.furDark, side * -0.27, -0.7, 0.3); p.add(paw); body.add(p); return p; };
  const armL = arm(-1), armR = arm(1);

  // head
  const head = new THREE.Group(); head.position.set(0, 0.22, 0.05); jax.add(head);
  head.add(slab(ellipse(0.16, 0.16), 0.2, COL.furDark, -0.6, 0.48, -0.12));
  head.add(slab(ellipse(0.16, 0.16), 0.2, COL.furDark, 0.6, 0.48, -0.12));
  head.add(slab(ellipse(0.8, 0.66), 0.7, COL.fur, 0, 0, 0));
  head.add(slab(ellipse(0.4, 0.27), 0.14, COL.cream, 0, -0.2, 0.36, 0.05));
  head.add(slab(ellipse(0.12, 0.08), 0.1, COL.nose, 0, -0.08, 0.47, 0.04));
  head.add(slab(ellipse(0.1, 0.06), 0.03, COL.blush, -0.48, -0.16, 0.37, 0.02));
  head.add(slab(ellipse(0.1, 0.06), 0.03, COL.blush, 0.48, -0.16, 0.37, 0.02));
  const eyes = new THREE.Group(); eyes.position.set(0, 0.12, 0.38); head.add(eyes);
  [-0.3, 0.3].forEach((x) => { const e = slab(ellipse(0.085, 0.095), 0.06, COL.eye, x, 0, 0, 0.02); eyes.add(e); eyes.add(slab(ellipse(0.025, 0.025), 0.02, COL.shine, x + 0.03, 0.035, 0.05, 0.005)); });
  // whiskers — thin slabs
  [-1, 1].forEach((s) => [0.02, -0.06].forEach((dy) => { const w = slab(ellipse(0.2, 0.008), 0.01, COL.furDark, s * 0.52, -0.16 + dy, 0.42, 0.003); w.rotation.z = s * dy * 2; head.add(w); }));

  // ── props (same extrude-and-layer construction) ──
  const rect = (w, h) => new THREE.Shape([new THREE.Vector2(-w / 2, -h / 2), new THREE.Vector2(w / 2, -h / 2), new THREE.Vector2(w / 2, h / 2), new THREE.Vector2(-w / 2, h / 2)]);
  const tri = (w, h) => new THREE.Shape([new THREE.Vector2(-w / 2, 0), new THREE.Vector2(w / 2, 0), new THREE.Vector2(0, h)]);
  const ring = (r, t) => { const sh = new THREE.Shape(); sh.absarc(0, 0, r, 0, Math.PI * 2, false); const hole = new THREE.Path(); hole.absarc(0, 0, r - t, 0, Math.PI * 2, true); sh.holes.push(hole); return sh; };
  const PAW = [-0.27, -0.7, 0.42];
  const prop = (parent) => { const g = new THREE.Group(); g.visible = false; parent.add(g); return g; };
  // flipping page inside the book
  const pageFlip = new THREE.Group(); pageFlip.position.set(0, 0, 0.1); book.add(pageFlip);
  pageFlip.add(slab(rect(0.44, 0.54), 0.012, COL.pages, 0.23, 0, 0, 0.004)); pageFlip.visible = false;
  // pencil (right paw)
  const pencil = prop(armR); pencil.position.set(PAW[0], PAW[1], PAW[2]); pencil.rotation.z = -0.55;
  pencil.add(slab(rect(0.08, 0.62), 0.07, COL.amber, 0, 0.18, 0, 0.015)); pencil.add(slab(rect(0.08, 0.1), 0.07, COL.blush, 0, 0.53, 0, 0.015)); pencil.add(slab(tri(0.08, 0.14), 0.07, 0xf6dcb4, 0, -0.27, 0, 0.01)); const tip = slab(tri(0.03, 0.05), 0.072, COL.nose, 0, -0.27, 0, 0.005); tip.rotation.z = Math.PI; tip.position.y = -0.18; pencil.add(tip);
  pencil.children[2].rotation.z = Math.PI; pencil.children[2].position.y = -0.13;
  // magnifying glass (right paw)
  const glass = prop(armR); glass.position.set(PAW[0], PAW[1], PAW[2]);
  glass.add(slab(rect(0.07, 0.42), 0.07, COL.furDark, 0, 0.18, 0, 0.015)); glass.add(slab(ring(0.24, 0.05), 0.07, 0x9ca3af, 0, 0.62, 0, 0.015));
  const lens = new THREE.Mesh(new THREE.CircleGeometry(0.2, 28), new THREE.MeshStandardMaterial({ color: 0xbfe9f2, transparent: true, opacity: 0.45, roughness: 0.1 })); lens.position.set(0, 0.62, 0.02); glass.add(lens);
  // fishing rod (right paw) + line + bobber
  const rod = prop(armR); rod.position.set(PAW[0], PAW[1], PAW[2]); rod.rotation.z = -0.9;
  rod.add(slab(rect(0.05, 2.0), 0.05, 0x8a5428, 0, 1.0, 0, 0.01));
  const lineGeo = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), new THREE.Vector3(), new THREE.Vector3()]); const line = new THREE.Line(lineGeo, new THREE.LineBasicMaterial({ color: 0xf5f5f4 })); line.visible = false; scene.add(line);
  const ripple = new THREE.Mesh(new THREE.RingGeometry(0.14, 0.17, 32), new THREE.MeshBasicMaterial({ color: 0xbfe9f2, transparent: true, opacity: 0.6, side: THREE.DoubleSide })); ripple.rotation.x = -Math.PI / 2.4; ripple.visible = false; scene.add(ripple);
  const tipV = new THREE.Vector3();
  const bobber = new THREE.Group(); bobber.add(slab(ellipse(0.1, 0.1), 0.1, COL.blush, 0, 0, 0, 0.02)); bobber.add(slab(rect(0.2, 0.05), 0.11, 0xffffff, 0, 0.03, 0, 0.01)); scene.add(bobber); bobber.visible = false;
  // mug (left paw) + steam
  const mug = prop(armL); mug.position.set(0.5, -0.66, 0.42);
  mug.add(slab(rect(0.34, 0.38), 0.3, 0x6ecddc, 0, 0, 0, 0.04)); mug.add(slab(ring(0.12, 0.045), 0.08, 0x6ecddc, -0.21, 0.02, 0, 0.01));
  const grip = slab(ellipse(0.12, 0.1), 0.18, COL.furDark, -0.22, 0.0, 0.12, 0.02); mug.add(grip); mug.add(slab(ellipse(0.13, 0.03), 0.02, 0x7a4b2a, 0, 0.18, 0.12, 0.005));
  const steam = [0, 1].map((i) => { const m = slab(ellipse(0.025, 0.12), 0.01, 0xffffff, -0.06 + i * 0.12, 0.42, 0.05, 0.004); m.material = m.material.clone(); m.material.transparent = true; mug.add(m); return m; });
  // party hat + confetti
  const hat = prop(head); hat.position.set(0.18, 0.55, 0); hat.rotation.z = -0.3;
  hat.add(slab(tri(0.56, 0.7), 0.3, COL.book, 0, 0, 0, 0.03)); hat.add(slab(rect(0.4, 0.06), 0.31, COL.amber, 0, 0.18, 0.01, 0.01)); hat.add(slab(ellipse(0.08, 0.08), 0.12, COL.amber, 0, 0.72, 0, 0.02));
  const confetti = Array.from({ length: 20 }, (_, i) => { const m = slab(rect(0.09, 0.05), 0.01, [COL.amber, COL.blush, 0xa78bfa, 0x6ecddc, 0x4ade80][i % 5], 0, 0, 0, 0.004); m.visible = false; m.userData = { x: (Math.random() - 0.5) * 3.4, s: 0.6 + Math.random() * 0.8, r: Math.random() * 6, d: Math.random() }; scene.add(m); return m; });
  // thumb (right paw)
  const thumb = prop(armR); thumb.position.set(PAW[0] - 0.02, PAW[1] + 0.1, PAW[2]); thumb.add(slab(ellipse(0.06, 0.12), 0.12, COL.furDark, 0, 0.08, 0, 0.02));
  // tiny printer + paper
  const printer = new THREE.Group(); printer.position.set(0, -1.6, 0.95); printer.visible = false; jax.add(printer);
  printer.add(slab(rect(1.1, 0.42), 0.5, 0xd6d3d1, 0, 0, 0, 0.05)); printer.add(slab(rect(0.8, 0.06), 0.52, 0x44403c, 0, 0.12, 0, 0.01)); printer.add(slab(ellipse(0.04, 0.04), 0.52, 0x4ade80, 0.4, -0.08, 0, 0.01));
  const paper = slab(rect(0.66, 0.5), 0.01, COL.pages, 0, 0.2, -0.05, 0.004); printer.add(paper);
  // water to float on
  const water = slab(ellipse(2.3, 0.28), 0.1, 0x6ecddc, 0, -1.2, -0.4, 0.04); water.material = water.material.clone(); water.material.transparent = true; water.material.opacity = 0.55; water.visible = false; scene.add(water);

  jax.position.y = 0.55;
  scene.add(jax);

  let mood = opts.mood || 'chill', moodAt = 0, raf = 0, visible = true, t0 = performance.now(), lastT = 0;
  let nextBlink = 2.5;
  const io = new IntersectionObserver((es) => { visible = es[0].isIntersecting; if (visible && !raf) loop(); });
  io.observe(host);
  const props = [pencil, glass, rod, mug, hat, thumb, printer];

  function frame(t) {
    const m = t - moodAt, dt = Math.min(0.1, t - lastT); lastT = t;
    jax.rotation.set(0, Math.sin(t * 0.55) * 0.5, 0);
    jax.position.set(0, 0.55 + Math.sin(t * 1.6) * 0.05, 0);
    head.rotation.set(0, 0, Math.sin(t * 0.8) * 0.06);
    tail.rotation.y = Math.sin(t * 2.2) * 0.35;
    armL.rotation.z = 0; armR.rotation.z = 0; book.visible = true; book.position.set(0, -1.02, 0.6); book.rotation.set(-0.25, 0, 0);
    props.forEach((p) => { p.visible = false; }); pageFlip.visible = false; bobber.visible = false; line.visible = false; ripple.visible = false; mug.rotation.z = 0; water.visible = false; confetti.forEach((c) => { c.visible = false; });
    let eyeOpen = 1;
    if (t > nextBlink) { eyeOpen = 0.12; if (t > nextBlink + 0.12) nextBlink = t + 2.5 + Math.random() * 3; }
    switch (mood) {
      case 'chill': case 'reading': head.rotation.x = 0.16; jax.rotation.y *= 0.45; break;
      case 'flipping': { head.rotation.x = 0.16; jax.rotation.y *= 0.35; pageFlip.visible = true; const c = (t % 2.4) / 2.4; const p = Math.min(1, Math.max(0, (c - 0.3) / 0.4)); pageFlip.rotation.y = -Math.PI * p; pageFlip.position.z = 0.1 + Math.sin(p * Math.PI) * 0.06; const lift = Math.min(1, Math.max(0, (c - 0.18) / 0.12)) * (1 - Math.min(1, Math.max(0, (c - 0.75) / 0.12))); armR.rotation.z = lift * 1.15; armL.rotation.z = -lift * 0.25; break; }
      case 'writing': head.rotation.x = 0.22; jax.rotation.y *= 0.25; pencil.visible = true; armR.rotation.z = 0.25 + Math.sin(t * 13) * 0.06; pencil.rotation.z = -0.55 + Math.sin(t * 13) * 0.08; pencil.position.y = PAW[1] + 0.12 + Math.sin(t * 13) * 0.02; break;
      case 'checking': glass.visible = true; jax.rotation.y = Math.sin(t * 1.2) * 0.25; armR.rotation.z = 1.0 + Math.sin(t * 2.4) * 0.3; head.rotation.z = Math.sin(t * 2.4) * 0.1; book.visible = false; break;
      case 'fishing': { rod.visible = true; book.visible = false; const cyc = t % 6, bite = cyc > 4.2 && cyc < 5.2, tug = bite ? Math.sin((cyc - 4.2) * Math.PI) : 0;
        armR.rotation.z = 0.45 + Math.sin(t * 1.3) * 0.08 + tug * 0.35; rod.rotation.z = -0.9 + Math.sin(t * 1.3) * 0.06 + tug * 0.25; jax.rotation.y = -0.3 + Math.sin(t * 0.4) * 0.08; head.rotation.z = -0.08 - tug * 0.08; eyeOpen = bite ? 1 : eyeOpen;
        bobber.visible = true; bobber.position.set(1.55, -1.75 + Math.sin(t * 2.2) * 0.04 - tug * 0.18, 0.7); bobber.rotation.z = Math.sin(t * 2.2) * 0.15;
        ripple.visible = true; const rp = (t * 0.6) % 1; ripple.position.set(1.55, -1.83, 0.7); ripple.scale.setScalar(1 + rp * 1.6 + tug); ripple.material.opacity = 0.55 * (1 - rp);
        jax.updateMatrixWorld(true); rod.localToWorld(tipV.set(0, 2.0, 0)); const pos = lineGeo.attributes.position; pos.setXYZ(0, tipV.x, tipV.y, tipV.z); pos.setXYZ(1, (tipV.x + 1.55) / 2, Math.min(tipV.y, -1.6) - 0.15 + tug * 0.2, (tipV.z + 0.7) / 2); pos.setXYZ(2, 1.55, bobber.position.y + 0.08, 0.7); pos.needsUpdate = true; line.visible = true; break; }
      case 'tea': mug.visible = true; book.visible = false; { const sip = (t % 5) < 1.2; armL.rotation.z = sip ? -0.95 : -0.3; mug.rotation.z = -armL.rotation.z; head.rotation.x = sip ? -0.12 : 0.05; jax.rotation.y *= 0.4; steam.forEach((sm, i) => { sm.position.y = 0.42 + ((t * 0.5 + i * 0.5) % 1) * 0.35; sm.material.opacity = 0.7 * (1 - ((t * 0.5 + i * 0.5) % 1)); }); } break;
      case 'wave': armR.rotation.z = 2.4 + Math.sin(t * 9) * 0.35; if (m > 2.4) mood = 'chill'; break;
      case 'thumbsup': thumb.visible = true; armR.rotation.z = 1.5 + Math.min(1, m * 4) * 0.3; head.rotation.z = 0.12; jax.rotation.y *= 0.3; if (m > 2.6) mood = 'chill'; break;
      case 'stuck': armR.rotation.z = 2.75 + Math.sin(t * 14) * 0.08; head.rotation.z = 0.2; head.rotation.x = -0.06; jax.rotation.y *= 0.3; if (m > 3.2) mood = 'chill'; break;
      case 'printing': printer.visible = true; book.visible = false; head.rotation.x = 0.25; jax.rotation.y *= 0.2; { const c = (t % 1.8) / 1.8; paper.position.y = 0.05 + c * 0.42; paper.visible = c < 0.95; printer.position.y = -1.6 + Math.sin(t * 30) * 0.004; } break;
      case 'party': {
        hat.visible = true; book.visible = false; const p = Math.min(1, m / 1.6);
        if (m < 1.6) { jax.position.y += Math.sin(p * Math.PI) * 0.5; jax.rotation.y = p * Math.PI * 2; }
        armL.rotation.z = -2.2 + Math.sin(t * 8) * 0.15; armR.rotation.z = 2.2 - Math.sin(t * 8) * 0.15;
        confetti.forEach((c) => { const u = c.userData, y = 2.2 - ((m * u.s + u.d * 3) % 4.4); c.visible = true; c.position.set(u.x + Math.sin(t * 2 + u.r) * 0.15, y, 0.6); c.rotation.set(t * 3 + u.r, t * 2 + u.r, 0); });
        if (m > 3.2) mood = 'chill'; break;
      }
      case 'nap': eyeOpen = 0.08; book.visible = false; water.visible = true; jax.rotation.set(-0.2, 0, -Math.PI / 2 * 0.92); jax.position.set(0.15, -0.55 + Math.sin(t * 0.9) * 0.06, 0); armL.rotation.z = -0.5; armR.rotation.z = 0.5; tail.rotation.y = Math.sin(t * 0.8) * 0.15; water.position.y = -1.2 + Math.sin(t * 0.9) * 0.02; break;
    }
    eyes.scale.y = eyeOpen;
    renderer.render(scene, camera);
  }
  const minDt = 1000 / (opts.fps || 30); let last = 0;
  function loop() {
    raf = 0;
    if (!visible) return;
    const now = performance.now();
    if (now - last >= minDt) { last = now; frame((now - t0) / 1000); }
    if (!reduce) raf = requestAnimationFrame(loop);
  }
  loop();

  return {
    setMood(next) { mood = next; moodAt = (performance.now() - t0) / 1000; if (reduce) frame(moodAt); },
    dispose() {
      cancelAnimationFrame(raf); io.disconnect();
      scene.traverse((o) => { if (o.geometry) o.geometry.dispose(); });
      Object.values(mats).forEach((m) => m.dispose());
      renderer.dispose(); renderer.forceContextLoss && renderer.forceContextLoss();
      renderer.domElement.remove();
    },
  };
}
