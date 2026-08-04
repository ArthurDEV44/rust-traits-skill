---
name: review-epic
description: "Review and certify one already-implemented PRD epic (`EP-NNN`) as a single unit: prove the code is wired into real execution paths, audit correctness and security, remediate, validate the final state, and roll up story, epic, and PRD status. Use when asked to review an epic, certify an `IN_REVIEW` EP-NNN, validate an implemented epic, recheck a completed epic, or replace story-by-story review with one dependency-aware audit."
---

# review-epic

Review: $ARGUMENTS

## Objective

Prove one implemented epic against its outcome, child criteria, integration contracts, and applicable security boundaries. Correct confirmed defects and certify only the final repository state.

`SCOPE -> AUDIT -> REMEDIATE -> VERIFY/STATUS`

The epic is the execution unit. Child stories organize evidence and dependency reasoning; they never trigger separate review pipelines. This skill alone writes `DONE`, `BLOCKED`, or a downgrade. Report only meaningful milestones, scope ambiguity, confirmed blockers, and the final receipt.

## Profiles

Select the lowest profile covering the highest-risk changed surface.

| Profile | Use when | Audit and validation |
|---|---|---|
| FAST | Isolated behavior, established local pattern, bounded impact, no sensitive boundary | Orchestrator audit plus acceptance and changed-surface gates |
| DEFAULT | Shared integration, public behavior, multi-module change, or familiar dependency work | Orchestrator audit plus acceptance, integration, and applicable project gates |
| DEEP | Auth, payments, PII, migration, untrusted input, crypto, destructive operations, LLM tools, cross-service behavior, or unfamiliar critical logic | Orchestrator audit, optional single-axis independent review, relevant full suite, and applicable security gates |

File count alone does not select DEEP.

## Phase 1: SCOPE

1. Resolve the PRD path, epic ID, optional profile, optional `--base <git-ref>`, and optional `--quality`. If no epic is named, select the first epic at `IN_REVIEW`, otherwise the first epic whose non-cancelled children are implemented but not certified.
2. Read the PRD and adjacent status data. Extract the outcome, definition of done, non-goals, child stories, dependencies, acceptance criteria, explicit quality gates, and excluded scope.
3. Include every non-cancelled child, including stories already marked `IN_REVIEW` or `DONE`, and order them by dependency.
4. `IN_REVIEW` is the expected inbound state from `implement-epic` and carries no proof. If a child is `TODO`, `IN_PROGRESS`, or `BLOCKED`, inspect only enough evidence to distinguish stale status from incomplete implementation. Correct stale status after review; stop when implementation is genuinely incomplete.
5. Capture branch, HEAD, worktree state, and the review baseline. Prefer explicit `--base`; otherwise use the repository default branch merge-base only when it isolates the epic unambiguously. Include relevant committed, staged, unstaged, and untracked changes.
6. Map every changed artifact to a child criterion, the epic outcome, or a shared integration point. Preserve unrelated work. Use callers and imports as read-only context.
7. Do not blanket-exclude lockfiles, generated files, or binary artifacts. Review their source or metadata and validate source-to-artifact consistency when they affect the epic.
8. Stop with the exact ambiguous files and baseline when several epics cannot be isolated safely.
9. Select the risk profile and create one compact evidence map:

```text
criterion | PASS | FAIL | MANUAL_PROVEN | MANUAL_PENDING
evidence  | entry point, file:line, test, command, artifact, observation, or missing proof
```

**Gate:** The epic contract, ordered stories, exact diff, profile, and criterion map are known.

## Conditional Evidence

Use the PRD, local code, tests, history, and installed metadata as the default evidence.

Retrieve external evidence only for one named current or version-sensitive claim that can change a criterion verdict, finding, remediation, or validation gate:

| Need | Route |
|---|---|
| Repository behavior or cross-module flow | Targeted local inspection |
| Exact API, SDK, framework, CLI, or platform contract | Official documentation for the implicated version |
| Current advisory, standard, or external behavior | Current primary-source web evidence |

Use one route per question and stop when the claim is resolved. Record the supported fact, source pointer, and review impact. External evidence must not rewrite the product outcome, non-goals, or acceptance criteria without explicit user authority. Do not launch generic research.

## Phase 2: AUDIT

