---
name: implement-epic
description: "Implement or finish one PRD epic (`EP-NNN`) end to end, from scoped requirements through code, wiring proof, and final validation, then hand off for certification. Use when asked to implement, complete, or finish an epic. Routes external research only when a current or version-sensitive fact can change implementation or tests, and hands off at `IN_REVIEW` without ever certifying `DONE`."
---

# implement-epic

Implement: $ARGUMENTS

## Objective

Finish one epic with the smallest workflow that proves every requirement from a real execution path.

`SCOPE -> [GROUND if needed] -> IMPLEMENT -> VERIFY/HANDOFF`

Work autonomously through ordinary implementation choices and repairable failures. Report only meaningful milestones, scope changes, or blockers.

This skill never certifies. It owns `TODO -> IN_PROGRESS -> IN_REVIEW` and hands the epic to `review-epic`, which alone writes `DONE`, `BLOCKED`, or a downgrade.

## Profiles

Select the lowest profile covering the highest-risk incomplete story.

| Profile | Use when | Final validation |
|---|---|---|
| FAST | Isolated behavior, established local pattern, bounded impact, no sensitive boundary | Acceptance tests and configured changed-surface gates |
| DEFAULT | Shared integration, public behavior, multi-module change, or familiar dependency work | Acceptance, integration, and applicable project gates |
| DEEP | Auth, payments, PII, migration, untrusted input, crypto, destructive operations, LLM tools, cross-service behavior, or unfamiliar critical logic | Relevant full suite plus applicable security or supply-chain gates |

File count alone does not select DEEP.

## Phase 1: SCOPE

1. Resolve the PRD path, epic ID, and optional profile. If no epic is named, select the first epic with `TODO`, `IN_PROGRESS`, or `BLOCKED` stories.
2. Read the PRD, adjacent status data, repository guidance, and only the code needed to locate integration points.
3. Extract the outcome, non-goals, incomplete child stories, dependencies, acceptance criteria, explicit quality gates, and current statuses. Exclude `DONE`, `IN_REVIEW`, and `CANCELLED` stories unless reimplementation was requested.
4. Order incomplete stories by dependency. Inspect an incomplete dependency before calling it external or blocking.
5. Capture branch, HEAD, worktree state, and the epic baseline. Preserve unrelated changes.
6. Inspect implicated manifests and lockfiles. Map every candidate file and acceptance criterion to a story or shared integration point.
7. Identify the execution roots the epic must plug into: route table, command registry, rendered component tree, dependency container, migration list, scheduler, event subscriptions, config readers, or public package exports.
8. Create a compact evidence map with only: story or criterion, expected implementation path, entry point that reaches it, proving test or observation, and status. Do not create a new file unless the repository already uses one.
9. For each unresolved question, classify `source_need` as `none`, `codebase`, `docs`, or `web`.

**Gate:** Scope, dependency order, baseline, risk profile, execution roots, and criterion-to-proof paths are explicit.

## Conditional GROUND

Skip external research when the PRD, local code, installed metadata, and existing tests already establish the required behavior.

For a question that can change implementation or validation, use exactly one necessary route:

| Need | Route |
|---|---|
| Current repository behavior | Targeted local inspection |
| Version-sensitive API, SDK, framework, CLI, or platform contract | Official documentation for the implicated version; use Context7 only when it is the cheapest reliable route |
| Volatile external behavior, standard, or security requirement | Current primary-source web evidence |

Rules:

1. State the concrete question before retrieval.
2. Stop when the question has enough evidence to decide implementation or tests.
3. Record only the supported fact, source pointer, and concrete impact.
4. Prefer repository behavior unless current evidence proves it defective or obsolete.
5. External evidence may amend the technical plan, compatibility handling, or tests. It must not rewrite the product outcome, non-goals, or acceptance criteria without explicit user authority.
6. If evidence contradicts the requested product contract, report the contradiction instead of silently changing the target.
7. Do not invoke a second broad research workflow or collect generic background.

## Phase 2: IMPLEMENT

