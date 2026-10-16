# Redesign audit

For upgrading an interface that already exists: "make it look better", "more professional", "modernize", "polish". The goal is a visibly better result with the same functionality, in the same stack, in reviewable steps.

A request to redesign is itself the justification for visual change, so the gate in `ux-foundations.md` section 7 is satisfied for appearance. It still applies to behavior: do not change how things work unless that was asked for.

## Sequence

1. **Scan.** Identify the framework, the styling method (and its version, for example Tailwind v3 versus v4), the component library, token or theme files, fonts and icon set. Render the main screens and screenshot them before touching anything; these are the "before".
2. **Diagnose.** Go through the audit lists below against the screenshots. Write down what you find, grouped by layer (tokens, components, pages), most damaging first. Separate real problems from things that are merely not to your taste.
3. **Agree on scope when it is large.** If the diagnosis implies touching most of the app, tell the user what you intend and in what order before doing all of it.
4. **Fix in the order given under "Fix order".** Work at the token and shared-component level wherever possible so one change improves every screen.
5. **Verify.** Render again, compare with the "before", and run `ui-review-checklist.md`. Confirm that nothing functional changed.

## Constraints

- Keep the stack. No framework or styling-library migration.
- Keep functionality, routes, data flow, form behavior and accessibility semantics intact. Test after each group of changes.
- Check the dependency file before importing a library; do not add one for something the platform or the project already provides.
- Small, targeted improvements over a rewrite. Each step should be reviewable on its own.
- Keep what is already good. Not everything in an old interface is a problem.
- Brand assets (logo, brand colors, a chosen typeface) are not yours to replace without asking.

## What to look for

Each item is a symptom to check, with the usual remedy. The remedies follow `ui-design-core.md`; read the matching section there for detail. None of these is automatically wrong: confirm it is a problem here before changing it.

### Typography

- No type scale: many near-identical sizes. Collapse to 6 to 8 steps.
- A default font used with no decisions about weight, size or spacing. Either choose a face that suits the subject or tune the existing one properly; a swap alone is not a fix.
- Headings that do not lead: too close in size and weight to body text. Increase the step, tighten line height and tracking on large sizes.
- Body lines too long. Constrain to about 65 characters.
- Only regular and bold in use. Medium and semibold give finer levels.
- Numbers that do not align in tables. Tabular figures, right alignment.
- Orphaned last words in headings. `text-wrap: balance`.
- Uppercase tracked labels on every section, title case on every heading. Use sentence case; keep small caps or uppercase for the few places it helps.

### Color and surfaces

- Too many accent colors, or accents used decoratively. Reduce to one accent with a defined role, plus semantic colors.
- Grays of mixed temperature. One tinted neutral ramp.
- Low-contrast text, especially muted text and placeholders. Compute and fix to AA.
- Stock framework palette untouched. Derive a palette for this product.
- Gradients as decoration (the purple-to-blue wash in particular). Remove unless they carry the concept.
- The same gray shadow on everything; shadows on things that do not float. Define a few elevation levels and apply them by meaning.
- A border, a shadow and a tinted background on the same element. Keep one.
- Abrupt dark sections in a light page or the reverse, with no reason. Keep one tonal family per page.
- Dark mode made by inversion. Rebuild as its own palette.

### Layout and spacing

- No spacing scale: arbitrary gaps. Map everything to the scale.
- Equal spacing everywhere, so groups do not read. Tighten within groups, widen between them.
- No container or measure: content stretched across wide screens. Constrain.
- Everything centered. Left-align content; keep centering for short ceremonial blocks.
- Every section built from the same grid of identical cards. Vary structure according to content; use cards only for discrete objects.
- Sibling cards whose titles, text and buttons do not line up. Shared rows or subgrid; pin actions to the bottom.
- Layout held together with percentage math. Use Grid.
- `100vh` sections that jump on mobile. `min-height: 100dvh`.
- One radius on every element regardless of size; nested corners with equal radii. Small radius family; inner radius = outer minus padding.
- Cramped data screens or, conversely, a work tool padded like a landing page. Set density by how the screen is used.

### Components and states

- Missing hover, focus-visible, active or disabled states. Add all of them.
- Focus outline removed. Restore a visible ring.
- No loading, empty or error states. Design them.
- Current page not marked in navigation.
- Buttons with no hierarchy: several filled buttons competing. One primary, the rest quieter.
- Modals used for simple edits. Move inline or to a panel where that keeps context.
- Icons from mixed sets or with mixed stroke widths. One set, one weight, consistent sizes.
- Emoji standing in for icons.
- Native `alert()` or bare browser validation for errors. Inline messages.
- Links to `#` and buttons that do nothing. Wire them up or remove them.

### Motion

- Instant state changes that feel abrupt, or slow ones that feel sluggish. Bring into the ranges in `ui-design-core.md` section 9.
- Animated `top`, `left`, `width`, `height`. Use transforms and opacity.
- Fade-up on every section, lift on every card. Keep one deliberate moment, remove the rest.
- No reduced-motion handling. Add it.

### Content

- Lorem ipsum, "John Doe", "Acme", identical avatars, identical dates, suspiciously round numbers. Replace with plausible content.
- Marketing filler and vague headlines. Rewrite plainly (see `ux-foundations.md` section 6).
- "Oops!" errors and exclamation marks on success. Make them direct.

### Structure and code

- Clickable `div`s, missing landmarks, heading levels out of order. Use semantic elements.
- Images without meaningful `alt`.
- Hardcoded colors, sizes and z-index values scattered through components. Move to tokens.
- Inline styles mixed with the styling system. Consolidate.
- Fixed pixel widths that break at other sizes.
- Missing page title, description and favicon.
- Dead code and debug leftovers.

### Commonly forgotten

- A way back from every screen; a useful not-found page.
- A skip link to main content.
- Form validation and what happens on failure.
- Legal links, and consent handling where the jurisdiction requires it.

## Fix order

Highest visual return for lowest risk first.

1. **Tokens**: establish or clean up the color, type, spacing, radius and shadow scales. Most later fixes become find-and-replace.
2. **Typography**: scale, weights, line height, measure, and the face if it is being changed.
3. **Color**: neutral ramp, single accent, contrast.
4. **Spacing and layout**: scale, grouping, container, alignment.
5. **Shared components**: buttons, inputs, cards, navigation, with all their states.
6. **Loading, empty and error states.**
7. **Page-specific composition**: the one or two screens that carry the product get individual attention, guided by `visual-direction.md`.
8. **Motion**, last.

Stop when the audit items that mattered are resolved. A redesign is finished when the interface is coherent, not when every element has been changed.

## Reporting

Summarize by layer what changed and why, show before and after where possible, list anything found but deliberately left alone, and state anything that needs a decision from the user (brand color, typeface licensing, copy that needs a subject expert).