1. Read the complete epic diff once from the baseline. Read full files only where the diff lacks enough context.
2. Establish reachability before judging anything else. Read the epic in reverse: start from the repository's real execution roots (route table, command registry, rendered component tree, dependency container, migration list, scheduler, event subscriptions, config readers, public package exports) and determine where the new behavior enters. A diff shows what was added; missing wiring is an absence and never appears in it, so a diff-only audit is structurally blind to this class of defect.
3. Run the wiring checks against the final repository state, not the diff:
   - Every symbol the epic adds or modifies has at least one referent outside its own module and outside tests. An exported symbol whose only referents are tests is `BLOCKING` unless the PRD names it as a public API or extension point.
   - Every criterion traces from an execution root to the implementing code.
   - When new code replaces an existing path, the old path is no longer reachable. Both paths wired means runtime behavior did not change despite a clean diff.
   - Every added config key, environment variable, or feature flag is actually read, and its default exists in the manifest the project really loads.
   - A static reference proves linkage, not reachability. Where entry is conditional (route never registered, dead branch, component imported but never rendered, flag never enabled), require an execution observation: a test through the entry point, a log, a trace, or an explicit manual check. Record `MANUAL_PENDING` when unavailable.
4. Map every acceptance criterion and the epic definition of done to evidence. Use `PASS` only for current proof reached from a real entry point, `FAIL` for contradicted, unreachable, or missing required behavior, `MANUAL_PROVEN` for an observed manual check, and `MANUAL_PENDING` for a required observation not yet available.
5. Review cross-story composition in dependency order: shared contracts, state transitions, unhappy paths, regressions, and criteria that work alone but fail together.
6. Audit only applicable correctness, error handling, tests, performance cliffs, dependencies, and security boundaries.
7. Classify each locally supported issue:
   - `BLOCKING`: violates a criterion, definition of done, correctness or security invariant, leaves required behavior unreachable from a real entry point, or creates a concrete regression.
   - `NON_BLOCKING`: confirmed in-scope improvement that does not affect certification. All maintainability findings land here.
   - `OUTSIDE_EPIC`: valid observation without authority to change this epic.
8. Require each finding to include `file:line`, affected criterion or outcome, concrete failure path, and smallest complete correction. Deduplicate findings by root cause. Ignore style preferences and unsupported speculation.
9. Independent review is optional and capped. For DEEP, use at most one fresh read-only reviewer on a named high-risk axis, `correctness` or `security`. With explicit `--quality`, add at most one `maintainability` reviewer at any profile. Supply the epic contract, the same baseline and worktree state this review uses, the named axis, and minimum context. For `maintainability`, supply the smell baseline below and require findings scoped to the epic diff. Validate every returned finding directly against local code before retaining it.

**Gate:** Every criterion is accounted for, reachability is established or explicitly pending, and every retained finding has local evidence.

## Phase 3: REMEDIATE

1. Consolidate confirmed findings by root cause, affected story, and shared integration point.
2. Apply one coherent repair pass for all `BLOCKING` findings, wiring defects first. Apply `NON_BLOCKING` changes only when they are directly in scope, low-risk, and have a clear proof oracle.
3. Maintainability findings never gate certification. Apply one only when it stays inside the epic diff and does not restructure code the epic did not touch. Report the rest without acting.
4. Preserve unrelated work and product intent. Leave the affected criterion unresolved when correction requires an unapproved irreversible decision, destructive migration, or architecture expansion.
5. Add or update regression tests for mechanically testable defects, entering through the real execution path. Inspect every patched line and its immediate callers.
6. If a repair fails, make one targeted alternate attempt based on new evidence. If the same cause survives two distinct approaches, record the exact blocker and stop repairing that path.
7. Do not rerun the complete audit. Recheck the repaired failure paths and update the evidence map.

**Gate:** No confirmed blocking finding remains without a concrete blocker.

## Phase 4: VERIFY/STATUS

After the last code change, including any maintainability repair, run one coherent final validation bundle:

1. Explicit PRD quality gates applicable to the epic surface.
2. Acceptance and regression tests for every child story.
3. Cross-story and integration tests for shared contracts.
4. Configured type, build, lint, and format checks that apply to changed files and are not subsumed by an aggregate project command.
5. The relevant full suite when required by the PRD, the profile is DEEP, a public contract changed, or regression scope cannot be bounded.
6. Existing security or supply-chain checks when manifests, dependencies, trust boundaries, or sensitive behavior changed.

If validation fails, fix the specific cause, inspect the repair, rerun affected checks, then produce the final bundle required by the resulting state. Only evidence from the successful bundle after the last change certifies the epic.

Then:

