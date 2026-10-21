# Changing a book with intents

Every change to a book is an intent, a small JSON object with a `type`. Send a list of them to `apply_intents`: they are applied together, as one step the person can undo, or not at all if one is refused. The full schema is the `folio://schema/intent` resource; these are the ones used most.

Find the ids you need with `get_book` (spreads and frames) and `get_spread` (a frame's box and its words). A frame or guide lives on a spread or on a master page, so intents that name one say `"on": {"spread": "<spread id>"}` or `"on": {"master": "<master id>"}`. Sizes are in points (72 to the inch), measured from the top left of the page.

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