1. Set the epic, PRD, and first eligible story to `IN_PROGRESS` when the repository tracks those states.
2. Implement incomplete stories in dependency order. Read before editing and batch adjacent work that shares an integration point.
3. Wire every new behavior into a real execution path in the same pass, using the roots identified in SCOPE. Wiring is part of the story, not a follow-up. An exported symbol nothing calls is unfinished work, not perfectible code.
4. When new code replaces an existing path, remove or redirect the old path. Leaving both wired means runtime behavior did not change despite a complete diff.
5. Make every added config key, environment variable, or feature flag actually read, with its default present in the manifest the project really loads.
6. Add or update tests that prove acceptance criteria, relevant unhappy paths, boundaries, and repaired regressions. At least one proof per criterion must start from the real entry point. A unit test on the new module alone does not prove the criterion.
7. Run focused checks during implementation when they resolve an active ambiguity or protect a contract needed by later work.
8. Update the evidence map as criteria gain concrete proof.
9. Keep changes inside the scoped epic. If implementation exposes a product-level contradiction, stop and report it. If it exposes only a technical correction, update the plan and continue.

**Gate:** Every eligible story is implemented, every criterion has a proof path starting at a real entry point, every symbol the epic adds has a referent outside its own module and outside tests, no replaced path stays wired, and the changed surface matches the epic.

## Phase 3: VERIFY/HANDOFF

After the last code change, run one coherent final validation bundle:

1. Explicit PRD quality gates applicable to the changed surface.
2. Acceptance and regression tests for every story.
3. Cross-story or integration tests when a contract is shared.
4. Configured type, build, lint, and format checks that apply to changed files.
5. The relevant full suite when required by the PRD, the profile is DEEP, a public contract changed, or regression scope cannot be bounded.
6. Existing security or supply-chain checks when manifests, trust boundaries, or sensitive behavior changed.

If validation fails, fix the root cause, inspect the repair, rerun the failed and impacted checks, then rerun the final bundle required by the resulting state. Only the successful bundle after the last change is handoff evidence.

Then:

1. Map every acceptance criterion to current `file:line`, its entry point, and a test, command, artifact, or explicit manual observation.
2. Confirm the final diff contains only scoped epic work and preserves unrelated user changes.
3. Set every implemented non-cancelled child story to `IN_REVIEW`. Mark manual proof as manual. Never set a story, epic, or PRD to `DONE`.
4. Roll up timestamps, counters, and epic status to `IN_REVIEW`. Leave `completed_at` unset; it belongs to certification. Validate the tracker using an existing repository check when available.
5. Print `STATUS IN_REVIEW` when every non-cancelled child is implemented, wired, and validated. Print `STATUS BLOCKED` otherwise.
6. Return a compact receipt: epic and profile, scope corrections, implemented stories, criterion evidence with entry points, changed files, final validation results, status changes, blockers, and the `review-epic` invocation to run next.

## Failure Handling

| Scenario | Action |
|---|---|
| PRD or epic not found | Inspect repository conventions and report the attempted paths or available IDs |
| Required external evidence is unavailable | Use one official fallback route; if still unresolved, state the gap and do not invent the contract |
| Same cause survives two distinct repair approaches | Report `BLOCKED` unless a new observable fact changes the diagnosis |
| Repair changes code after validation | Invalidate only dependent evidence and produce a new final bundle for the resulting state |
| Credentials, permissions, external systems, or human-only proof are required | Preserve truthful state and report the exact blocker |
| Behavior cannot be wired without an unapproved architectural decision | Report the exact wiring gap; do not ship an orphaned module as complete |
| Epic is already `IN_REVIEW` or `DONE` | Stop and point to `review-epic` unless reimplementation was explicitly requested |

## Done When

- Every non-cancelled story and acceptance criterion has current evidence reachable from a real entry point.
- Every symbol the epic adds has a non-test referent, and no replaced path stays wired.
- The complete final validation bundle passes after the last code change.
- External facts used by the implementation have real source pointers.
- Story, epic, PRD, timestamps, and counters are consistent at `IN_REVIEW`.
- No blocker or residual uncertainty is hidden.

## Constraints

- Preserve the epic boundary, repository conventions, and unrelated user work.
- Prefer local evidence and deterministic checks over model assertions.
- Never weaken tests, fabricate proof, or claim validation from an earlier code state.
- Never mark a story, epic, or PRD `DONE`. Certification belongs to `review-epic`.
- Do not audit style, abstraction quality, or maintainability. Fix only what blocks a criterion; leave quality judgement to review.
- Never commit, push, publish, perform destructive external actions, or expand product scope without explicit authorization.

## Examples

- `/implement-epic tasks/prd-notifications.md EP-002`
- `/implement-epic tasks/prd-billing.md EP-003 --profile deep`
- `/implement-epic tasks/prd-search.md EP-001`
