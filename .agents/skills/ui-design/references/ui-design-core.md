# UI design core

General principles for executing an interface. Numbers are defaults to start from and adjust in the rendered result. A project's existing scale overrides them.

## Contents

1. Visual hierarchy
2. Typography
3. Spacing
4. Layout and grids
5. Color and contrast
6. Surfaces and depth
7. Components and states
8. Tokens and design systems
9. Motion
10. Accessibility
11. Responsive behavior

## 1. Visual hierarchy

Before styling, rank the content of the view: what is read first, second, third, and what is only there for reference. Most views have three levels; more than four is usually a sign that the view is doing too many jobs.

Signals, roughly from strongest to weakest: size, weight, color contrast against the background, surrounding space, position (top and left in LTR), isolation, then decoration (borders, backgrounds, icons).

- Make a level with two or three signals together, and make the step obvious. 16px next to 17px is noise; 16px regular next to 20px semibold is a level.
- De-emphasize instead of emphasizing. When everything competes, lower the secondary content (smaller, lighter color, regular weight) before making the primary louder.
- Labels are usually less important than their values. In "Status: Active" the value carries the emphasis; the label can be small and muted or dropped entirely when context makes it obvious.
- One primary button per view: filled, accent color. Secondary: outline or neutral fill. Tertiary: text only. A destructive action is not automatically primary; it is styled as destructive only where it is the main action of a confirmation.
- Weight and color are cheaper than size. A heading can stay the same size as body text and still lead through weight and a darker color.
- Space is a hierarchy signal. An element with more empty space around it reads as more important.

Test: blur the screenshot or squint. The order in which blobs appear should match the ranking you wrote down.

## 2. Typography

### Families

One family is enough for most product UI. Two is the maximum for most work: one for display, one for text, and they should differ clearly (a serif with a sans, a condensed with a regular width). Two similar sans-serifs look like an error.

Choose the text face for its behavior at 14 to 16px: open apertures, distinguishable `I l 1` and `O 0`, a real italic, the weights you need, tabular figures. Check that the font covers the languages the product ships in; Portuguese needs `ã õ ç á é í ó ú â ê ô à`.

System font stacks are a legitimate choice for tools and dashboards: fast, familiar, no layout shift.

### Scale

Pick a base and a ratio, generate the steps, then round to whole pixels and stop. Around 6 to 8 sizes cover an entire product.

- Dense product UI: base 14px, ratio about 1.2 (12, 14, 16, 20, 24, 30).
- General UI and content: base 16px, ratio 1.25 (12, 14, 16, 20, 25, 31, 39).
- Marketing and editorial: base 16 to 18px, ratio 1.333 or larger, with a display size well above the rest (48 to 96px and up).

Define sizes in `rem` so user font-size settings apply. Do not set body text below 14px on desktop or 16px on mobile inputs (iOS Safari zooms the page on focus for inputs under 16px).

### Line height, measure, tracking

- Line height is inversely related to size: body 1.45 to 1.6, UI labels 1.2 to 1.4, headings 1.05 to 1.25. Long lines need more leading, short lines less.
- Measure: 45 to 75 characters for running text. Set it with `max-width: 65ch` on the text block, not on the page.
- Tracking: tighten large display type slightly (-0.01 to -0.03em); leave body text alone; open up all-caps and very small text (+0.04 to +0.08em).
- Left-align body text. Center only short blocks (up to about three lines). Avoid justified text on the web.

### Details that separate careful from careless

```css
h1, h2, h3 { text-wrap: balance; }
p { text-wrap: pretty; }
.numeric, td.num { font-variant-numeric: tabular-nums; }
```

- Weights: regular for text, medium or semibold for emphasis and headings. Avoid weights under 400 for text below about 20px.
- Use real typographic characters: curly quotes, en dash for ranges, ellipsis character, multiplication sign for dimensions.
- Right-align numeric columns and give them tabular figures so digits line up.
- Truncation is a decision. Decide per field between wrap, clamp to N lines, or ellipsis with the full value reachable (title, tooltip, detail view).
- Load fonts with `font-display: swap` and a metric-compatible fallback to limit layout shift; preload only the one or two files used above the fold.

## 3. Spacing

Use one scale for every margin, padding and gap. A 4px base works for nearly everything:

`4, 8, 12, 16, 24, 32, 48, 64, 96, 128`

The steps grow as they get larger because the eye compares ratios, not absolute differences: 4 versus 8 is visible, 64 versus 68 is not.

