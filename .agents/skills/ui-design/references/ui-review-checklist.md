# UI review checklist

Run this on your own work after implementing and before reporting it as done. It is a review of the rendered interface, not of the source.

## How to run it

1. Render the real thing: dev server, preview, Storybook, or a static file in a browser. If nothing can be rendered in this environment, say so in the report and review the code against the list as a weaker substitute.
2. Capture screenshots at a narrow width (about 375px) and a wide one (about 1280 to 1440px). Add dark mode if the product supports it.
3. Look at the screenshots before reading the code again. Write down what you see first, second, third.
4. Go through the passes below in order. The early ones find the expensive problems.
5. Fix, render again, re-check what you touched.
6. Stop when the passes are clean. Do not keep restyling for variety.

Scale the effort to the change: a one-line color fix needs passes 1, 4 and 9, not all of them.

## Pass 1: Does it do the job

- [ ] The view's purpose is clear within a few seconds, without reading everything.
- [ ] There is one primary action and it is the most prominent interactive element.
- [ ] It does what was asked. Nothing requested is missing; nothing unrequested was restyled or removed.
- [ ] It looks like it belongs to the rest of the product: same tokens, components, spacing and tone as neighbouring screens.

## Pass 2: Hierarchy

- [ ] Squint or blur the screenshot: the order in which elements stand out matches their importance.
- [ ] Convert to grayscale: hierarchy and grouping still read without color.
- [ ] No two elements compete for first place.
- [ ] Secondary and tertiary content is visibly quieter, not just slightly smaller.
- [ ] Nothing is emphasized that does not need to be (stray bold, extra color, boxes around things).

## Pass 3: Typography

- [ ] Every font size on screen is a step of the scale. Count the distinct sizes; more than about six in one view needs a reason.
- [ ] Body text is at least 14px (16px for mobile inputs) and its line height is comfortable.
- [ ] Running text is 45 to 75 characters per line.
- [ ] Headings do not end on a single orphaned word; multi-line headings have tight line height.
- [ ] Numbers in columns align (tabular figures, right-aligned).
- [ ] No faux bold or italic; the weights used are actually loaded.
- [ ] Fonts load without a visible jump, and the fallback is acceptable.
- [ ] Accented characters and the product's languages render in the chosen face.

## Pass 4: Spacing and alignment

- [ ] Every gap is a step of the spacing scale; no near-duplicates (14px next to 16px).
- [ ] Space within groups is smaller than space between groups, everywhere.
- [ ] Headings sit closer to what they introduce than to what precedes them.
- [ ] Elements that should share an edge do. Scan each vertical edge in the screenshot.
- [ ] Sibling items align internally: titles, text and actions line up across cards and columns.
- [ ] Icons are optically centered with their text; nothing looks a pixel off.
- [ ] Padding is consistent across components of the same kind and size.
- [ ] Nothing touches the viewport edge or another element without intended space.

## Pass 5: Color and contrast

- [ ] Body and secondary text reach 4.5:1; large text 3:1. Checked by calculation, including muted text and placeholders.
- [ ] Control boundaries, meaningful icons and focus rings reach 3:1.
- [ ] Text on images, gradients or accent backgrounds passes at its worst point.
- [ ] No state is communicated by color alone.
- [ ] The accent is used sparingly and means the same thing each time.
- [ ] Semantic colors are used only for their meaning.
- [ ] Grays share one temperature.
- [ ] Dark mode, if supported: checked separately, no leftover light-mode values, no pure-white glare, shadows and borders still separate surfaces.

## Pass 6: States

For each interactive element:

- [ ] Hover, focus-visible, active, disabled and selected exist and are distinguishable from each other.
- [ ] Loading keeps the element's size and blocks double submission.

For each view or data region:

- [ ] Loading state exists and matches the final layout (no jump on arrival).
- [ ] Empty state explains and offers a next step.
- [ ] Error state says what happened and what to do, next to where it happened.
- [ ] Success is confirmed in proportion to the action.
- [ ] Partial, offline or no-permission states are handled where they can occur.

## Pass 7: Content stress

Replace the tidy sample data with hostile data and render again.