1. Finalize every criterion as `PASS`, `FAIL`, `MANUAL_PROVEN`, or `MANUAL_PENDING`.
2. Confirm the final diff contains only scoped epic work and preserves unrelated changes.
3. Set `reviewed_at` using repository conventions for every fully audited, non-cancelled child.
4. Set a story to `DONE` only when every criterion is `PASS` or `MANUAL_PROVEN` from a real entry point, required gates pass, and no blocking finding remains. Use `IN_REVIEW` for pending manual proof or actionable correction, and `BLOCKED` for external dependencies, missing irreversible decisions, or repeated technical failure.
5. Downgrade a previously `DONE` story when current evidence disproves completion. Preserve a valid existing `completed_at`; set it for newly certified stories.
6. Recalculate story counters, epic status, and PRD status. The epic is `DONE` only when every child is `DONE` or `CANCELLED`.
7. Save and validate the status data, then return a compact receipt: epic and profile, per-story and epic verdicts, criterion evidence with entry points, wiring defects found, fixed findings, unapplied maintainability findings, final commands and results, status changes, manual proof, and blockers.

## Failure Handling

| Scenario | Action |
|---|---|
| PRD or epic not found | Inspect repository conventions and report attempted paths or available IDs |
| No eligible or reviewable epic | Report the status and baseline evidence; do not infer implementation from the tracker |
| Diff cannot be isolated | Report the ambiguous files and require a narrower base or isolated branch |
| Behavior is implemented but unreachable | Wire it when the entry point is unambiguous; otherwise mark the criterion `FAIL` and report the exact wiring gap |
| Required external evidence is unavailable | Use one official fallback route; if unresolved, mark the claim unverified |
| Same cause survives two distinct repair approaches | Mark affected criteria and stories `BLOCKED` |
| Required manual or external proof is unavailable | Preserve truthful `MANUAL_PENDING` or `BLOCKED` state |
| Epic is already reviewed | Stop unless re-review or an explicit epic was requested |

## Done When

- Every child criterion and the epic definition of done have current evidence reached from a real entry point.
- No behavior the epic adds is orphaned, unreachable, or shadowed by a replaced path that stays wired.
- Every retained finding is locally confirmed and classified.
- The complete final validation bundle passes after the last code change.
- The status tracker matches the evidence and unresolved work.
- No manual proof, blocker, unrelated change, or residual uncertainty is hidden.

## Constraints

- Preserve the epic boundary, dependency order, repository conventions, and unrelated user work.
- Prefer local evidence and deterministic checks over model assertions.
- Never treat tracker status as implementation proof, weaken tests, fabricate findings, or mark pending manual proof as complete.
- Never let a maintainability finding block certification or expand the diff beyond the epic.
- Never commit, push, publish, perform destructive external actions, or expand product scope without explicit authorization.

## Maintainability Baseline

Supplied to the `maintainability` reviewer only, under `--quality`. A documented repository standard always overrides an entry here, and anything tooling already enforces is skipped. Every entry is a labelled judgement call, never a hard violation.

- **Mysterious Name**: a name that does not reveal what it does or holds. Rename it; if no honest name comes, the design is murky.
- **Duplicated Code**: the same logic shape in more than one place in the change. Extract it, call it from both.
- **Feature Envy**: a function reaching into another object's data more than its own. Move it onto the data it envies.
- **Data Clumps**: the same few fields or params always travelling together. Bundle them into one type.
- **Primitive Obsession**: a primitive or string standing in for a domain concept. Give the concept its own small type.
- **Repeated Switches**: the same cascade on the same type recurring across the change. Replace with polymorphism or one shared map.
- **Shotgun Surgery**: one logical change forcing scattered edits across many files. Gather what changes together.
- **Divergent Change**: one module edited for several unrelated reasons. Split it so each changes for one reason.
- **Speculative Generality**: abstraction, params, or hooks added for needs the PRD does not have. Delete it.
- **Message Chains**: long navigation the caller should not depend on. Hide the walk behind one method.
- **Middle Man**: a unit that mostly delegates onward. Cut it, call the real target directly.
- **Refused Bequest**: an implementer ignoring or overriding most of what it inherits. Use composition.

## Examples

- `/review-epic tasks/prd-notifications.md EP-002`
- `/review-epic tasks/prd-billing.md EP-003 --profile deep`
- `/review-epic tasks/prd-search.md EP-001 --base feature/search-start`
- `/review-epic tasks/prd-search.md EP-001 --quality`