- Proximity is the rule that matters most. Space inside a group is smaller than the space between groups, which is smaller than the space between sections. A heading sits closer to the text it introduces than to the text above it.
- Inner less than or equal to outer: padding inside a container should not exceed the gap between that container and its siblings by much, or the grouping reverses.
- Start with more space than seems necessary and remove it. Cramped layouts are the commoner failure.
- Prefer `gap` on flex and grid parents over margins on children. It removes the first-child and last-child exceptions.
- Density is a product decision. Tools used all day (tables, editors, admin panels) want compact rows (32 to 40px) and tight padding; marketing and reading surfaces want generous section spacing (96px and up on desktop). Do not apply marketing whitespace to a data tool or the reverse.
- Optical corrections are allowed and expected: text in a button often needs 1px less padding on top than bottom depending on the font; an icon next to text needs less gap than two pieces of text; padding around a shape with rounded corners can be tighter than around a square one.
- Nested radii: inner radius = outer radius minus the padding between them. Equal radii on nested corners look pinched.

## 4. Layout and grids

Let content determine the layout. A grid is a tool for alignment, not a requirement to fill twelve columns.

- Constrain width. Text blocks get a measure (`65ch`). Pages get a container, commonly 1120 to 1280px for marketing, wider or fluid for data tools. Not every element should stretch to the container; a form at 1200px wide is harder to use than one at 480px.
- Align to few edges. Count the distinct left edges in a view; fewer is calmer. Things that are related share an edge.
- Use CSS Grid for two-dimensional structure and page skeletons, Flexbox for one-dimensional rows and toolbars.
- Intrinsic grids often replace breakpoints:

```css
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 18rem), 1fr));
  gap: 1.5rem;
}
```

- Use `subgrid` (or shared row templates) so titles, bodies and actions align across sibling cards with different content lengths. Pin card actions to the bottom.
- Sidebar plus content: fix the sidebar width (roughly 220 to 280px), let content flex, and give the content its own max-width inside.
- Unequal columns usually look more intentional than equal ones: 2:1, 3:2, or a fixed column beside a fluid one. Reserve equal columns for genuinely equal items.
- Reserve space for anything that loads late (images with `aspect-ratio` or width/height attributes, embeds, async sections) so the layout does not shift.
- Use logical properties (`margin-inline`, `padding-block`, `inset-inline-start`) when the product may ship in RTL languages.
- Use `min-height: 100dvh` rather than `100vh` for full-height sections on mobile.
- Keep a small z-index scale as tokens (base, dropdown, sticky, overlay, modal, toast). No `9999`.

## 5. Color and contrast

### Building a palette

- Neutrals do most of the work: 8 to 10 steps from background to text. Tint them slightly toward one hue (warm or cool) and keep that tint consistent. Mixing warm and cool grays looks accidental.
- One accent hue, with enough steps for default, hover, pressed, subtle background and text-on-accent.
- Semantic colors for success, warning, danger and info, each with a strong step (text, icon) and a subtle step (background). They are for meaning only, never decoration.
- A common proportion is roughly 60% neutral surface, 30% secondary neutral, 10% accent. The accent is strong because it is rare.
- Build ramps in a perceptual space (OKLCH) so steps with the same lightness number look equally light across hues:

```css
--accent-500: oklch(62% 0.19 255);
--accent-600: oklch(54% 0.19 255);
```

- Avoid pure black text on pure white for long reading; a very dark neutral is easier on the eye. Do not go so soft that contrast fails.

### Contrast (WCAG 2.2 AA)

- Text: at least 4.5:1 against its background.
- Large text (24px and up, or about 18.66px and up when bold): at least 3:1.
- UI component boundaries, icons that carry meaning, focus indicators and state indicators: at least 3:1 against adjacent colors.
- Placeholder text and muted secondary text are still text and need 4.5:1. Disabled controls are exempt, but should stay legible.
- Text over images or gradients: measure against the worst spot, or add a scrim.

Compute the ratio; do not estimate it. Light gray on white and white on a mid-tone accent are the usual failures.

### Meaning

Never signal state with color alone. Pair it with an icon, text, shape, position or pattern: an error field has a message and an icon, not only a red border; a chart series has a label or pattern, not only a hue.

### Dark mode

Dark mode is a second palette, not an inversion.

- Background is a dark neutral, not pure black (pure black with bright text causes halation; OLED-specific themes are the exception).
- Elevation is expressed by lighter surfaces, since shadows barely show.
- Reduce the chroma of accents and semantic colors; saturated colors vibrate on dark backgrounds.
- Re-check every contrast pair; do not assume they carry over.
- Set `color-scheme: light dark` so native controls and scrollbars follow.

## 6. Surfaces and depth

Separate things with, in order of preference: space, then a background tone difference, then a border, then a shadow. Reach for the next one only when the previous is not enough. A border and a shadow and a different background on the same card is three answers to one question.

