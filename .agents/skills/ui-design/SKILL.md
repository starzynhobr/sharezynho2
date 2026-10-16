---
name: ui-design
description: "Single UI/UX skill for any user-facing interface: usability, hierarchy, type, spacing, grids, color, tokens, states, motion, a11y, visual direction, palette/font lookup, redesign audit, self-review. Use when creating, changing, redesigning or reviewing UI."
---

# UI Design

One skill for interface work, from deciding whether a change is justified to checking the rendered result. This file is the router: it tells you which internal reference to read for the task in front of you. Everything needed is inside this skill directory.

## Precedence

When guidance conflicts, the higher item wins.

1. What the user asked for, in their words.
2. The product's existing design system, tokens and conventions. Extend them; do not introduce a parallel set.
3. Platform conventions and accessibility requirements.
4. This skill.
5. Your own aesthetic preference.

Every number in this skill is a starting point to tune by eye in the rendered result, not a law. If the project already has a scale, use theirs. Do not present a heuristic or a taste as a rule of science.

## Pick the mode

Identify which of these the task is, then read the listed files in order. Read a reference when you reach the step that needs it, not all of them up front.

| Mode | The task is | Read |
| --- | --- | --- |
| **Build new** | A new screen, page, component or app with no established look | `ux-foundations.md` → `visual-direction.md` → `ui-design-core.md` → `ui-review-checklist.md` |
| **Extend** | A feature or fix inside a product that already has a look | `ux-foundations.md` (sections 1 and 7) → `ui-design-core.md` for the parts you touch → `ui-review-checklist.md` |
| **Redesign** | "Make it look better/modern/professional", visual polish of something that exists | `redesign-audit.md` → `visual-direction.md` → `ui-design-core.md` → `ui-review-checklist.md` |
| **Review** | Critique or audit, with no implementation asked for | `ui-review-checklist.md`, plus `ux-foundations.md` if the flow itself is in question |

All paths are under `references/` next to this file.

If the mode is unclear, default to **Extend**: it changes the least.

## Workflow

### 1. Understand the task and what exists

State the user's goal on this screen and its one primary action. Then find out what is already there: token files, theme config, component library, font stack, spacing scale, dark mode, breakpoints. Open two or three existing screens, rendered if possible. New work must look like it belongs next to them.

Before changing existing UI, name the concrete problem the change solves. If the answer is only "a different valid look", leave it alone unless the user asked for a restyle.

### 2. Decide the direction

Skip when an existing system already answers it (most **Extend** work).

Otherwise write a short plan before any code: the subject and the job of the screen; the one element that carries the emphasis; type families, roles and scale; neutrals, one accent and semantic colors; the layout concept in one sentence; the density and why. Check the plan against the generic defaults listed in `visual-direction.md` and replace whatever is there only because it is the usual answer.

When the plan needs concrete candidates (a palette with real values, a font pairing, a chart type, a stack-specific tip), query the bundled catalog as described in `style-lookup.md`. Do this after the direction is written, never instead of it, and treat what comes back as options to filter.

If there are no tokens, define a small set first (`ui-design-core.md`, section 8) and build only from it.

### 3. Build in this order

1. Real content and structure, unstyled. Plausible copy and realistic data lengths, never lorem ipsum.
2. Layout and spacing, in grayscale. Hierarchy must already read here.
3. Type.
4. Color.
5. States: hover, focus, active, disabled, loading, empty, error.
6. Motion, last, and only where it explains something.

Work with the project's stack. Do not migrate frameworks or styling libraries, and check the dependency file before importing anything.

### 4. Render and review

Never judge UI from source alone. Render it, capture a narrow and a wide viewport, and run `ui-review-checklist.md`. Fix what it finds and render again. Report what was checked and what was not.

## Rules that hold in every mode

- Design for task completion first. A beautiful screen the user cannot finish their task on has failed.
- One primary action per view. Everything else is visibly quieter.
- Hierarchy comes from several signals at once (size, weight, color, space, position), with an obvious step between levels.
- Spacing comes from a scale. Related things sit closer together than unrelated things, always.
- Text contrast meets WCAG AA; state is never carried by color alone; focus is always visible; everything works from the keyboard.
- Every interactive element has all of its states designed, and every data region has loading, empty and error.
- Motion responds to an action or explains a change, and respects `prefers-reduced-motion`.
- No raw color, spacing or duration values inside components when a token exists.
- Layout survives real content: long names, empty lists, large numbers, translated strings.
- Copy is plain, specific and written in the user's vocabulary.
- Change the smallest layer that owns the problem. Do not restyle what was not part of the request.

## Reference files

| File | Contents |
| --- | --- |
| `references/ux-foundations.md` | Task and flow, interaction and error handling, information architecture, forms, feedback and states, copy, performance as UX, the gate for changing existing UI. |
| `references/ui-design-core.md` | Execution: hierarchy, typography, spacing, grids, color, surfaces, component states, tokens, motion, accessibility, responsive behavior. |
| `references/visual-direction.md` | Opinionated: choosing a direction from the subject, product UI versus marketing, scale, whitespace, asymmetry, the list of generic defaults, when not to pursue a direction. |
| `references/style-lookup.md` | Optional lookup tool (`scripts/search.py` over `data/`): palettes, font pairings, styles, charts, icons, stack tips. How to query it and how to treat the results. |
| `references/redesign-audit.md` | Upgrading an existing interface: scan, diagnose, fix order, what to look for, what not to break. |
| `references/ui-review-checklist.md` | Twelve review passes over the rendered result, severity levels, report format. |
