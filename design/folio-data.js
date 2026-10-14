// Folio — sample books, styles, icons, toolbar models, help library.
// Loaded as a plain script; everything hangs off window.FOLIO.
(function () {
  const PAPER = '#fbfaf6';
  const TINT = { p: '#a78bfa', o: '#ffaa40', r: '#f43f5e', c: '#22d3ee', g: '#2ec66a' };
  const tint = (k) => `color-mix(in srgb, ${TINT[k] || TINT.p} 30%, ${PAPER})`;
  const STY = {
    display: { n: 'Cover title', pt: 40, wt: 600, al: 'center', lh: 1.08 },
    title: { n: 'Book title', pt: 28, wt: 600, al: 'center', lh: 1.1 },
    byline: { n: 'Byline', pt: 13, it: 1, al: 'center' },
    story: { n: 'Story text', pt: 19, lh: 1.32 },
    storyC: { n: 'Story, centred', pt: 21, lh: 1.32, al: 'center', it: 1 },
    label: { n: 'Label', pt: 9.5, f: 'sans', wt: 500, ls: 0.02 },
    h1: { n: 'Heading', pt: 20, wt: 600, lh: 1.15 },
    h1big: { n: 'Big heading', pt: 28, wt: 600, lh: 1.1 },
    h2: { n: 'Subheading', pt: 12.5, f: 'sans', wt: 600 },
    body: { n: 'Body', pt: 10.5, lh: 1.45, al: 'justify' },
    bodyL: { n: 'Body, ragged', pt: 10.5, lh: 1.45 },
    caption: { n: 'Caption', pt: 8.5, f: 'sans', lh: 1.35 },
    callout: { n: 'Note box', pt: 10, f: 'sans', lh: 1.45, box: 1 },
    epigraph: { n: 'Epigraph', pt: 10, it: 1, al: 'center', lh: 1.5 },
    poem: { n: 'Poem', pt: 11, lh: 1.6 },
    folio: { n: 'Page number', pt: 8, al: 'center' },
    rh: { n: 'Running head', pt: 7.5, f: 'sans', al: 'center', ls: 0.04 },
    spine: { n: 'Spine', pt: 11, wt: 600, al: 'center' },
    ornament: { n: 'Ornament', pt: 18, al: 'center', lh: 1 },
    numeral: { n: 'Chapter numeral', pt: 64, wt: 600, lh: 1 },
    bubble: { n: 'Bubble text', pt: 14, f: 'sans', wt: 500, al: 'center', lh: 1.2 },
    cell: { n: 'Table cell', pt: 9.5, f: 'sans', lh: 1.3 },
    footnote: { n: 'Footnote', pt: 8, lh: 1.35 },
  };
  const T = (id, x, y, w, h, s, text) => ({ id, t: 'text', x, y, w, h, s, text });
  const I = (id, x, y, w, h, tn, label) => ({ id, t: 'image', x, y, w, h, tint: tn, label });

  const N1 = "Mara had kept the light at Gannet Point for eleven winters, and in all that time the kettle had never once failed to sing. It was a dented thing, older than the lamp, and it sat on the iron stove like a hen that refused to be moved.\n\nOn the morning the letter came, it was quiet. She noticed the silence before she noticed the envelope: a square of cream paper tucked under the door, damp at one corner, addressed in a hand she had not seen since she was nineteen.";
  const N2 = "She did not open it straight away. There were the windows to wipe, and the brass to rub, and the log to fill in with the same small numbers she wrote every day: wind from the south-west, sea moderate, visibility good. Habit is a kind of shelter, her father used to say. You build it a brick at a time, and one day you find you are living inside it.\n\nBy noon the fog had come in thick enough to lean on. Mara lit the lamp early, as the rules allowed, and sat with the letter on her knee while the beam turned slow circles above her head.";
  const N3 = "Inside was a single line. I am coming home on the Thursday boat, if you will have me. No name, but there did not need to be one. Only one person had ever written sevens with a bar through the middle, and only one person had ever left without saying goodbye.\n\nShe read it four times. Then she got up, filled the old kettle, and set it on the stove — and for the first time that morning, it began, very softly, to sing.";

  const KINDS = {
    picture: { label: 'Picture book', sub: 'Fixed layout · pictures lead', title: "The Otter Who Couldn't Sleep", w: 8.5, h: 8.5, m: 0.5, b: 0.125, cols: 6, flow: false, pages: 32, start: 1, folios: false,
      styles: ['story', 'storyC', 'display', 'byline', 'caption'], trims: ['8.5 × 8.5 in', '8 × 10 in', '10 × 8 in', '8.5 × 11 in'],
      spreads: [
        { name: 'Cover', single: true, frames: [I('c1', -0.125, -0.125, 8.75, 8.75, 'p', 'Cover illustration — Pip on a lily pad, moonlight'), T('t1', 0.75, 0.85, 7, 2, 'display', "The Otter Who Couldn't Sleep"), T('t2', 0.75, 7.35, 7, 0.6, 'byline', 'Written and illustrated by Mom')] },
        { frames: [I('i1', -0.125, -0.125, 8.625, 8.75, 'c', 'Reeds at dusk, the river slowing'), T('p1', 9.25, 0.9, 6.75, 2.6, 'story', 'Down by the reeds, where the river slows to a whisper, lived a small otter named Pip.'), I('i2', 10.25, 4.1, 4.75, 3.6, 'o', 'Pip peeking out of the den')] },
        { frames: [I('i3', -0.125, -0.125, 17.25, 5.9, 'r', 'The moon rising over the whole river — runs across the gutter'), T('p2', 0.75, 6.3, 6.75, 1.6, 'story', 'Every night the moon rose, round as a river stone.'), T('p3', 9.25, 6.3, 6.75, 1.6, 'story', 'And every night, Pip lay awake, counting ripples.')] },
        { frames: [T('p4', 0.75, 2.8, 7, 2.4, 'storyC', '“Maybe,” said Grandmother Heron, “you are counting the wrong thing.”'), I('i4', 8.5, -0.125, 8.625, 8.75, 'g', 'Grandmother Heron, standing on one leg')] },
      ] },
    novel: { label: 'Novel', sub: 'Flowing text · reflows on e-readers', title: 'Tea at the Lighthouse', w: 5, h: 8, m: 0.625, b: 0, cols: 1, flow: true, pages: 248, start: 1, folios: true, rh: true, rhR: "The keeper's kettle",
      styles: ['body', 'h1', 'label', 'epigraph', 'title', 'byline'], trims: ['5 × 8 in', '5.25 × 8 in', '5.5 × 8.5 in', '6 × 9 in'],
      spreads: [
        { name: 'Title page', single: true, frames: [T('a1', 0.625, 2.2, 3.75, 1.6, 'title', 'Tea at the Lighthouse'), T('a2', 0.625, 3.9, 3.75, 0.4, 'byline', 'A novel'), T('a3', 0.625, 6.6, 3.75, 0.4, 'byline', 'Mom')] },
        { opener: true, frames: [T('e1', 0.875, 2.6, 3.25, 1.4, 'epigraph', '“The sea keeps its own hours, and so, in time, will you.”\n— from the keeper\'s log'), T('ch', 5.625, 1.5, 3.75, 0.35, 'label', 'Chapter one'), T('cht', 5.625, 1.85, 3.75, 0.7, 'h1', "The keeper's kettle"), T('b1', 5.625, 2.9, 3.75, 4.45, 'body', N1)] },
        { frames: [T('b2', 0.625, 0.75, 3.75, 6.6, 'body', N2), T('b3', 5.625, 0.75, 3.75, 6.6, 'body', N3)] },
      ] },
    textbook: { label: 'Textbook', sub: 'Grid, figures, callouts', title: 'Watercolour Basics', w: 8.5, h: 11, m: 0.75, b: 0.125, cols: 12, flow: false, pages: 96, start: 12, folios: true,
      styles: ['h1big', 'h1', 'h2', 'bodyL', 'caption', 'callout', 'label'], trims: ['8.5 × 11 in', '8 × 10 in', '7 × 10 in'],
      spreads: [
        { frames: [T('l', 0.75, 0.75, 7, 0.35, 'label', 'Lesson 3 · Wet on wet'), T('h', 0.75, 1.15, 7, 1.1, 'h1big', 'Letting colour move on its own'),
          T('b', 0.75, 2.5, 3.42, 4.6, 'bodyL', 'Wet-on-wet painting means laying colour onto paper that is already damp. The water does half the work: pigment blooms outward, edges soften, and two colours meet without a hard line between them.\n\nIt can feel like losing control. That is the point. You are setting up the conditions, then letting the paper answer.'),
          I('f', 4.33, 2.5, 3.42, 3.0, 'c', 'Figure 3.1 — a sky bloom, thirty seconds after the first wash'), T('cap', 4.33, 5.6, 3.42, 0.6, 'caption', 'Figure 3.1 · Ultramarine dropped into a damp sky. Notice the soft, feathered edge.'),
          T('tip', 0.75, 7.5, 7, 1.3, 'callout', 'Try this · Wet a postcard-sized sheet edge to edge. Wait until the shine just leaves the surface, then touch in one colour and watch for a full minute before adding another.'),
          T('h2', 9.25, 0.75, 7, 0.5, 'h2', "What you'll need"), T('list', 9.25, 1.3, 3.42, 3, 'bodyL', '• Cold-pressed paper, 300 gsm\n• A large round brush, size 12\n• Two jars of clean water\n• Ultramarine, burnt sienna, and a yellow you love'),
          I('f2', 12.83, 1.3, 3.42, 3.0, 'o', 'Photo — palette and brushes laid out'), T('h2b', 9.25, 4.8, 7, 0.5, 'h2', 'Step by step'),
          T('steps', 9.25, 5.35, 7, 3.6, 'bodyL', "1. Wet the whole sheet with clean water using the big brush.\n2. Tilt the board so a little water gathers at one edge.\n3. Load the brush with ultramarine and touch it in near the top.\n4. Let it travel. Resist the urge to tidy it.\n5. While it's still damp, drop in sienna low on the page for a horizon.")] },
        { frames: [I('f3', 0.75, 0.75, 7, 5, 'p', 'Three skies side by side — early, middle and late in the drying'), T('cap3', 0.75, 5.9, 7, 0.5, 'caption', 'Figure 3.2 · The same wash at three moments. Timing is everything.'), T('h3', 9.25, 0.75, 7, 0.6, 'h1', 'Common surprises'), T('b4', 9.25, 1.55, 7, 5, 'bodyL', 'Cauliflowers — those frilly blooms — happen when wetter paint meets drier paper. They are not mistakes; plenty of painters chase them on purpose.\n\nIf colour disappears as it dries, you are not alone. Watercolour lightens by about a third. Go a little bolder than feels right.')] },
      ] },
    illustrated: { label: 'Illustrated', sub: 'Cookbook, garden, craft', title: 'Recipes from the Lake House', w: 7, h: 10, m: 0.625, b: 0.125, cols: 6, flow: true, pages: 144, start: 22, folios: true,
      styles: ['h1big', 'h2', 'bodyL', 'caption', 'label'], trims: ['7 × 10 in', '8 × 10 in', '8.25 × 8.25 in'],
      spreads: [
        { frames: [I('ph', -0.125, -0.125, 7.125, 10.25, 'o', 'Photo — blueberry buckle, still warm from the oven'), T('lb', 7.625, 0.9, 5.75, 0.35, 'label', 'Summer · Baking'), T('tt', 7.625, 1.3, 5.75, 1, 'h1big', 'Blueberry buckle'), T('mt', 7.625, 2.35, 5.75, 0.4, 'caption', 'Serves 8 · 1 hour · Best the same day'),
          T('ih', 7.625, 3, 2.5, 0.4, 'h2', "You'll need"), T('ing', 7.625, 3.45, 2.5, 4.5, 'bodyL', '2 cups flour\n2 tsp baking powder\n½ tsp salt\n½ cup butter, soft\n¾ cup sugar\n1 egg\n½ cup milk\n2 cups blueberries'),
          T('mh', 10.5, 3, 2.875, 0.4, 'h2', 'Method'), T('me', 10.5, 3.45, 2.875, 5.8, 'bodyL', 'Heat the oven to 375 °F and butter a square pan. Whisk the flour, baking powder and salt. Cream the butter and sugar until pale, beat in the egg, then fold in the flour and milk in turns.\n\nFold in the berries, spread in the pan, and scatter the crumb on top. Bake 40 to 45 minutes, until a skewer comes out clean.')] },
        { frames: [T('t2', 0.625, 0.9, 5.75, 1, 'h1big', 'Lake tea, iced'), T('m2', 0.625, 1.95, 5.75, 0.4, 'caption', 'Makes one big jug · 10 minutes, plus chilling'), T('b2', 0.625, 2.6, 5.75, 4, 'bodyL', 'Steep four bags of black tea in two cups of just-boiled water for five minutes. Stir in honey while it is hot, then top up with cold water, lemon slices and a handful of mint from the porch pots.'), I('p2', 7, -0.125, 7.125, 10.25, 'c', 'Photo — the jug on the dock rail, lake behind')] },
      ] },
    poetry: { label: 'Poetry', sub: 'Generous white space', title: 'Small Hours', w: 5.5, h: 8.5, m: 0.875, b: 0, cols: 1, flow: true, pages: 80, start: 8, folios: true,
      styles: ['h1', 'poem', 'epigraph', 'caption'], trims: ['5.5 × 8.5 in', '5 × 8 in', '6 × 9 in'],
      spreads: [
        { frames: [T('pt1', 0.875, 1.2, 3.75, 0.6, 'h1', 'Porch light'), T('po1', 0.875, 2.1, 3.75, 5.2, 'poem', 'Someone left it on for me\nthe way you leave a word\nunsaid but kept —\n\nmoths at the glass\nlike small grey letters\nno one sent,\n\nand the step\nthat always creaks\nstill creaking,\nstill here.'),
          T('pt2', 6.375, 1.2, 3.75, 0.6, 'h1', 'Inventory'), T('po2', 6.375, 2.1, 3.75, 5.2, 'poem', 'Two cups, one chipped.\nA radio that only finds\nthe weather.\n\nThe coat you wore\nto every funeral\nand both weddings.\n\nThis house,\nwhich remembers you\nbetter than I do\nsome mornings.')] },
      ] },
    memoir: { label: 'Memoir with photos', sub: 'Text with photo spreads', title: 'The Year We Built the Porch', w: 6, h: 9, m: 0.75, b: 0.125, cols: 6, flow: true, pages: 212, start: 40, folios: true, rh: true, rhR: 'Summer, 1998',
      styles: ['body', 'caption', 'h1', 'label'], trims: ['6 × 9 in', '5.5 × 8.5 in', '7 × 10 in'],
      spreads: [
        { frames: [I('mp', -0.125, -0.125, 6.125, 6.3, 'g', 'Photo — the porch frame, summer 1998'), T('mc', 0.75, 6.5, 4.5, 0.7, 'caption', 'The frame went up in a single weekend. The roof took the rest of the summer.'),
          T('mb', 6.75, 0.85, 4.5, 7.4, 'body', "Your grandfather measured everything twice and still cut the first post too short. He laughed about it for years — said the porch had a lean because it was listening for us coming up the road.\n\nWe built it on weekends, with whatever lumber the yard had left over and a borrowed level that was never returned. The neighbours came by with lemonade and opinions, in roughly equal measure.")] },
      ] },
  };

  const IC = {
    undo: 'M5.5 3.5L2.5 6.5l3 3M2.5 6.5H10a3.5 3.5 0 010 7H7', redo: 'M10.5 3.5l3 3-3 3M13.5 6.5H6a3.5 3.5 0 000 7h3',
    text: 'M3 3.5h10M8 3.5v9.5M6 13h4', image: 'M2 3h12v10H2zM2 11l3.5-3.5 3 3 2-2L14 12M11 5a1 1 0 110 2 1 1 0 010-2z',
    shape: 'M2.5 4.5a2 2 0 012-2h7a2 2 0 012 2v5a2 2 0 01-2 2H7l-3 2.5v-2.5a2 2 0 01-1.5-2z',
    addPage: 'M4 1.5h5.5l3 3v10H4zM9.5 1.5v3h3M8.25 7.5v4.5M6 9.75h4.5', dupPage: 'M5.5 4h6.5v10.5H5.5zM3.5 12V1.5H10', delPage: 'M4 1.5h5.5l3 3v10H4zM6.5 8.5l3 3M9.5 8.5l-3 3',
    grid: 'M2 2h12v12H2zM6 2v12M10 2v12', margins: 'M2 2h12v12H2zM4.5 4.5h7v7h-7z',
    bleed: 'M4 4h8v8H4zM1.5 4H3M4 1.5V3M14.5 4H13M12 1.5V3M1.5 12H3M4 14.5V13M14.5 12H13M12 14.5V13', guides: 'M5 1.5v13M1.5 10.5h13',
    snap: 'M4 2.5v5a4 4 0 008 0v-5M4 5.5h3M9 5.5h3', baseline: 'M2 4h12M2 7h12M2 10h12M2 13h12', columns: 'M2 2.5h3.5v11H2zM6.25 2.5h3.5v11h-3.5zM10.5 2.5H14v11h-3.5z',
    thread: 'M6.5 9.5l3-3M5 8l-1.5 1.5a2.1 2.1 0 003 3L8 11M11 8l1.5-1.5a2.1 2.1 0 00-3-3L8 5', master: 'M3 1.5h10v13H3zM3 4.5h10M6 11.5h4',
    zoomIn: 'M7 2.5a4.5 4.5 0 110 9 4.5 4.5 0 010-9zM10.5 10.5l3.5 3.5M5 7h4M7 5v4', zoomOut: 'M7 2.5a4.5 4.5 0 110 9 4.5 4.5 0 010-9zM10.5 10.5l3.5 3.5M5 7h4', fit: 'M2 6V2h4M10 2h4v4M14 10v4h-4M6 14H2v-4',
    print: 'M4.5 6V2h7v4M4.5 11.5H2V6h12v5.5h-2.5M4.5 9h7v5h-7z', ereader: 'M4 1.5h8v13H4zM6 4h4M6 6h4M6 8h3M7 12.5h2',
    export: 'M8 10V2M5 5l3-3 3 3M3 9v5h10V9', check: 'M8 1.5a6.5 6.5 0 110 13 6.5 6.5 0 010-13zM5 8.2l2 2 4-4.2',
    bold: 'M4.5 2.5h4.25a2.75 2.75 0 010 5.5H4.5zM4.5 8h5a3 3 0 010 6h-5z', italic: 'M7 2.5h6M3 13.5h6M10 2.5L6 13.5',
    alignL: 'M2 3h12M2 6.5h8M2 10h12M2 13.5h8', alignC: 'M2 3h12M4 6.5h8M2 10h12M4 13.5h8', alignR: 'M2 3h12M6 6.5h8M2 10h12M6 13.5h8', justify: 'M2 3h12M2 6.5h12M2 10h12M2 13.5h12',
    front: 'M6 6h8v8H6zM2 10V2h8', back: 'M2 2h8v8H2zM6 14h8V6', book: 'M3 2.5h8a2 2 0 012 2v9.5H5a2 2 0 01-2-2zM3 11.5a2 2 0 012-2h8',
    open: 'M2 4h4l1.5 1.5H14v8H2z', save: 'M3 2h8l3 3v8a1 1 0 01-1 1H3a1 1 0 01-1-1V3a1 1 0 011-1zM5 2v4h5V2M5 10h6v4H5z',
    importDoc: 'M8 2v8M5 7l3 3 3-3M3 11v3h10v-3', cover: 'M3.5 1.5h9v13h-9zM6 5h4M6 7.5h4', toc: 'M2 3.5h1.5M5.5 3.5H14M2 7.5h1.5M5.5 7.5H14M2 11.5h1.5M5.5 11.5H14',
    chapter: 'M3 2.5h10M3 5.5h6M2 9.5h12M2 12.5h12', jax: 'M8 1.5a6.5 6.5 0 110 13 6.5 6.5 0 010-13zM6.2 6.2a1.9 1.9 0 113 1.5c-.6.4-1.2.8-1.2 1.6M8 11.3v.2',
    ribbon: 'M1.5 2.5h13v4h-13zM1.5 9h13M1.5 12h13', ruler: 'M1.5 5h13v6h-13zM4 5v2.5M6.5 5v1.5M9 5v2.5M11.5 5v1.5', rguides: 'M5 1.5v13M1.5 10.5h13M1.5 1.5h3v3', rulerUnits: 'M1.5 5h13v6h-13zM4 5v2.5M6.5 5v1.5M9 5v2.5M11.5 5v1.5', rgClear: 'M5 1.5v13M1.5 10.5h13M10 3l4 4M14 3l-4 4', barcode: 'M2 3v10M4 3v10M5.5 3v10M8 3v10M9.5 3v10M12 3v10M14 3v10', menubar: 'M1.5 2.5h13v11h-13zM1.5 5.5h13M3.5 4h2M7 4h2', newBook: 'M3 2.5h8a2 2 0 012 2v9.5H5a2 2 0 01-2-2zM8 5v4M6 7h4',
    trash: 'M3 4h10M6 4V2.5h4V4M4.5 4l.75 10h5.5L11.5 4', settings: 'M8 5.5a2.5 2.5 0 110 5 2.5 2.5 0 010-5zM8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4',
    keyboard: 'M1.5 4h13v8h-13zM4 6.5h1M7 6.5h1M10 6.5h2M4 9.5h8', info: 'M8 1.5a6.5 6.5 0 110 13 6.5 6.5 0 010-13zM8 7v4.5M8 4.6v.4', quit: 'M6 2.5H3v11h3M10 5l3 3-3 3M13 8H6',
  };
  Object.assign(IC, {
    underline: 'M4.5 2.5v5a3.5 3.5 0 007 0v-5M3.5 14h9',
    strike: 'M3 8h10M10.8 4.6C10.3 3.5 9.3 3 8 3 6.3 3 5 3.9 5 5.2c0 .9.5 1.5 1.4 1.9M5.2 11.2c.5 1.1 1.6 1.8 3 1.8 1.8 0 3-.9 3-2.3 0-.5-.1-.9-.4-1.2',
    smallcaps: 'M2 4.5h6M5 4.5v8M9.5 7.5h4.5M11.75 7.5v5',
    sup: 'M2 5l5 7M7 5l-5 7M10 2.5c.3-.6.9-1 1.6-1 .9 0 1.4.6 1.4 1.3 0 1.1-1.5 1.7-3 3.2h3',
    sub: 'M2 3.5l5 7M7 3.5l-5 7M10 10.5c.3-.6.9-1 1.6-1 .9 0 1.4.6 1.4 1.3 0 1.1-1.5 1.7-3 3.2h3',
    colour: 'M4 12L7.5 2.5h1L12 12M5.3 8.5h5.4M2.5 14.5h11',
    highlight: 'M9.5 2.5l4 4L8 12H4V8zM2.5 14.5h11',
    clear: 'M3 3.5h8M7 3.5L5 12.5M10 9l4 4M14 9l-4 4',
    bullets: 'M5.5 4h8M5.5 8h8M5.5 12h8M2.5 4h.5M2.5 8h.5M2.5 12h.5',
    numbers: 'M6 4h8M6 8h8M6 12h8M2 3l1-.5V5.5M2 10.5c.2-.4.6-.6 1-.6.5 0 .9.3.9.8 0 .7-1 1.1-1.9 2h2',
    indentMore: 'M2 3h12M7 6.5h7M7 10h7M2 13.5h12M2 6l2.5 2.25L2 10.5',
    indentLess: 'M2 3h12M7 6.5h7M7 10h7M2 13.5h12M4.5 6L2 8.25l2.5 2.25',
    firstIndent: 'M6 3h8M2 6.5h12M2 10h12M2 13.5h9',
    lineSpacing: 'M7 3.5h7M7 8h7M7 12.5h7M3.5 2v12M2 3.5L3.5 2 5 3.5M2 12.5L3.5 14 5 12.5',
    paraSpacing: 'M2 2.5h12M2 5h12M2 11h12M2 13.5h12M8 6.5v3.5',
    dropCap: 'M2 2.5h5v5H2zM9 3h5M9 6h5M2 10h12M2 13h12',
    tracking: 'M2 12L4.5 4h1L8 12M2.8 9.5h4.4M10 8h4M12.5 6.5L14 8l-1.5 1.5M11.5 6.5L10 8l1.5 1.5',
    find: 'M7 2.5a4.5 4.5 0 110 9 4.5 4.5 0 010-9zM10.5 10.5l3.5 3.5',
    spell: 'M2 10L4.5 3h1L8 10M2.8 7.8h4.4M8.5 11.5l2 2 4-4.5',
    link: 'M6.5 9.5l3-3M5 8l-1.5 1.5a2.1 2.1 0 003 3L8 11M11 8l1.5-1.5a2.1 2.1 0 00-3-3L8 5',
    rect: 'M2.5 3.5h11v9h-11z',
    round: 'M5 3.5h6a2.5 2.5 0 012.5 2.5v4a2.5 2.5 0 01-2.5 2.5H5A2.5 2.5 0 012.5 10V6A2.5 2.5 0 015 3.5z',
    ellipse: 'M8 3.5c3.3 0 5.5 2 5.5 4.5S11.3 12.5 8 12.5 2.5 10.5 2.5 8 4.7 3.5 8 3.5z',
    line: 'M2.5 13.5l11-11',
    speech: 'M4 2.5h8a2 2 0 012 2v4a2 2 0 01-2 2H7l-3 3v-3a2 2 0 01-2-2v-4a2 2 0 012-2z',
    thought: 'M8 2.5c3 0 5.5 1.6 5.5 3.75S11 10 8 10 2.5 8.4 2.5 6.25 5 2.5 8 2.5zM4.5 12a1 1 0 110 2 1 1 0 010-2z',
    shout: 'M8 1.5l1.5 3 3.5-.5-1.5 3 3 2-3.5.5.5 3.5-3-2-2.5 3-.5-3.5-3.5.5 2-3-3-2 3.5-.5-1-3.5z',
    table: 'M2 2.5h12v11H2zM2 6h12M2 9.75h12M6.5 2.5v11',
    ornament: 'M8 3c-2 0-3.5 1.5-3.5 3.5 0 1.5 1 2.5 2 2.5S8 8 8 7M8 3c2 0 3.5 1.5 3.5 3.5 0 1.5-1 2.5-2 2.5S8 8 8 7M8 7v6.5M5 13.5h6',
    footnote: 'M2 3h9M2 6h9M2 9h6M2 13h12M12 2.5v4',
    glossary: 'M3 2.5h8.5a1 1 0 011 1v11H4a1 1 0 01-1-1zM5.5 6h4.5M5.5 8.5h3',
    indexEntry: 'M3 3h2M3 6.5h2M3 10h2M7 3h6M7 6.5h6M7 10h6M3 13.5h10',
    templates: 'M2 2.5h5.5v5.5H2zM9 2.5h5v3H9zM9 7h5v6.5H9zM2 9.5h5.5v4H2z',
    background: 'M2 2.5h12v11H2zM2 10l3.5-3 3 2.5 2-1.5L14 11',
    crop: 'M4.5 1.5v10h10M1.5 4.5h10v10',
    fitFill: 'M2 2h12v12H2zM5 5h6v6H5z',
    rotL: 'M3 6.5A5 5 0 1 1 4 11M3 2.5v4h4',
    rotR: 'M13 6.5A5 5 0 1 0 12 11M13 2.5v4H9',
    flipH: 'M8 1.5v13M6 4L2.5 12H6zM10 4l3.5 8H10z',
    flipV: 'M1.5 8h13M4 6L12 2.5V6zM4 10l8 3.5V10z',
    border: 'M2 2h12v12H2zM4.5 4.5h7v7h-7z',
    corners: 'M2.5 13.5V7a4.5 4.5 0 014.5-4.5h6.5',
    filter: 'M6 4.5a3.5 3.5 0 110 7 3.5 3.5 0 010-7zM10 4.5a3.5 3.5 0 110 7 3.5 3.5 0 010-7z',
    wrap: 'M2 3h12M2 6.5h5M2 10h5M2 13.5h12M9 5.5h5v6H9z',
    replace: 'M2 3h12v10H2zM10 6l2 2-2 2M12 8H6',
    alignLeftE: 'M2.5 1.5v13M4.5 4h8v3h-8zM4.5 9h5v3h-5z',
    alignCenterE: 'M8 1.5v13M3 4h10v3H3zM5 9h6v3H5z',
    alignRightE: 'M13.5 1.5v13M3.5 4h8v3h-8zM6.5 9h5v3h-5z',
    alignTopE: 'M1.5 2.5h13M4 4.5v8h3v-8zM9 4.5v5h3v-5z',
    alignMidE: 'M1.5 8h13M4 3v10h3V3zM9 5v6h3V5z',
    alignBotE: 'M1.5 13.5h13M4 3.5v8h3v-8zM9 6.5v5h3v-5z',
    distH: 'M2 2v12M14 2v12M6 5h4v6H6z',
    distV: 'M2 2h12M2 14h12M5 6h6v4H5z',
    group: 'M2 2h5v5H2zM9 9h5v5H9zM1 1h1M14 1h1M1 15h1M14 15h1',
    ungroup: 'M2 2h5v5H2zM9 9h5v5H9z',
    lock: 'M4 7h8v6.5H4zM5.5 7V5a2.5 2.5 0 015 0v2',
    eye: 'M1.5 8S4 3.5 8 3.5 14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8zM8 6a2 2 0 110 4 2 2 0 010-4z',
    layers: 'M8 2l6 3-6 3-6-3zM2 8l6 3 6-3M2 11l6 3 6-3',
    popout: 'M9 2.5h4.5V7M13.5 2.5L7.5 8.5M11.5 9.5v4h-9v-9h4',
    search: 'M7 2.5a4.5 4.5 0 110 9 4.5 4.5 0 010-9zM10.5 10.5l3.5 3.5',
    rowAdd: 'M2 2.5h12v4H2zM2 9.5h7M2 13.5h7M12 9v5M9.5 11.5h5',
    rowDel: 'M2 2.5h12v4H2zM2 9.5h7M2 13.5h7M9.5 11.5h5',
    colAdd: 'M2.5 2v12h4V2zM9.5 2v7M13.5 2v7M12 9.5v5M9.5 12h5',
    colDel: 'M2.5 2v12h4V2zM9.5 2v7M13.5 2v7M9.5 12h5',
    headRow: 'M2 2.5h12v11H2zM2 6h12',
    fill: 'M3 9l5-5 5 5-5 5zM12.5 11.5c0 1 .7 2 1.5 2s1.5-1 1.5-2-1.5-2.5-1.5-2.5',
    stroke: 'M2 13.5c3-1 4-8 6-8s2 6 6-2',
    tabs: 'M1.5 4.5h4l1-2h3l1 2h4v9h-13z',
    sidebar: 'M1.5 2.5h13v11h-13zM10 2.5v11',
    toolbarI: 'M1.5 2.5h13v11h-13zM1.5 6h13',
  });
  const ICON_FOR = { askJax: 'jax', importText: 'importDoc', importLayout: 'open', pageSize: 'book', kindle: 'ereader', kobo: 'ereader', ribbonToggle: 'ribbon', rulers: 'ruler', quickPdf: 'export', preflight: 'check', marginsT: 'margins', toolbarLayout: 'toolbarI', sidebarLayout: 'sidebar', shapeKind: 'round', tableStyle: 'table', headToggle: 'headRow', pageBg: 'background' };
  const RIBBON = {
    home: [{ g: 'History', items: [['undo', 'Undo'], ['redo', 'Redo']] }, { g: 'Font', special: 'font', items: [['bold', 'Bold'], ['italic', 'Italic'], ['underline', 'Underline'], ['strike', 'Strikethrough'], ['smallcaps', 'Small caps'], ['sup', 'Superscript'], ['sub', 'Subscript'], ['colour', 'Text colour', 0, 0, 'menu'], ['highlight', 'Highlight', 0, 0, 'menu'], ['clear', 'Clear formatting']] },
      { g: 'Paragraph', items: [['alignL', 'Left'], ['alignC', 'Centre'], ['alignR', 'Right'], ['justify', 'Justify'], ['bullets', 'Bullets'], ['numbers', 'Numbering'], ['indentLess', 'Outdent'], ['indentMore', 'Indent'], ['firstIndent', 'First-line indent'], ['lineSpacing', 'Line spacing', 0, 0, 'menu'], ['paraSpacing', 'Space after', 0, 0, 'menu'], ['dropCap', 'Drop cap'], ['tracking', 'Letter spacing', 0, 1, 'menu']] },
      { g: 'Editing', items: [['find', 'Find & replace', 1], ['spell', 'Spelling']] }, { g: 'Help', items: [['askJax', 'Ask Jax', 1]] }],
    insert: [{ g: 'Frames', items: [['text', 'Text box', 1], ['image', 'Picture', 1], ['table', 'Table', 1]] }, { g: 'Shapes', items: [['rect', 'Rectangle'], ['round', 'Rounded'], ['ellipse', 'Ellipse'], ['line', 'Line'], ['shape', 'Note box']] }, { g: 'Bubbles', items: [['speech', 'Speech'], ['thought', 'Thought'], ['shout', 'Shout']] },
      { g: 'Decorate', items: [['ornament', 'Ornament', 1, 0, 'menu']] }, { g: 'Book parts', items: [['cover', 'Cover', 1], ['chapter', 'Chapter break'], ['toc', 'Contents'], ['footnote', 'Footnote'], ['barcode', 'ISBN barcode'], ['glossary', 'Glossary term', 0, 1], ['indexEntry', 'Index entry', 0, 1]] }, { g: 'Templates', items: [['templates', 'Spread templates', 1]] }, { g: 'Bring in', items: [['importText', 'Import text', 1], ['importLayout', 'Import layout', 1, 1]] }],
    picture: [{ g: 'Picture', items: [['replace', 'Replace', 1], ['fitFill', 'Fit / fill'], ['crop', 'Crop'], ['toBleed', 'To bleed']] }, { g: 'Rotate', items: [['rotL', 'Rotate left'], ['rotR', 'Rotate right'], ['flipH', 'Flip across'], ['flipV', 'Flip down']] }, { g: 'Look', items: [['border', 'Border', 1, 0, 'menu'], ['corners', 'Corners', 1, 0, 'menu'], ['filter', 'Filter', 1, 0, 'menu']] }, { g: 'Text', items: [['wrap', 'Wrap text', 1, 0, 'menu']] }],
    shape: [{ g: 'Shape', items: [['fill', 'Fill', 1, 0, 'menu'], ['stroke', 'Outline', 1, 0, 'menu'], ['shapeKind', 'Change shape', 1, 0, 'menu']] }, { g: 'Rotate', items: [['rotL', 'Rotate left'], ['rotR', 'Rotate right']] }, { g: 'Text', items: [['wrap', 'Wrap text', 1, 0, 'menu']] }],
    table: [{ g: 'Rows', items: [['rowAdd', 'Add row'], ['rowDel', 'Remove row']] }, { g: 'Columns', items: [['colAdd', 'Add column'], ['colDel', 'Remove column']] }, { g: 'Look', items: [['headToggle', 'Header row', 1], ['tableStyle', 'Table style', 1, 0, 'menu']] }],
    pages: [{ g: 'Pages', items: [['addPage', 'Add pages', 1], ['dupPage', 'Duplicate'], ['delPage', 'Remove']] }, { g: 'Templates', items: [['templates', 'Spread templates', 1]] }, { g: 'Background', items: [['pageBg', 'Page background', 1, 0, 'menu']] }, { g: 'Book', items: [['cover', 'Cover', 1], ['pageSize', 'Book size', 1], ['marginsT', 'Margins']] }, { g: 'Masters', x: 1, items: [['master', 'Master pages', 1], ['thread', 'Link text']] }],
    layout: [{ g: 'Rulers', items: [['rulers', 'Rulers', 1], ['rguides', 'Guides'], ['rulerUnits', 'Units', 0, 0, 'menu'], ['rgClear', 'Clear guides']] }, { g: 'Helpers', items: [['margins', 'Margins'], ['grid', 'Grid'], ['bleed', 'Bleed'], ['guides', 'Frame edges']] }, { g: 'Snapping', items: [['snap', 'Snap', 1]] }, { g: 'Arrange', items: [['front', 'To front'], ['back', 'To back'], ['group', 'Group'], ['ungroup', 'Ungroup'], ['lock', 'Lock']] },
      { g: 'Align', items: [['alignLeftE', 'Left edges'], ['alignCenterE', 'Centres'], ['alignRightE', 'Right edges'], ['alignTopE', 'Tops'], ['alignMidE', 'Middles'], ['alignBotE', 'Bottoms']] }, { g: 'Distribute', items: [['distH', 'Across'], ['distV', 'Down']] }, { g: 'Type grid', x: 1, items: [['baseline', 'Baseline grid'], ['columns', 'Columns']] }],
    view: [{ g: 'Preview', items: [['print', 'Print', 1], ['kindle', 'Kindle', 1], ['kobo', 'Kobo', 1]] }, { g: 'Zoom', items: [['zoomIn', 'Zoom in'], ['zoomOut', 'Zoom out'], ['fit', 'Fit']] }, { g: 'Interface', items: [['toolbarLayout', 'Toolbar layout', 1, 0, 'menu'], ['sidebarLayout', 'Sidebar layout', 1, 0, 'menu'], ['menubar', 'Menu bar']] }],
    export: [{ g: 'Check', items: [['preflight', 'Check book', 1]] }, { g: 'Export', items: [['export', 'Export book', 1], ['quickPdf', 'Quick PDF']] }],
  };
  const TABS = [['home', 'Home'], ['insert', 'Insert'], ['pages', 'Pages'], ['layout', 'Layout'], ['view', 'View'], ['export', 'Export']];
  const CLASSIC_A = [[['newBook', 'New book'], ['open', 'Open'], ['save', 'Save']], [['undo', 'Undo'], ['redo', 'Redo']], [['text', 'Text box', 1], ['image', 'Picture', 1], ['shape', 'Note box']]];
  const CLASSIC_B = [[['bold', 'Bold'], ['italic', 'Italic'], ['alignL', 'Left'], ['alignC', 'Centre'], ['alignR', 'Right']], [['front', 'To front'], ['back', 'To back']], [['margins', 'Margins'], ['grid', 'Grid'], ['bleed', 'Bleed'], ['snap', 'Snap']], [['print', 'Print'], ['kindle', 'Kindle'], ['kobo', 'Kobo']], [['zoomOut', 'Zoom out'], ['fit', 'Fit'], ['zoomIn', 'Zoom in']], [['ribbonToggle', 'Switch to the ribbon']]];

  const HELP = [
    { t: 'Bleed and trim', k: ['bleed', 'edge', 'trim', 'white', 'sliver', 'cut', 'pink'], a: "Pictures that touch the page edge should run past it to the pink dashed line — that's 0.125 in of bleed. The printer trims there, so you never get a white sliver." },
    { t: 'Spine width', k: ['spine', 'thick', 'cover', 'paperback', 'hardcover'], a: 'The spine grows with your page count and paper — about 0.0023 in per page on white paper. Export → Paperback shows the exact width and draws the whole cover for you.' },
    { t: 'Page numbers', k: ['number', 'numbers', 'folio', 'footer', 'header', 'running'], a: 'Page numbers live on master page A, so every page gets one and they renumber themselves. Picture books usually leave them off — switch them on under Pages → Master pages (Expert).' },
    { t: 'Kindle and Kobo', k: ['kindle', 'kobo', 'ebook', 'e-reader', 'epub', 'reflow', 'reader'], a: 'Flowing books become EPUBs that reflow — readers pick their own text size. Picture books export as fixed layout, so every page stays exactly as you designed it.' },
    { t: 'Picture quality', k: ['picture', 'image', 'photo', 'blurry', 'resolution', 'ppi', 'dpi', 'quality', 'sharp'], a: "For print, pictures look sharp at 300 pixels per inch at the size they're placed. Folio checks every picture before export and tells you which ones are too small." },
    { t: 'Importing your words', k: ['word', 'docx', 'import', 'google', 'docs', 'paste', 'odt', 'rtf', 'markdown', 'scribus', 'idml', 'indesign', 'affinity'], a: 'Insert → Import text takes Word, Google Docs, LibreOffice, Pages/RTF and Markdown — headings become chapters, bold and italics come along. Import layout (Expert) opens Scribus, IDML and EPUB.' },
    { t: 'Moving things neatly', k: ['snap', 'align', 'line', 'guide', 'move', 'drag', 'straight', 'neat'], a: 'Drag a box and thin cyan lines appear when it lines up with a margin, the centre or another box. Hold Alt to move freely. Arrow keys nudge; Shift moves ten times further.' },
    { t: 'Undo', k: ['undo', 'mistake', 'oops', 'wrong'], a: 'Ctrl+Z undoes and Ctrl+Shift+Z puts it back. Folio also keeps a recovery copy inside your .folio file every few minutes.' },
    { t: 'Adding pages', k: ['add', 'new', 'pages', 'more', 'extra', 'reorder', 'order'], a: "Pages → Add pages puts a new spread after the one you're on. Drag the little pages in the shelf to change their order." },
    { t: 'Text styles', k: ['font', 'typeface', 'size', 'bigger', 'smaller', 'style', 'heading'], a: 'Pick a style — Story text, Heading, Caption — in the side panel. Smaller and Bigger adjust just this box; changing the style itself updates every box that uses it.' },
    { t: 'Sending it to Amazon', k: ['amazon', 'kdp', 'publish', 'upload', 'export', 'send', 'print', 'printer'], a: "Export book → tick Paperback and Kindle. You'll get an interior PDF, a cover PDF and an EPUB — the three files KDP asks for." },
  ];
  const TIPS = [
    "psst — right-click anything on the page for more options. even me. actually not me, i'm ticklish.",
    "double-click a text box to type straight onto the page. it's like writing in the margins, but allowed.",
    'hold alt while dragging if the guides are being a bit too helpful.',
    'things that line up look calm, even when nobody can say why. the cyan lines are me helping.',
    'you can drag the little pages in the shelf to put them in a different order.',
    "i save a recovery copy inside your .folio file every few minutes. i'm very organised. for an otter.",
  ];
  const GUIDE_FIXED = [
    ['Name your book', 'Click the title on the cover and type your own. The byline under it is yours too.', 'goCover'],
    ['Tell the story', 'Double-click a text box on each page to type straight onto it.', 'goPage1'],
    ['Add your pictures', 'Drop each illustration onto a picture frame. Edge-to-edge ones should reach the pink dashed line.', 'goPage1'],
    ['Check every page', "Open the This page tab and I'll point out anything worth a look.", 'pageTab'],
    ['Preview on e-readers', 'See how the pages turn on a Kindle or Kobo before anyone else does.', 'kindle'],
    ['Send it out', 'Export book makes the KDP and Kobo files in one go, cover included.', 'export'],
  ];
  const GUIDE_FLOW = [
    ['Bring in your words', 'Insert → Import text takes Word, Google Docs, LibreOffice or Markdown. Chapters are found for you.', 'importText'],
    ['Pick your styles', 'Click a heading and choose a style in the side panel — every chapter follows along.', 'goPage1'],
    ['Check the opening pages', 'Title page first, then chapter one starting on a right-hand page.', 'goPage1'],
    ['Preview on Kindle and Kobo', 'Readers choose their own text size, so the words reflow. Have a look.', 'kindle'],
    ['Check the cover', 'Export → Paperback draws your full cover with the spine measured for you.', 'export'],
    ['Send it out', 'Export book makes everything KDP and Kobo need in one go.', 'export'],
  ];
  // KDP paperback spine, inches per page (KDP Print cover calculator). Hardcover uses KDP's lookup table — see spec/05-export.md.
  const PER_PAGE = { white: 0.002252, cream: 0.0025, colour: 0.002347 };

  window.FOLIO = { PAPER, TINT, tint, STY, KINDS, KIND_ORDER: ['picture', 'novel', 'textbook', 'illustrated', 'poetry', 'memoir'], IC, ICON_FOR, RIBBON, TABS, CLASSIC_A, CLASSIC_B, HELP, TIPS, GUIDE_FIXED, GUIDE_FLOW, PER_PAGE };
})();