- [ ] Very long strings: names, titles, emails, URLs without spaces. Text wraps or truncates on purpose; nothing overflows or pushes the layout.
- [ ] Very short and missing values: empty fields, no avatar, no image.
- [ ] Zero items, one item, and many items (pagination, virtual scroll or a sensible limit).
- [ ] Large numbers, negative numbers, long currency values.
- [ ] Longer translations (German or Portuguese typically run 30% longer than English). Buttons and tabs still fit.
- [ ] No lorem ipsum, placeholder names or invented statistics left in.

## Pass 8: Responsive

- [ ] 320px wide: no horizontal scroll, nothing clipped, text readable.
- [ ] About 375px, 768px, 1280px: layout is intentional at each, not merely unbroken.
- [ ] Very wide (1920px and up): content is constrained, lines are not overlong, nothing is stranded.
- [ ] Short viewport height: primary action reachable; sticky elements do not eat the screen.
- [ ] Touch targets are large enough and spaced apart; nothing depends on hover.
- [ ] Images and media keep their aspect ratio and reserve their space.
- [ ] Browser zoom at 200%: still usable.

## Pass 9: Keyboard and assistive technology

- [ ] Tab through the whole view: every interactive element is reachable, in visual order, with a visible focus ring at each stop.
- [ ] Enter and Space activate; Escape closes overlays; arrow keys work in menus, tabs and lists that call for them.
- [ ] Opening a dialog moves focus into it and traps it; closing returns focus to the trigger.
- [ ] No focusable element is hidden off-screen or behind a sticky bar.
- [ ] Controls are native elements or have correct roles. No clickable `div`.
- [ ] Every control and image has a meaningful accessible name; form fields have labels.
- [ ] Headings are in order; landmarks exist; page has a title and `lang`.
- [ ] Dynamic updates (toasts, validation, async results) are announced.
- [ ] An automated checker (axe, Lighthouse) reports no violations, if one is available. It does not replace the manual steps above.

## Pass 10: Motion

- [ ] Every animation has a reason: feedback, continuity, or attention to a change. Remove the rest.
- [ ] Durations feel quick: interactions under about 200ms, larger transitions under about 350ms.
- [ ] Nothing animates layout properties in a way that stutters.
- [ ] Nothing blocks input while it animates.
- [ ] With `prefers-reduced-motion: reduce` emulated, movement is gone and the interface still works.
- [ ] No looping or autoplaying motion without a way to pause it.

## Pass 11: Generic check

Look at the screenshot as if someone else made it.

- [ ] Could this screen belong to any other product with the logo swapped? If yes, what in it is specific to this subject?
- [ ] Is there one clear idea, applied consistently, or several half-applied ones?
- [ ] Any defaults from `visual-direction.md` ("Generic defaults to notice") present that were not chosen on purpose?
- [ ] Is every card, border, shadow, icon, badge and label doing a job? Remove one decorative element and see whether anything is lost.
- [ ] Is the copy specific and plain, written for the user, with no filler superlatives?

For product UI inside an existing system, replace this pass with: does it match its neighbours exactly?

## Pass 12: Code hygiene

- [ ] No raw color, spacing, radius, shadow, z-index or duration values where a token exists.
- [ ] Existing components reused rather than re-implemented; no one-off overrides at call sites.
- [ ] No arbitrary utility values (`[13px]`, `[#3a3a3a]`) without a reason.
- [ ] No fixed heights on containers that hold text.
- [ ] No leftover debug outlines, test backgrounds, commented-out blocks or unused styles.
- [ ] No new dependency added for something the platform or the project already provides.
- [ ] The console is clean of errors and warnings introduced by the change.

## Severity

- **Blocker**: the task cannot be completed, content is unreachable or unreadable, a WCAG AA failure, layout broken at a supported width. Fix before reporting.
- **Major**: unclear hierarchy, missing state, inconsistency with the product, overflow with realistic content. Fix before reporting.
- **Minor**: small alignment or spacing irregularities, polish. Fix if cheap; otherwise list.

## Report

Keep it short and factual:

- What was built or changed.
- How it was verified: what was rendered, at which widths and themes, which passes were run.
- What was found and fixed.
- What remains, with severity.
- What was not verified (for example: no screen reader run, no real device, dark mode not rendered).

Do not claim a check that was not performed.