- A card is justified when the content is a discrete object that can be acted on, moved or compared. A section of a page is not a card. Do not nest cards.
- Borders: 1px, low contrast against the surface for decorative separation. Borders that define an input or control need 3:1.
- Shadows: decide on one light direction (usually from above), define 3 to 4 elevation levels as tokens, and layer two shadows per level (a tight one for the contact edge and a wide soft one). Tint the shadow toward the background hue instead of using pure black.
- Elevation means something: it tells the user what floats above what (dropdown over page, modal over everything). Do not put shadows on things that are not above anything.
- Radius: choose a small family (for example 4, 8, 12, full) and assign by element size: small radii for small controls, larger for containers. Radius sets tone: sharp reads technical or editorial, large reads friendly.

## 7. Components and states

A component is not done until every state it can enter is designed.

| State | Notes |
| --- | --- |
| Default | The resting look. |
| Hover | Pointer devices only; never the only way to reveal something essential. |
| Focus-visible | A clear ring, 2px or more, 3:1 against adjacent colors, with an offset so it is not swallowed by the element. |
| Active / pressed | Immediate feedback: darker tone, slight inset or 1px translate. |
| Selected / current | Distinct from hover and from focus. Current page in navigation is marked. |
| Disabled | Visibly inactive and not focusable; say why nearby when the reason is not obvious. Prefer leaving a submit button enabled and validating on submit. |
| Loading | Keep the element's size; show progress in place; prevent double submission. |
| Error | Message next to the field, says what is wrong and how to fix it, linked with `aria-describedby`. |
| Empty | Explains what belongs here and offers the action that fills it. |

Further points:

- Forms: visible labels above or beside inputs (a placeholder is not a label), help text below, one column by default, field width hinting at expected length, correct `type`, `inputmode` and `autocomplete`. Validate on blur or submit, not on every keystroke before the user has finished.
- Buttons: label with a verb and object ("Save changes"). Keep height consistent within a size (for example 32, 40, 48). Icon-only buttons need an accessible name and usually a tooltip.
- Targets: at least 24×24 CSS px (WCAG 2.2 AA minimum); aim for 44×44 on touch. Extend the hit area with padding or a pseudo-element when the visual is small.
- Tables: left-align text, right-align numbers, tabular figures, sticky header for long tables, row height matched to density, a hover or zebra treatment but not both at full strength.
- Icons: one set, one stroke width, sized on the same grid (16, 20, 24), aligned to the text's optical center. Decorative icons get `aria-hidden="true"`.
- Skeletons mirror the real layout so nothing jumps when content arrives. For waits under about 300ms show nothing.
- Overlays: modals for decisions that block; otherwise inline expansion, popover or side panel. Every overlay closes with Escape and returns focus to its trigger.

## 8. Tokens and design systems

Use three layers. Components only ever reference the second or third.

1. Primitive: raw values. `--gray-900`, `--blue-500`, `--space-4`.
2. Semantic: purpose. `--color-text`, `--color-text-muted`, `--color-surface`, `--color-surface-raised`, `--color-border`, `--color-accent`, `--color-danger`, `--radius-control`, `--shadow-overlay`.
3. Component (only when needed): `--button-bg`, `--input-border`.

Theming (dark mode, brand variants, density) is done by remapping the semantic layer; component code does not change.

A minimal set for a new project:

```css
:root {
  color-scheme: light dark;

  --font-sans: "Your Text Face", system-ui, sans-serif;
  --text-xs: 0.75rem;  --text-sm: 0.875rem; --text-md: 1rem;
  --text-lg: 1.25rem;  --text-xl: 1.5rem;   --text-2xl: 2rem;

  --space-1: 0.25rem; --space-2: 0.5rem; --space-3: 0.75rem;
  --space-4: 1rem;    --space-6: 1.5rem; --space-8: 2rem;
  --space-12: 3rem;   --space-16: 4rem;

  --radius-sm: 4px; --radius-md: 8px; --radius-lg: 12px;

  --color-bg: ...;        --color-surface: ...;
  --color-text: ...;      --color-text-muted: ...;
  --color-border: ...;    --color-accent: ...;
  --color-on-accent: ...; --color-danger: ...;
  --color-focus: ...;

  --duration-fast: 120ms; --duration-base: 200ms; --duration-slow: 320ms;
  --ease-out: cubic-bezier(0.2, 0, 0, 1);
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
}
```

Rules for working inside an existing system:

- Search for an existing component and an existing token before creating either.
- Extend through variants and props, not through one-off overrides at the call site.
- If a value is missing from the scale, either use the nearest step or add a step deliberately. Do not hardcode an in-between value.
- Name by role, not by appearance (`--color-danger`, not `--red`; `size="sm"`, not `height-32`).
- With Tailwind or similar, put tokens in the theme config and avoid arbitrary values (`p-[13px]`, `text-[#3a3a3a]`).
- Prefer composition (small parts combined) to one component with many boolean props.

