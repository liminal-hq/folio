# 07 · Jax in Folio

Jax 🦦 is the Liminal HQ companion. In Folio he's a small **3D otter** holding a book. He offers gentle, contextual help, can walk a first-time author through a whole book, and answers "how do I…?" questions offline. He is always optional, never covers pages or dialogs, and never nags.

## Consent (D-08)

On the welcome screen: **"Would you like Jax's help?"**

| Choice | Behaviour |
|---|---|
| Yes, walk me through it | Panel opens on the **Guide** tab: a step-by-step walk for this book kind |
| Just tips now and then (default) | Panel opens on **Tip**. Contextual tips, otherwise he chills |
| No thanks | Panel stays closed. He waits on his status-bar perch; click to call him |

You can change this any time: app menu → ☐ Jax, Ctrl+J, **Hush** in the panel, or Preferences → Jax.

## Placement

- **Perch:** the mini Jax (`●‿● jax 🦦`, the DS `Jax variant="mini"`) at the right end of the status bar. It's always there in the editor and hidden on welcome/about (Jax rules).
- **Panel:** clicking the perch expands Jax into the **right end of the page shelf** (470 px). He takes shelf space, so he never covers the canvas or the inspector. Modal dialogs cover him; he never sits on top of a dialog.

## Panel tabs

1. **Tip:** one contextual tip, in Jax's voice (lowercase, wry, kind). There are two buttons: *Another tip* and *I'm lost — what next?*
2. **This page:** a live checklist for the current spread, built from the same geometry as preflight (04): has words · pictures still placeholders · words outside the safe margin · pictures stopping short of the bleed. When everything passes he says "That's everything on this page."
3. **Guide** (only when chosen): "Step 2 of 6 · Tell the story", with *Back*, *Show me* (navigates, e.g. jumps to page 1, switches to the Kindle preview, opens Export) and *Next*. There are separate step lists for fixed and flowing kinds.
4. **Ask:** free-form "How do I…?", answered from Folio's help library, on this computer.

**I'm lost** picks the first open item from This page. If there is none, it picks the next Guide step. If there is none of those either, it suggests the next likely action (preview, then export).

## Contextual tip triggers (first match wins; *Another tip* rotates the general pool)

| Context | Tip (voice sample) |
|---|---|
| Dragging with guides firing | "hold alt while dragging if the guides are being a bit too helpful." |
| Picture selected | "pictures that touch the edge should run past it to the pink dashed line…" |
| Text selected | "double-click a text box to type straight onto the page…" |
| Reflow preview | "on e-readers, people pick their own text size. your words flow; your page breaks don't…" |
| Fixed-layout preview | "keep story text 18 pt or bigger so it reads well on a small screen." |
| Grid shown | "things that line up look calm, even when nobody can say why." |

Rate limit: at most one unprompted tip change per 90 seconds. There are never pop-ups, sounds or badges.

## 3D model (`jax3d.js` → `apps/folio/src/jax/`)

Built **the way Jar builds its fish** (`apps/jar/src/render/models/fishGeometry.ts`): flat 2D silhouettes become `THREE.Shape`s and are extruded into thin bevelled slabs (`ExtrudeGeometry`, `bevelSegments: 3`, `curveSegments: 28`) layered in z. Production will author the silhouettes as SVG (`jax-svg/*.svg`, parsed once with `SVGLoader`, shared across instances), as Jar does. The prototype builds them in code.

| Piece | Shape | Depth | Colour |
|---|---|---|---|
| Torso | ellipse 0.78 × 1.02 | 0.72 | fur `#b8743a` |
| Belly | ellipse 0.5 × 0.72 | 0.10 | cream `#f6dcb4` |
| Head | ellipse 0.8 × 0.66 | 0.70 | fur |
| Ears | circles r 0.16 | 0.20 | dark fur `#8a5428` |
| Muzzle / nose | ellipses | 0.14 / 0.10 | cream / `#2a1a14` |
| Eyes + shine | ellipses | 0.06 / 0.02 | `#1a1210` / white |
| Cheeks | ellipses | 0.03 | blush `#e25d75` (booth crimson) |
| Whiskers | hairline slabs | 0.01 | dark fur |
| Arms | ellipse 0.16 × 0.42 on shoulder pivots | 0.26 | fur |
| Tail | tapered paddle (quadratic) | 0.22 | dark fur |
| Book | rect 1.0 × 0.64 + pages | 0.08 | cover `#e25d75`, pages paper |

Lighting: warm hemisphere, a key light from the upper right, and a **purple rim** (`#a78bfa`) from behind-left (Afterglow). The canvas is transparent and sits on a soft amber radial in the panel.

### Moods, props and poses (mapped to Folio events)

Every prop is built like his body: flat silhouettes (`Shape`) extruded into bevelled slabs and parented to a shoulder pivot or the head, so they move with him.

