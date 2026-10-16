# Visual direction

Opinionated guidance for choosing how an interface should look when the brief leaves that open. Everything here yields to an explicit request and to an existing design system.

## Why generated UI looks generic

Each default choice is reasonable on its own. The safe font, the centered hero, the three feature cards, the soft shadow, the fade-up on scroll: none is wrong. Stacked together, with no choice made for this particular subject, they produce a page that could belong to any product. Generic is the absence of decisions, not the presence of bad ones.

The fix is not decoration. It is making a small number of choices that could only have been made for this brief, and carrying them through consistently.

## Start from the subject

Before picking anything visual, answer in a sentence each:

- What is this, and who uses it, in what situation? (An accountant closing the month at a desk is not a teenager on a phone at a concert.)
- What should it feel like? Pick two or three adjectives that exclude something: "precise, quiet, dense" excludes playful; "warm, handmade, slow" excludes technical.
- What does the subject's own world look like? Its materials, documents, instruments, signage, packaging. A gym app can borrow from weight plates and chalkboards; a translation tool from dictionaries and proof marks. This is where non-generic ideas come from.

Then pick the one thing the design will be remembered for: an unusual type treatment, a single strong color, a layout idea, a piece of imagery, one interaction. One. Everything else stays disciplined so that it can be seen.

## The axes

A direction is a position on each of these. Choose deliberately; write the choices down.

| Axis | Range |
| --- | --- |
| Density | Compact and information-rich ↔ sparse and focused |
| Type voice | Neutral grotesque ↔ humanist ↔ geometric ↔ serif ↔ mono ↔ expressive display |
| Scale contrast | Even sizes ↔ extreme jumps between display and text |
| Color | Near-monochrome ↔ one accent ↔ multi-hue |
| Temperature | Cool neutrals ↔ warm neutrals |
| Shape | Sharp corners ↔ soft ↔ fully rounded |
| Depth | Flat ↔ bordered ↔ layered with shadow |
| Composition | Symmetric and centered ↔ asymmetric and offset |
| Imagery | None ↔ diagrammatic ↔ photographic ↔ illustrated |
| Motion | Still ↔ functional only ↔ expressive |

Coherence matters more than any single choice. Sharp corners, a mono face, hairline borders and no shadows agree with each other. Sharp corners with a bubbly rounded display face and heavy shadows do not, unless the clash is the idea.

## Product UI is not a marketing page

Decide which one you are making.

**Product UI** (dashboards, editors, admin, settings, anything used repeatedly): the user came to do work. Familiarity is a feature. Character comes from precision: a well-tuned type scale, exact alignment, considered density, consistent states, good empty and error screens, one confident accent. Expressive layout and decorative motion get in the way on the hundredth visit. Restraint here is not generic; sloppiness is.

**Marketing, landing, portfolio, editorial**: the user has not decided to care yet. The page must be recognizably about its subject within one screen. This is where scale, asymmetry, imagery and one orchestrated motion moment belong.

Applying landing-page drama to a data tool, or dashboard neutrality to a landing page, is the commonest direction error.

## Tools of emphasis and when to use them

### Scale

Large differences in size create drama and an unmistakable entry point. Use a very large element (headline, number, image) when the view has one message. Do not use it when the view has many equally important items; that needs an even rhythm instead. If you go big, go clearly big: a display size three to six times the body size reads as intent, 1.5 times reads as indecision.

### Whitespace

Space isolates and gives weight. Use generous space around the one thing that matters and to separate sections with different purposes. Do not use it as a default padding multiplier: empty space around unimportant content makes a page feel thin, and in a work tool it costs the user scrolling. Whitespace should be uneven on purpose: tight within groups, wide between them.

### Asymmetry

A symmetric, centered layout is stable and formal, and is the default. Asymmetry creates direction and tension. Use it when there is a natural pairing of unequal things (headline and image, text and form, summary and detail), or to lead the eye down a page by alternating weight. Anchor it to the grid: offset by whole columns, keep a strong shared edge, so that it reads as composed rather than misaligned. Do not use it for forms, tables, settings or anything scanned in a straight line.

### Alignment

