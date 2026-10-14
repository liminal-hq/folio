# Ribbon (Liminal HQ component)

A data-driven ribbon toolbar for Liminal HQ desktop apps (Folio, Waypoint, future tools). It uses the Waypoint native-flex tokens (`--wp-*`), follows the system light or dark theme, and matches the Waypoint chrome and ContextMenu. The prototype lives in `Ribbon.dc.html`, with the showcase in `Ribbon Kit.dc.html`.

## Anatomy

| Part | Future component | Notes |
|---|---|---|
| Ribbon | `<Ribbon>` | Root: `role="toolbar"`. Owns the active tab, toggles, values, menus and tooltips |
| File button | `<RibbonFile>` | Accent-filled. Opens the app menu or backstage (`model.file.items`) |
| Tab | `<RibbonTab>` | `role="tab"`. Contextual tabs are accent-coloured and underlined, and appear only when listed in `contextual` |
| Mode switch | `<RibbonModeSwitch>` | Easy / Expert. In Easy, groups and items marked `expert` are hidden |
| Group | `<RibbonGroup>` | `role="group"`, label at the bottom, hairline divider on the right |
| Large button | `<RibbonButton size="large">` | 22 px glyph above a label. 58 px tall (48 px compact) |
| Small button | `<RibbonButton size="small">` | 14 px glyph + label, three per column (two in compact) |
| Row cluster | `<RibbonRow>` / `<RibbonCluster>` | Dense two-row layout for Font and Paragraph groups. Clusters are split by hairlines |
| Toggle | `kind: 'toggle'` | `aria-pressed`. A `radio` key makes a mutually exclusive set (alignment, lists) |
| Menu / split | `kind: 'menu'` | ▾ caret. Opens the shared ContextMenu (`palette="native"`). Colour items show swatches |
| Select | `kind: 'select'` | Native select, 26 px |
| Stepper | `kind: 'stepper'` | − value + (sizes, counts) |
| Swatch underline | `swatch` | 3 px bar under colour and highlight glyphs showing the current value |
| Tooltip | built in | 450 ms delay (60 ms when moving between buttons): name, shortcut chip, one-line description |

## Variants

- `variant`: `full` (default) · `simplified` (one row: tabs + icon buttons) · `collapsed` (tabs only; clicking drops the groups over the content; Pin keeps them open)
- `density`: `comfortable` (32 + 84 px) · `compact` (28 + 60 px, no group labels)
- `showLabels`: false gives an icon-only ribbon with tooltips
- `theme`: `dark` · `light` (`data-wp-theme`)

## Model

```ts
interface RibbonModel { file?: { label: string; items?: [value: string, label: string][] }; tabs: RibbonTab[] }
interface RibbonTab { id: string; label: string; contextual?: boolean; groups: RibbonGroup[] }
interface RibbonGroup { id: string; label: string; expert?: boolean; items?: RibbonItem[]; rows?: RibbonItem[][][] /* rows → clusters → items */ }
interface RibbonItem { id: string; label: string; d: string /* 16 px SVG path */; size?: 'large' | 'small'; kind?: 'button' | 'toggle' | 'menu' | 'select' | 'stepper';
  active?: boolean; radio?: string; disabled?: boolean; expert?: boolean; key?: string; desc?: string; swatch?: string;
  options?: [value: string, label: string][]; value?: string | number; width?: number; step?: number; min?: number; max?: number }
```

## Events

`onCommand({ id, value, item })` fires for every press, toggle (value = new state), menu pick (value = option), select and stepper change, `tab`, `mode` and `file`. The ribbon keeps its own toggle and value state, so it works uncontrolled. Apps that need control pass `active` and `value` in the model.

## Behaviour and accessibility

- Buttons pass `keepFocus` so text selections in an editor survive a click (`mousedown → preventDefault`).
- Keyboard (for the real component): Alt shows KeyTips; ←/→ between tabs; Tab into the panel; arrows within a group; Enter or Space activates; Esc returns to the tab.
- Every button has an accessible name. Toggles expose `aria-pressed`.
- Minimum 26 px targets (44 px in Larger controls mode).

## Building the component set

Split the DC into `Ribbon`, `RibbonTab`, `RibbonGroup`, `RibbonButton`, `RibbonRow`/`RibbonCluster`, `RibbonModeSwitch` and `RibbonTooltip` once a second app adopts it. Ship it in `@liminal-hq/waypoint-chrome` beside `WindowTitleBar` and `ContextMenu`. The model format stays the same.
