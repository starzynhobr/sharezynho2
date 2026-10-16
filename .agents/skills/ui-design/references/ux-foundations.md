# UX foundations

Whether the interface works for the person using it. This comes before how it looks. Stack-agnostic: applies to web, mobile, desktop and game UI.

## Contents

1. Start from the task
2. Interaction
3. Information architecture
4. Forms
5. States and feedback
6. Copy
7. The gate for changing existing UI
8. Performance as UX
9. Evidence discipline
10. Validation

## 1. Start from the task

Before designing or changing anything, identify:

- the user's goal on this screen, in one sentence;
- the information they need to make the next decision;
- the primary action, the secondary actions and the merely optional ones;
- how it is likely to fail (bad input, no data, no network, no permission);
- the patterns already in the product that work and should be kept.

Make the primary path obvious without hiding useful secondary actions. Prefer recognition over recall: show the relevant choices, context, status and consequences rather than making the user remember them.

Use progressive disclosure only when it reduces real complexity. Hiding commonly needed information to make the screen look cleaner moves the cost to the user.

## 2. Interaction

- The result of every interaction is understandable. If it is not already obvious, acknowledge it promptly.
- Prevent predictable errors where practical: constrain inputs, offer sensible defaults, disable nothing without saying why.
- Recoverable errors are explained in plain language with a next action.
- Preserve user control: cancel, back, undo, edit. Undo is better than a confirmation dialog for reversible actions.
- Match confirmation cost to consequence. Do not confirm trivial reversible actions; do protect destructive, irreversible ones, and name what will be destroyed.
- Keep frequent targets easy to hit and dangerous actions away from likely mis-click or mis-tap paths.
- Prefer the familiar pattern unless diverging has a clear benefit to the user. Novelty in interaction costs learning time.
- Do not add spinners, pulses or optimistic updates that provide no information.
- Every flow has a way back. No dead ends, no links that go nowhere.

Fitts's and Hick's laws describe tendencies, not layout recipes. How hard a choice is also depends on familiarity, grouping, labels and context.

## 3. Information architecture

- Use the user's vocabulary, not the implementation's. A user manages notifications, not webhook configuration.
- Group by meaning and by workflow, not by arbitrary item counts. There is no magic number of navigation items.
- Navigation reflects the product's mental model. Do not merge clear categories just to shorten a menu, and do not split one concept to fill a layout.
- Show where the user is: current page marked in navigation, a title that matches the link that led there, breadcrumbs where the hierarchy is deep.
- Put things where users of this platform expect them. Conventions (search top, primary action bottom-right of a dialog on some platforms and left on others, back gesture) are part of the architecture.
- Modals are for decisions that must block. For everything else prefer inline editing, a popover, or a side panel that keeps context visible.

## 4. Forms

- Ask only for what is needed now.
- Visible labels; a placeholder is not a label. Help text sits below the field.
- One column by default. Group related fields; make dependencies between fields visible.
- Split into steps only when sequencing, complexity or focus genuinely benefits, and then show progress and allow going back without losing input.
- Field width hints at expected length. Use the right input type, `inputmode` and `autocomplete`.
- Validate on blur or on submit, not on every keystroke before the user has finished. Keep what they typed.
- Error messages sit next to the field, say what is wrong and how to fix it, and are announced to assistive technology.
- Prefer leaving the submit button enabled and validating on submit over a disabled button with no explanation.
- Make review and correction easy before anything irreversible.

## 5. States and feedback

Design the states the product can actually enter. Consider, where relevant: initial, loading, empty, partial, success, error, offline or unavailable, disabled, selected, focused, no permission.

- Loading: show nothing for very short waits; a skeleton when the final structure is known and stable; a spinner or text status otherwise. Keep the layout from jumping when content arrives.
- Empty: explain what belongs here and offer the action that fills it, when there is one.
- Error: say what happened and what to do, where it happened. Never a bare "Something went wrong" when more is known.
- Success: confirm in proportion to the action. Saving a field does not need a celebration.
- Feedback is proportionate, not decorative.

## 6. Copy

Words in an interface exist to make it easier to understand and use. They are design material.

- Write from the user's side. Name things by what they are or do, in plain terms. Specific and clear beats clever.
- Buttons say what happens: "Save changes", "Delete project", not "Submit" or "OK". An action keeps the same name through the flow: the "Publish" button leads to a "Published" confirmation.
- Active voice, plain verbs, sentence case, no filler.
- Errors state the problem and the fix without apology, blame or jokes. "Couldn't save. Check your connection and try again." Not "Oops!".
- No exclamation marks on routine success.
- Avoid marketing superlatives and filler verbs (seamless, effortless, elevate, unleash, supercharge, next-gen). Say what the thing does.
- Each piece of text does one job. Remove labels and headings that restate what is already visible.
- When inventing sample content, make it plausible for the subject: believable names, uneven numbers, varied dates. Never lorem ipsum, "John Doe" or "Acme".
- Match tone to the audience and the product; keep it consistent across screens.

## 7. The gate for changing existing UI

Before changing something that already works, ask: **what concrete problem does this solve?**

A change is justified when it meaningfully improves task completion, comprehension, error prevention or recovery, accessibility, consistency, maintainability of the design system, responsiveness or performance, or an explicit goal the user stated (including "make it look better").

If the result would only be a different valid aesthetic, keep what is there.

- Fix the smallest layer that owns the problem: a token before a component, a component before a page.
- Avoid cascading redesign. A fix to one screen is not permission to restyle its neighbours.
- Consistency matters, uniformity does not: different importance can justify different emphasis.
- Visual trends (glass, gradients, flat, cards, minimalism, maximalism, shadows, rounded corners) are style choices, never usability rules.

## 8. Performance as UX

- Acknowledge input immediately, even when the result takes time.
- Keep layouts stable: reserve space for anything that loads late.
- Important content before nonessential content.
- No unnecessary assets, fonts, or work on the main thread.
- Measure with what fits the platform. Web metrics are not universal rules for native, desktop or game software.

## 9. Evidence discipline

Know what kind of claim you are making:

- **Standard**: a formal requirement (WCAG, platform guidelines).
- **Evidence**: supported by research.
- **Heuristic**: a useful principle that depends on context.
- **Convention**: an established product or platform pattern.
- **Aesthetic**: a deliberate visual choice.

Do not present a heuristic or an aesthetic preference as a law. Avoid unsupported magic numbers ("at most 7 menu items", "3 to 5 fields per group"). Use hard numbers only where a standard gives them, and treat the rest as defaults.

When reviewing, report only real issues, in the form: **problem → why it matters → recommended change → kind of claim**. Do not invent findings to make a review look thorough.

## 10. Validation

- Inspect the existing system before proposing anything.
- Render and use the interface; walk the primary task from start to finish.
- Exercise the edge and error states, not only the happy path.
- Check keyboard and assistive behavior, and the screen sizes and input methods the product supports.
- Compare with the product's existing conventions.

For high-impact flows, a heuristic review does not replace testing with representative users. Say so when it applies.

Reference anchors, for when deeper justification is needed: Nielsen's usability heuristics; Fitts (1954); Hick (1952) and Hyman (1953); Cowan (2001) on working memory; WCAG 2.2 and WAI-ARIA; ISO 9241-210; the target platform's human-interface guidelines.