## 9. Motion

Motion earns its place by doing one of four jobs: confirming an action, showing where something came from or went, directing attention to a change, or expressing brand character in one deliberate moment. If it does none, remove it.

### Timing

| Kind | Duration |
| --- | --- |
| Hover, press, toggle, color change | 80 to 150ms |
| Dropdown, tooltip, popover, small reveal | 150 to 220ms |
| Modal, drawer, panel, layout shift | 220 to 350ms |
| Full-page or hero sequence | 400 to 700ms, used rarely |

- Exits are faster than entrances (about 70% of the duration).
- Larger travel distance and larger elements take longer. Frequent interactions take less; something done a hundred times a day should feel instant.
- Easing: decelerate (`ease-out`) for things entering or responding to the user; accelerate (`ease-in`) for things leaving; `ease-in-out` for things moving between two on-screen positions. Avoid `linear` except for continuous loops and progress.
- Springs suit direct manipulation (drag, swipe, pull). Keep bounce low in product UI.

### Execution

- Animate `transform` and `opacity`. Avoid animating `width`, `height`, `top`, `left`, `margin` or `box-shadow` directly; use transforms, `grid-template-rows: 0fr → 1fr`, or the View Transitions API.
- Keep spatial logic consistent: a panel that slides in from the right leaves to the right; a popover grows from its trigger (`transform-origin`).
- Stagger sparingly: 20 to 40ms between items, capped so the whole list finishes in about 300ms.
- Never block input while an animation runs, and make animations interruptible.
- Do not animate on every scroll position. One orchestrated entrance is better than a fade-up on each section.
- Loading indicators appear only after a short delay (about 300ms) to avoid flashing.

### Reduced motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

This blanket rule is a floor. Better: keep opacity and color transitions, and remove movement, scaling, parallax and autoplay. Anything that moves for more than five seconds needs a pause control. Nothing flashes more than three times per second.

## 10. Accessibility

Baseline: WCAG 2.2 AA. Treat it as part of the build, not a later pass.

- Semantics first. Use the native element (`button`, `a`, `label`, `input`, `select`, `dialog`, `nav`, `main`, `table`) before reaching for ARIA. A `div` with a click handler is not a button.
- One `h1`, heading levels in order without skipping, landmarks for the main regions, a skip link to main content.
- Keyboard: everything operable with Tab, Shift+Tab, Enter, Space, arrows where the pattern calls for them, and Escape to close. Tab order follows visual order. No positive `tabindex`.
- Focus: always visible (`:focus-visible`), never removed without a replacement, not hidden behind sticky headers. Dialogs trap focus while open and restore it on close.
- Names: every control has an accessible name. Icon-only buttons use `aria-label`. Inputs have a `<label for>`. Images that carry meaning have descriptive `alt`; decorative ones have `alt=""`.
- Status changes that happen without a page change are announced (`role="status"` or `aria-live="polite"`; `role="alert"` only for errors that need immediate attention).
- Text resizes to 200% and the page reflows at 320 CSS px wide without horizontal scrolling or lost content. Do not disable zoom.
- Users can override line height and letter spacing without clipping: avoid fixed heights on text containers.
- Dragging has a non-drag alternative. Hover-only content is reachable by keyboard and stays open while the pointer is over it.
- Set `lang` on the document. Do not rely on `title` attributes for essential information.

What automated tools cannot tell you: whether the reading order makes sense, whether names are meaningful, whether focus goes somewhere sensible. Tab through the screen yourself.

## 11. Responsive behavior

- Decide breakpoints by where the content breaks, not by device names. Widen the viewport slowly and add a breakpoint where the layout starts to look wrong.
- Prefer fluid techniques that need no breakpoint: `minmax()`, `auto-fit`, `flex-wrap`, `clamp()`.

```css
h1 { font-size: clamp(2rem, 1.2rem + 3.5vw, 4rem); }
.section { padding-block: clamp(3rem, 8vw, 8rem); }
```

  Include a `rem` term in fluid type so it still responds to user zoom.
- Use container queries for components that live in different-width slots (a card in a sidebar and in a main grid).
- Things that change with width: number of columns, navigation pattern, table presentation (priority columns, horizontal scroll inside the table's own container, or a stacked card view), density, and what is visible by default. Hierarchy and available actions should not change.
- Touch: no hover dependence, larger targets, primary actions within thumb reach, respect `env(safe-area-inset-*)`.
- Check at about 320, 375, 768, 1280 and 1920px wide, and at a short height (landscape phone, small laptop). Very wide screens fail as often as narrow ones: lines too long, content stranded in one corner.
- Images: `srcset` and `sizes`, modern formats, `loading="lazy"` below the fold, explicit dimensions.
