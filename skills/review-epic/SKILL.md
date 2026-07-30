---
name: review-epic
description: "Review and correct one already-implemented PRD epic (`EP-NNN`) as a single unit, then validate its final state and roll up story, epic, and PRD status. Use when asked to review an epic, validate an implemented EP-NNN, recheck a completed epic, or replace story-by-story review with one dependency-aware audit."
---

# review-epic

Review: $ARGUMENTS

## Objective

Prove one implemented epic against its outcome, child criteria, integration contracts, and applicable security boundaries. Correct confirmed defects and certify only the final repository state.

`SCOPE -> AUDIT -> REMEDIATE -> VERIFY/STATUS`

The epic is the execution unit. Child stories organize evidence and dependency reasoning; they never trigger separate review pipelines. Report only meaningful milestones, scope ambiguity, confirmed blockers, and the final receipt.

## Profiles

Select the lowest profile covering the highest-risk changed surface.

| Profile | Use when | Audit and validation |
|---|---|---|
| FAST | Isolated behavior, established local pattern, bounded impact, no sensitive boundary | Orchestrator audit plus acceptance and changed-surface gates |
| DEFAULT | Shared integration, public behavior, multi-module change, or familiar dependency work | Orchestrator audit plus acceptance, integration, and applicable project gates |
| DEEP | Auth, payments, PII, migration, untrusted input, crypto, destructive operations, LLM tools, cross-service behavior, or unfamiliar critical logic | Orchestrator audit, optional single-axis independent review, relevant full suite, and applicable security gates |

File count alone does not select DEEP.

## Phase 1: SCOPE

1. Resolve the PRD path, epic ID, optional profile, and optional `--base <git-ref>`. If no epic is named, select the first epic whose non-cancelled children are implemented but review is incomplete.
2. Read the PRD and adjacent status data. Extract the outcome, definition of done, non-goals, child stories, dependencies, acceptance criteria, explicit quality gates, and excluded scope.
3. Include every non-cancelled child, including stories already marked `DONE`, and order them by dependency.
4. If a child is `TODO`, `IN_PROGRESS`, or `BLOCKED`, inspect only enough evidence to distinguish stale status from incomplete implementation. Correct stale status after review; stop when implementation is genuinely incomplete.
5. Capture branch, HEAD, worktree state, and the review baseline. Prefer explicit `--base`; otherwise use the repository default branch merge-base only when it isolates the epic unambiguously. Include relevant committed, staged, unstaged, and untracked changes.
6. Map every changed artifact to a child criterion, the epic outcome, or a shared integration point. Preserve unrelated work. Use callers and imports as read-only context.
7. Do not blanket-exclude lockfiles, generated files, or binary artifacts. Review their source or metadata and validate source-to-artifact consistency when they affect the epic.
8. Stop with the exact ambiguous files and baseline when several epics cannot be isolated safely.
9. Select the risk profile and create one compact evidence map:

```text
criterion | PASS | FAIL | MANUAL_PROVEN | MANUAL_PENDING
evidence  | file:line, test, command, artifact, observation, or missing proof
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
2. Map every acceptance criterion and the epic definition of done to evidence. Use `PASS` only for current proof, `FAIL` for contradicted or missing required behavior, `MANUAL_PROVEN` for an observed manual check, and `MANUAL_PENDING` for a required observation not yet available.
3. Review cross-story composition in dependency order: shared contracts, state transitions, unhappy paths, regressions, and criteria that work alone but fail together.
4. Audit only applicable correctness, error handling, tests, performance cliffs, dependencies, and security boundaries.
5. Classify each locally supported issue:
   - `BLOCKING`: violates a criterion, definition of done, correctness or security invariant, or creates a concrete regression.
   - `NON_BLOCKING`: confirmed in-scope improvement that does not affect certification.
   - `OUTSIDE_EPIC`: valid observation without authority to change this epic.
6. Require each finding to include `file:line`, affected criterion or outcome, concrete failure path, and smallest complete correction. Deduplicate findings by root cause. Ignore style preferences and unsupported speculation.
7. For DEEP only, use at most one fresh read-only reviewer when a named high-risk axis justifies independent context. Choose correctness or security. Supply the epic contract, diff, named axis, and minimum context. Validate every returned finding directly against local code before retaining it.

**Gate:** Every criterion is accounted for and every retained finding has local evidence.

## Phase 3: REMEDIATE

1. Consolidate confirmed findings by root cause, affected story, and shared integration point.
2. Apply one coherent repair pass for all `BLOCKING` findings. Apply `NON_BLOCKING` changes only when they are directly in scope, low-risk, and have a clear proof oracle.
3. Preserve unrelated work and product intent. Leave the affected criterion unresolved when correction requires an unapproved irreversible decision, destructive migration, or architecture expansion.
4. Add or update regression tests for mechanically testable defects. Inspect every patched line and its immediate callers.
5. If a repair fails, make one targeted alternate attempt based on new evidence. If the same cause survives two distinct approaches, record the exact blocker and stop repairing that path.
6. Do not rerun the complete audit. Recheck the repaired failure paths and update the evidence map.

**Gate:** No confirmed blocking finding remains without a concrete blocker.

## Phase 4: VERIFY/STATUS

After the last code change, run one coherent final validation bundle:

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
4. Set a story to `DONE` only when every criterion is `PASS` or `MANUAL_PROVEN`, required gates pass, and no blocking finding remains. Use `IN_REVIEW` for pending manual proof or actionable correction, and `BLOCKED` for external dependencies, missing irreversible decisions, or repeated technical failure.
5. Downgrade a previously `DONE` story when current evidence disproves completion. Preserve a valid existing `completed_at`; set it for newly completed stories.
6. Recalculate story counters, epic status, and PRD status. The epic is `DONE` only when every child is `DONE` or `CANCELLED`.
7. Save and validate the status data, then return a compact receipt: epic and profile, per-story and epic verdicts, criterion evidence, fixed findings, final commands and results, status changes, manual proof, and blockers.

## Failure Handling

| Scenario | Action |
|---|---|
| PRD or epic not found | Inspect repository conventions and report attempted paths or available IDs |
| No eligible or reviewable epic | Report the status and baseline evidence; do not infer implementation from the tracker |
| Diff cannot be isolated | Report the ambiguous files and require a narrower base or isolated branch |
| Required external evidence is unavailable | Use one official fallback route; if unresolved, mark the claim unverified |
| Same cause survives two distinct repair approaches | Mark affected criteria and stories `BLOCKED` |
| Required manual or external proof is unavailable | Preserve truthful `MANUAL_PENDING` or `BLOCKED` state |
| Epic is already reviewed | Stop unless re-review or an explicit epic was requested |

## Done When

- Every child criterion and the epic definition of done have current evidence.
- Every retained finding is locally confirmed and classified.
- The complete final validation bundle passes after the last code change.
- The status tracker matches the evidence and unresolved work.
- No manual proof, blocker, unrelated change, or residual uncertainty is hidden.

## Constraints

- Preserve the epic boundary, dependency order, repository conventions, and unrelated user work.
- Prefer local evidence and deterministic checks over model assertions.
- Never treat tracker status as implementation proof, weaken tests, fabricate findings, or mark pending manual proof as complete.
- Never commit, push, publish, perform destructive external actions, or expand product scope without explicit authorization.

## Examples

- `/review-epic tasks/prd-notifications.md EP-002`
- `/review-epic tasks/prd-billing.md EP-003 --profile deep`
- `/review-epic tasks/prd-search.md EP-001 --base feature/search-start`
