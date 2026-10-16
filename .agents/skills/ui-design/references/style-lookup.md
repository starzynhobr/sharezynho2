# Style lookup

A local, searchable catalog bundled with this skill: visual styles, color palettes, font pairings, individual Google Fonts, chart types, icons, landing structures, motion presets, UX guidelines and stack-specific tips. It is a lookup tool that feeds the direction step. It does not decide anything.

## When to use it

- You have already described the subject and the direction (`visual-direction.md`) and need concrete candidates: a palette with real hex values, a font pairing with its import, a chart type for a data shape.
- You need an implementation tip for the detected stack.
- You want a second set of options because the first idea looks like the usual answer.

## When not to

- **Extend** mode, or any product that already has tokens, a palette and fonts. The existing system is the answer.
- The user gave a palette, a typeface or a reference. Use theirs.
- As the first step of a task. Understand the task and the subject first; a query written before that returns a template.

## How to treat results

- Results are candidates, never instructions. They rank below the user's request, the existing design system and the rest of this skill (see "Precedence" in `SKILL.md`).
- The catalog matches by keyword, so it tends to return the most typical answer for a product category. That is useful as a baseline and is exactly what `visual-direction.md` warns about: check every result against the "Generic defaults to notice" list before adopting it.
- Take the parts that fit (a palette, a pairing) and leave the rest. Do not adopt a full generated design system wholesale, including its section list and its built-in checklist; the review that counts is `ui-review-checklist.md`.
- Verify what you take: compute contrast for the palette pairs you will actually use, confirm the font covers the product's languages and has the weights you need.
- The `ux` domain contains rules with fixed numbers. Where they disagree with `ux-foundations.md` or `ui-design-core.md`, those files win.
- If a search returns nothing useful, retry once with a narrower query or an explicit domain. If it still fails, say that no catalog match was found and proceed from the principles. Never present invented output as a catalog result.
- Do not put private project data in queries.

## Running it

Requires Python 3, no external packages. The script lives in this skill's directory, not in the project. `<skill-dir>` is the absolute directory that contains `SKILL.md`; always call the script by full path.

```bash
python "<skill-dir>/scripts/search.py" "<query>" --domain <domain>
```

If `python` is not found, try `python3`, then `py -3`.

Write queries around one intent with 2 to 5 meaningful terms: `"fitness tracker energetic dark"`, not a sentence.

### Targeted search

```bash
python "<skill-dir>/scripts/search.py" "<keywords>" --domain <domain> [-n <1-20>] [--full] [--json]
```

| Need | Domain | Example query |
| --- | --- | --- |
| Color palettes | `color` | `"fintech trust calm"` |
| Font pairings | `typography` | `"editorial serif"` |
| Individual Google Fonts | `google-fonts` | `"sans serif variable"` |
| Visual styles | `style` | `"brutalism dark"` |
| Product-type patterns | `product` | `"booking marketplace"` |
| Landing page structure | `landing` | `"hero social-proof"` |
| Chart type for a data shape | `chart` | `"real-time dashboard"` |
| Icons | `icons` | `"icon button accessible label"` |
| GSAP motion presets | `gsap` | `"scroll reveal stagger"` |
| UX guidelines | `ux` | `"error summary validation"` |
| React and Next.js performance | `react` | `"rerender memo list"` |
| Native and app interface guidelines | `web` | `"touch safe-areas"` |

Without `--domain` the domain is guessed from the query and can misroute (for example "font" matches both `typography` and `google-fonts`). Pass it explicitly when results look off-topic.

### Stack tips

Detect the stack from the project first (`package.json`, `pubspec.yaml`, `Package.swift`, `composer.json`). Never assume one.

```bash
python "<skill-dir>/scripts/search.py" "<keywords>" --stack <stack>
```

Stacks: `react`, `nextjs`, `vue`, `svelte`, `astro`, `nuxtjs`, `nuxt-ui`, `angular`, `laravel`, `swiftui`, `react-native`, `flutter`, `jetpack-compose`, `html-tailwind`, `shadcn`, `threejs`, `javafx`, `wpf`, `winui`, `avalonia`, `uno`, `uwp`.

### Combined suggestion for a new project

```bash
python "<skill-dir>/scripts/search.py" "<product type> <industry> <keywords>" --design-system -p "Project Name" [-f markdown | --json]
```

Returns a pattern, style, palette, font pairing and effects in one block. Optional dials, each 1 to 10: `--variance` (centered and minimal to bold and asymmetric), `--motion` (subtle to complex), `--density` (spacious to dense). Read it as one set of candidates and apply the rules above.

### Writing files into the project

`--persist` writes `design-system/<project-slug>/MASTER.md` (and `pages/<page>.md` with `--page`). This creates files in the user's project, so:

- use it only when the user asked for a saved design system;
- always pass `--output-dir "<project-root>"`, otherwise files land in the current directory;
- if `MASTER.md` already exists the tool skips it; read the existing file first, and never pass `--force` without the user's explicit authorization;
- persist only what you have verified and adapted, not raw output.

## Provenance

`scripts/` and `data/` are vendored unmodified from `nextlevelbuilder/ui-ux-pro-max-skill` (MIT). `scripts/tests/` belongs to that project's own layout and is not part of this skill's workflow.