Left-aligned text with a ragged right edge is the most readable and the most flexible. Centered layouts suit short, ceremonial content: a hero with one line, an empty state, a confirmation. A page that centers everything has no edges for the eye to follow.

### Color

A restrained palette with one accent used rarely is stronger than several colors used often. Consider letting a neutral be the "brand": a specific warm off-white or a deep ink can carry identity as well as a hue. Large flat areas of a saturated color are a strong move; use once per page at most. If the subject has a color of its own (a sport's turf, a material, a signal standard), start there instead of from a default blue.

### Type as the design

When there is no imagery, type is the image. Options: one very large line; a distinctive display face used only for headings; a strong contrast in width or weight between two cuts of the same family; a mono face for data set against a serif for prose. Pick one device and repeat it; do not use all of them.

### Imagery and texture

Real content beats abstraction: a screenshot of the actual product, a real photo of the subject, a real data sample. Abstract gradient blobs, 3D shapes and generic illustration sets say nothing about the subject. If there is no real imagery, a diagram, a piece of the interface, or type alone is better than filler.

### Motion

Choose a level and hold it. Functional only (state changes, overlays) is right for most product UI. One signature moment (a page-load sequence, a hero interaction) suits a marketing page. Motion everywhere is never the right level. See `ui-design-core.md`, section 9, for timing.

### Density

Compact is not ugly and sparse is not premium. Match density to how often and how long the screen is used and how much the user needs to compare at once. Experts want more on screen; first-time visitors want less.

## Generic defaults to notice

None of these is forbidden. Each is a default: if it appears in your plan, ask whether you chose it for this brief or simply arrived at it. If the brief asks for one of them, use it.

Layout
- Centered hero: headline, subhead, two buttons, then a screenshot in a browser frame.
- A row of three equal feature cards with an icon, a title and two lines of text.
- Every section the same structure: small label, centered heading, grid of cards.
- Everything in a rounded card, cards inside cards.
- Stats row of four big numbers with small labels.

Color and surface
- Purple-to-blue or pink-to-orange gradients on text, buttons or backgrounds.
- Dark background with glowing blurred color blobs.
- Glass panels (blur plus translucent white) without a reason.
- The same faint gray shadow and the same radius on every element.
- Default framework palette (stock indigo or blue accent) untouched.
- Cream background, serif display face and a terracotta accent as the "tasteful" fallback.

Type
- The same sans face that every template uses, at default weights, with no scale decisions.
- An uppercase, letter-spaced eyebrow label above every heading.
- One word in a headline set in a gradient, italic or accent color.
- Mono face for small labels to signal "technical" on a non-technical product.

Content
- Emoji as icons or bullet markers.
- Headlines of the form "Verb your noun" with words like seamless, effortless, powerful, elevate, supercharge.
- Numbered markers (01, 02, 03) on things that are not a sequence.
- Invented metrics, fake testimonials, lorem ipsum, "John Doe".
- A sparkle icon to denote anything involving AI.

Motion and interaction
- Every section fades and slides up on scroll.
- Every card lifts and grows on hover.
- A cursor-following glow or spotlight border without purpose.

## Committing

A direction that is applied at 60% reads as an accident. Once chosen:

- Apply it everywhere it logically applies. If corners are sharp, the buttons, inputs, cards, images and badges are all sharp.
- Push the main idea further than is comfortable, then check it in the render and pull back if it hurts use.
- Keep everything that is not the main idea quiet: neutral color, consistent spacing, no second gimmick.
- Before finishing, remove one decorative thing. If nothing is lost, it was not needed.

## When not to pursue a direction

- There is an existing product with its own look. Match it. A new screen that is more "designed" than its neighbours is a consistency bug.
- The task is a fix or a small feature. Do not restyle surrounding code that was not part of the request.
- The user asked for something conventional or gave a reference. Follow the reference.
- The audience depends on convention to get work done (accessibility tools, internal operations, forms with legal weight). Clarity outranks character.

## Presenting a direction

When the direction is open and the work is substantial, state it briefly before building so it can be corrected cheaply: the subject as you understand it, the adjectives, the one memorable element, type, color, layout concept, and what you are deliberately leaving out. A few lines are enough. For small tasks, make the choice, build, and mention the choice in the summary.