| Mood | Trigger | Pose / prop |
|---|---|---|
| chill · reading 🤓 | default | head tipped to the book, slow sway, tail wag, blinks |
| flipping 📖 | idle rotation | a page swings over the book's spine every 2.4 s |
| fishing 🎣 | idle rotation | rod from the right paw, line, bobber bobbing at the edge of the frame |
| tea 🍵 | idle rotation | cyan mug in the left paw, steam wisps, sips every 5 s |
| wave 👋 | panel opens, petted | right arm up, 9 Hz wave (2.4 s) |
| writing ✏️ | while typing in a text box | pencil (amber, pink eraser) scribbling over the book |
| checking 🔍 | export dialog / preflight | magnifying glass sweeping, head following |
| printing 🖨️ | export running | tiny printer in front of him, paper sliding out |
| party 🎉 | export done, walk-through finished | hop + spin, both arms up, party hat, confetti (3.2 s) |
| thumbsup 👍 | page checklist passes, guide step done | right arm up, thumb |
| stuck 🤔 | "I'm lost" with open issues | paw scratching the head, head tilt |
| nap 😴 | 45 s idle | **floats on his back** on a strip of water, eyes closed, slow bob. Any input wakes him |

Idle moods rotate (chill → flipping → fishing → tea) with 35% odds every 5 s after 15 s of quiet. Caption (mono, crimson): `jax · {emoji caption}`. We dropped "2.0" from his name in Folio.

Performance: one WebGL context, rendered only while visible (`IntersectionObserver`), paused when the window is hidden, and frozen under `prefers-reduced-motion`. It falls back to the box-drawn DS `Jax` if WebGL is unavailable. Target cost: < 0.5 ms/frame.

## Local help model (D-09): "Ask" via ONNX

**Goal:** answer free-form "how do I…?" questions grounded in Folio's own help library, offline, on modest hardware (her Windows 11 laptop).

### Pipeline (`folio-jax` crate, `ort` + `tokenizers`)

1. **Corpus:** `help/*.md`, short task articles (bleed, spine, page numbers, Kindle vs Kobo, importing from Word…). The library in `folio-data.js` (`HELP`) is the seed. Each chunk is 80–200 words and carries a `title`, `keywords` and an `actions` list (e.g. `open-export:paperback`).
2. **Retrieve:** a small sentence-embedding model (e.g. *all-MiniLM-L6-v2* or *bge-small-en-v1.5*, ONNX int8, ~25–35 MB) embeds the question. Cosine top-k (k=3) runs over the precomputed chunk embeddings shipped with the app, blended with BM25 to pick up exact terms like "KEPUB".
3. **Context:** current kind, mode, selection, spread checklist, and target. These are sent as structured hints, never as the document's text.
4. **Answer** (two tiers):
   - **Tier A (default, ships in v1):** extractive. Show the best chunk, lightly templated into Jax's voice, plus *Show me* buttons from its `actions`. This is fast, deterministic and cannot hallucinate.
   - **Tier B (experimental toggle, optional ~300–500 MB download):** a small instruct model (e.g. *Qwen2.5-0.5B-Instruct* or *SmolLM2-360M-Instruct*, ONNX int4) rephrases the retrieved chunks into a 2–3 sentence answer. The prompt is restricted to the retrieved text, the output is capped at 80 tokens, and the reply is refused if retrieval confidence is below threshold ("i'm not sure yet — try asking about…").
5. **Attribution:** every answer shows its source: "From Folio's help library · Bleed and trim · answered on this computer".

### Guardrails

- Jax never edits the book by himself. *Show me* only navigates or opens a dialog.
- No network: the model files are bundled or downloaded once from Liminal HQ's release assets, with checksums verified.
- Questions are not logged, unless the user opts into "Help improve Jax's library" (local file only, exportable).
- If the hardware is too slow (> 1.5 s retrieval on first run), stay on Tier A.

### Prototype

The prototype uses keyword scoring over `HELP` as a stand-in for retrieval (Tier A behaviour) so the flow can be judged before the model work starts.

### Built so far

The panel is in: a status-bar Jax button (and the Jax setting) opens a 470 px panel at the right end of the shelf with Tip, This page and Guide tabs. Tips follow what is selected or shown; the guide steps through the book with "Show me" buttons. **Ask** is a fourth tab (after This page): a question is matched by its words against a library of ten short task articles (bleed, spine, page numbers, Kindle and Kobo, importing, sending to Amazon, ISBN and barcode, checking the book, colour for print, undo), the best one is shown as written with a **Show me** that goes to what it talks about, and a question with no word Jax knows gets a plain "I don't know that one yet". That is tier A of D-09 (extractive, on this computer, nothing made up) with keyword scoring in place of the retrieval model; the articles live in `jax/helpArticles.ts`. There is no 3D model yet, and the otter is a placeholder emoji.
