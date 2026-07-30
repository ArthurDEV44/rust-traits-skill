---
name: implement-epic
description: "Implement or finish one PRD epic (`EP-NNN`) end to end, from scoped requirements through code, review, final validation, and status roll-up. Use when asked to implement, complete, or certify an epic. Routes external research only when a current or version-sensitive fact can change implementation or tests, and certifies `DONE` only from evidence on the final repository state."
---

# implement-epic

Implement: $ARGUMENTS

## Objective

Finish one epic with the smallest workflow that proves every requirement.

`SCOPE -> [GROUND if needed] -> IMPLEMENT -> REVIEW -> VERIFY/STATUS`

Work autonomously through ordinary implementation choices and repairable failures. Report only meaningful milestones, scope changes, or blockers.

## Profiles

Select the lowest profile covering the highest-risk incomplete story.

| Profile | Use when | Final validation |
|---|---|---|
| FAST | Isolated behavior, established local pattern, bounded impact, no sensitive boundary | Acceptance tests and configured changed-surface gates |
| DEFAULT | Shared integration, public behavior, multi-module change, or familiar dependency work | Acceptance, integration, and applicable project gates |
| DEEP | Auth, payments, PII, migration, untrusted input, crypto, destructive operations, LLM tools, cross-service behavior, or unfamiliar critical logic | Relevant full suite plus applicable security or supply-chain gates |

File count alone does not select DEEP.

## Phase 1: SCOPE

1. Resolve the PRD path, epic ID, and optional profile. If no epic is named, select the first epic with incomplete stories.
2. Read the PRD, adjacent status data, repository guidance, and only the code needed to locate integration points.
3. Extract the outcome, non-goals, incomplete child stories, dependencies, acceptance criteria, explicit quality gates, and current statuses. Exclude `DONE` and `CANCELLED` stories unless revalidation was requested.
4. Order incomplete stories by dependency. Inspect an incomplete dependency before calling it external or blocking.
5. Capture branch, HEAD, worktree state, and the epic baseline. Preserve unrelated changes.
6. Inspect implicated manifests and lockfiles. Map every candidate file and acceptance criterion to a story or shared integration point.
7. Create a compact evidence map with only: story or criterion, expected implementation path, proving test or observation, and status. Do not create a new file unless the repository already uses one.
8. For each unresolved question, classify `source_need` as `none`, `codebase`, `docs`, or `web`.

**Gate:** Scope, dependency order, baseline, risk profile, and criterion-to-proof paths are explicit.

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
3. Add or update tests that prove acceptance criteria, relevant unhappy paths, boundaries, and repaired regressions.
4. Run focused checks during implementation when they resolve an active ambiguity or protect a contract needed by later work.
5. Update the evidence map as criteria gain concrete proof. Do not mark stories `DONE`.
6. Keep changes inside the scoped epic. If implementation exposes a product-level contradiction, stop and report it. If it exposes only a technical correction, update the plan and continue.

**Gate:** Every eligible story is implemented, every criterion has a proof path, and the changed surface matches the epic.

## Phase 3: REVIEW

1. Review the complete epic diff from the baseline against the outcome, criteria, repository conventions, integration contracts, error paths, security boundaries, and regression risks.
2. Validate every finding against exact local evidence. Ignore unsupported speculation and taste-only preferences.
3. Fix every confirmed in-scope correctness, security, scope, or regression issue. Add a regression test when mechanically testable.
4. Inspect each repair and its immediate callers.
5. For DEEP only, use at most one independent review mode when a distinct high-risk axis justifies fresh context. Choose correctness or security, not both. The orchestrator validates every returned finding before acting.

**Gate:** The final diff has no unresolved confirmed in-scope finding.

## Phase 4: VERIFY/STATUS

After the last code change, run one coherent final validation bundle:

1. Explicit PRD quality gates applicable to the changed surface.
2. Acceptance and regression tests for every story.
3. Cross-story or integration tests when a contract is shared.
4. Configured type, build, lint, and format checks that apply to changed files.
5. The relevant full suite when required by the PRD, the profile is DEEP, a public contract changed, or regression scope cannot be bounded.
6. Existing security or supply-chain checks when manifests, trust boundaries, or sensitive behavior changed.

If validation fails, fix the root cause, inspect the repair, rerun the failed and impacted checks, then rerun the final bundle required by the resulting state. Only the successful bundle after the last change is certification evidence.

Then:

1. Map every acceptance criterion to current `file:line`, test, command, artifact, or explicit manual observation.
2. Confirm the final diff contains only scoped epic work and preserves unrelated user changes.
3. Mark a child story `DONE` only when all its criteria are proven. Mark manual proof as manual.
4. Roll up timestamps, counters, epic status, and PRD status. Validate the tracker using an existing repository check when available.
5. Print `STATUS DONE` only when every non-cancelled child is proven on the final state.
6. Return a compact receipt: epic and profile, scope corrections, completed stories, criterion evidence, changed files, final validation results, review repairs, status changes, and blockers.

## Failure Handling

| Scenario | Action |
|---|---|
| PRD or epic not found | Inspect repository conventions and report the attempted paths or available IDs |
| Required external evidence is unavailable | Use one official fallback route; if still unresolved, state the gap and do not invent the contract |
| Same cause survives two distinct repair approaches | Report `BLOCKED` unless a new observable fact changes the diagnosis |
| Repair changes code after validation | Invalidate only dependent evidence and produce a new final bundle for the resulting state |
| Credentials, permissions, external systems, or human-only proof are required | Preserve truthful state and report the exact blocker |
| Epic is already `DONE` | Stop unless reimplementation or revalidation was explicitly requested |

## Done When

- Every non-cancelled story and acceptance criterion has current evidence.
- The complete final validation bundle passes after the last code change.
- The final diff is reviewed and contains no unresolved confirmed in-scope issue.
- External facts used by the implementation have real source pointers.
- Story, epic, PRD, timestamps, and counters are consistent.
- No blocker or residual uncertainty is hidden.

## Constraints

- Preserve the epic boundary, repository conventions, and unrelated user work.
- Prefer local evidence and deterministic checks over model assertions.
- Never weaken tests, fabricate proof, or claim `DONE` from an earlier code state.
- Never commit, push, publish, perform destructive external actions, or expand product scope without explicit authorization.

## Examples

- `/implement-epic tasks/prd-notifications.md EP-002`
- `/implement-epic tasks/prd-billing.md EP-003 --profile deep`
- `/implement-epic tasks/prd-search.md EP-001`
