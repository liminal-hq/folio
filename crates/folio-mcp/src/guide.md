# Changing a book with intents

Every change to a book is an intent, a small JSON object with a `type`. Send a list of them to `apply_intents`: they are applied together, as one step the person can undo, or not at all if one is refused. The full schema is the `folio://schema/intent` resource; these are the ones used most.

Find the ids you need with `get_book` (spreads and frames) and `get_spread` (a frame's box and its words). A frame or guide lives on a spread or on a master page, so intents that name one say `"on": {"spread": "<spread id>"}` or `"on": {"master": "<master id>"}`. Sizes are in points (72 to the inch). Positions are in the spread's own space: the origin is the top left of its first page, and on a spread of two pages the right page starts one trim width in (a trim of 612 points puts the right page's left edge at x = 612, so a frame 36 points in from it is at x = 648). A page alone (the cover, a single page, an opener or a closer) starts at x = 0, whichever side of the book it is on. A frame that lies off its page is drawn nowhere, and preflight says its text is outside the margin.

## Move or resize a frame

```json
{
	"type": "MoveFrame",
	"on": { "spread": "sp_1" },
	"frame": "fr_1",
	"box": { "x": 36, "y": 36, "w": 200, "h": 40 }
}
```

## Put words in a text frame that holds its own words

```json
{ "type": "SetText", "frame": "fr_1", "text": "The otter who couldn't sleep" }
```

## Write a story that runs through linked frames

A story is the paragraphs of a flow (`flows` in the book). Replace paragraphs `at` to `at + remove` with new ones; give a paragraph a style such as `h1` (a chapter title), `h2` (a subheading) or `epigraph`, or leave it out for body text. Frames are added to the story's pages for you as the words grow.

```json
{
	"type": "SpliceStory",
	"flow": "fl_1",
	"at": 0,
	"remove": 0,
	"insert": [
		{ "style": "h1", "inline": [{ "text": "Chapter one" }] },
		{
			"inline": [
				{ "text": "It was a " },
				{ "text": "very", "italic": true },
				{ "text": " long night." }
			]
		}
	]
}
```

To change one paragraph's words and keep their look:

```json
{ "type": "SetParagraphText", "flow": "fl_1", "para": 1, "text": "It was a long night." }
```

## Add a frame

`AddFrame` takes a whole frame. These are a text frame (`style` names a paragraph style such as `body`, `display`, `byline`, `caption`; `get_book` lists the styles in use) and a picture frame (bring the file in with `import_picture` first). Give each frame a new id that no frame in the book has.

```json
{
	"type": "AddFrame",
	"on": { "spread": "sp_1" },
	"frame": {
		"type": "text",
		"id": "fr_note",
		"box": { "x": 36, "y": 36, "w": 300, "h": 80 },
		"rotation": 0,
		"locked": false,
		"style": "body",
		"content": { "inline": [{ "text": "Hello", "style": null, "bold": false, "italic": false }] },
		"columns": 1,
		"baselineLock": false
	}
}
```

```json
{
	"type": "AddFrame",
	"on": { "spread": "sp_1" },
	"frame": {
		"type": "image",
		"id": "fr_pic",
		"box": { "x": 36, "y": 130, "w": 300, "h": 240 },
		"rotation": 0,
		"locked": false,
		"description": "A fox asleep",
		"image": {
			"asset": "assets/images/abc123.png",
			"fit": "fit",
			"focus": [0.5, 0.5],
			"crop": null
		}
	}
}
```

## Add a spread after another

```json
{ "type": "AddSpread", "after": "sp_1", "duplicate": false }
```

## Remove a frame

```json
{ "type": "DeleteFrame", "on": { "spread": "sp_1" }, "frame": "fr_1" }
```

## Set the book's title and author

```json
{
	"type": "SetDetails",
	"details": {
		"title": "Pip and the Moon",
		"subtitle": null,
		"authors": [],
		"description": "",
		"isbn": {}
	}
}
```

## Put a picture in a picture frame

Bring the file in with `import_picture`, which answers with the asset's path, then:

```json
{ "type": "SetImage", "frame": "fr_2", "asset": "assets/images/abc123.png", "fit": "fill" }
```
