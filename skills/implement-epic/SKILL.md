---
name: implement-epic
description: "Research, scope-correct, implement, test, review, retest, verify, and complete one PRD epic (`EP-NNN`) autonomously. Use when asked to implement or finish an epic end to end and certify it `DONE` with current Exa web evidence, Context7 documentation, two post-change test passes, review remediation, and status roll-up."
---

# implement-epic

Implement: $ARGUMENTS

## Objective

Finish one complete epic and certify it only when the final repository state proves every requirement.

`SCOPE -> RESEARCH -> SCOPE VALIDATION -> IMPLEMENT -> TESTS A -> REVIEW -> TESTS B -> VERIFY -> STATUS`

Continue through the entire workflow without pausing for ordinary implementation choices, findings, test failures, or repairable regressions. Print one compact progress line per phase. Maintain a `Proof Ledger` under 400 tokens containing: epic ID, ordered stories, baseline, research questions and sources, scope amendments, acceptance proof, changed files, risk axes, test bundles and results, review findings, status path, decisions, and external blockers. Update it instead of repeating plans.

## Profiles

Select the lowest profile covering the highest-risk incomplete story. Profiles change review and regression depth, never whether mandatory research, both test passes, or final verification run.

| Profile | Use when | Review depth | Regression depth |
|---|---|---|---|
| FAST | Isolated behavior, established in-repo pattern, bounded impact, no sensitive boundary | Orchestrator review | Targeted acceptance and configured surface gates |
| DEFAULT | Shared integration, public behavior, multi-module change, or familiar dependency work | Orchestrator plus one fresh correctness review when available | Changed-surface integration gates and broader tests when the contract is shared |
| DEEP | Auth, payments, PII, destructive operations requiring user approval, migration, untrusted input, crypto, LLM tools, cross-service behavior, or unfamiliar critical logic | Independent correctness and security axes when distinct | Full relevant suite plus applicable security or supply-chain gates |

File count alone does not select DEEP. An explicit profile may escalate but never hide a DEEP condition.

## Phase 1 - SCOPE

1. Parse the PRD path, epic ID, and optional profile. If no epic is provided, select the first epic with incomplete stories and record the choice.
2. Read the PRD, adjacent status JSON, repository guidance, and only the code needed to locate the epic's integration points. Extract the outcome, non-goals, child stories, dependencies, acceptance criteria, explicit quality gates, and current statuses.
3. Exclude `DONE` and `CANCELLED` stories unless the user explicitly requests reimplementation. Topologically order the rest. Inspect an incomplete dependency before declaring it external or blocking.
4. Capture branch, HEAD, worktree state, and the epic baseline. Preserve unrelated user changes and map every candidate file to a story, criterion, or shared integration point.
5. Inspect manifests and lockfiles to identify exact frameworks, libraries, SDKs, APIs, CLIs, and versions implicated by the epic.
6. Define specific research questions, scope assumptions to validate, risk axes, acceptance proof, Test A, Test B, and the final verification checklist.
7. Build the Proof Ledger and a one-screen execution card.

**Gate:** The epic boundary, ordered stories, baseline, dependency state, research questions, affected versions, proof plan, and downstream gates are explicit.

## Phase 2 - RESEARCH

Research is mandatory and validates the proposed scope before implementation. Read [Phase Protocols](references/phase-protocols.md) once before starting this phase.

1. Use the Exa MCP first for targeted current web research. Cover the domain expectations, applicable standards or security guidance, known implementation hazards, and external behavior that could invalidate the scope.
2. Prefer primary sources. Fetch the full source when search excerpts are insufficient. If Exa is unavailable or fails, use the current CLI's native route defined in Phase Protocols.
3. Run at least one targeted Context7 lookup for the epic's primary framework, SDK, API, CLI, or platform, then cover every other version-sensitive contract implicated by the epic. Resolve the library ID, select the exact installed version when indexed, and query concrete API or configuration questions. If nothing relevant is indexed, record the attempted lookup and verify against official documentation through the approved fallback.
4. Compare external evidence with the PRD, local code, manifests, tests, and established project patterns. Research does not override current repository behavior without evidence that the behavior is defective or obsolete.
5. Record a compact research brief: claim, primary source or Context7 ID, scope impact, implementation constraint, test implication, and remaining uncertainty.
6. Continue until every research question is answered well enough to accept or amend the scope. Do not collect generic background that cannot change a decision or test.

**Gate:** Exa or its approved fallback has produced current web evidence, at least one targeted Context7 lookup is recorded, all implicated version-sensitive contracts are covered, and each finding has an explicit scope or test impact.

## Phase 3 - SCOPE VALIDATION

1. Classify each research finding as `CONFIRMS`, `AMENDS`, or `OUTSIDE_EPIC`.
2. Correct the PRD and status tracker when evidence exposes a missing constraint, invalid acceptance criterion, broken dependency, obsolete API assumption, or required quality gate. Preserve the original epic outcome and non-goals.
3. Do not expand the epic with optional capabilities. Put nonessential discoveries outside the epic and leave implementation unchanged.
4. Resolve ordinary ambiguity autonomously from the PRD, code, project conventions, Exa evidence, and Context7 docs. Prefer the smallest reversible decision that satisfies the outcome and record it.
5. Re-read the amended epic, rebuild the dependency order and criterion-to-proof map, and validate the PRD or status schema with an existing repository check when available.

**Gate:** The final scope is internally consistent, evidence-backed, dependency-ordered, testable, and unchanged in product intent.

## Phase 4 - IMPLEMENT

1. Set the epic, PRD, and first eligible story to `IN_PROGRESS`.
2. Implement incomplete stories in dependency order. Read each file before editing it and batch adjacent slices that share an integration point.
3. Add or update tests that directly prove acceptance criteria, unhappy paths, boundary conditions, and repaired regressions.
4. Use focused checks during implementation when they resolve an active ambiguity or protect a contract needed by later slices. These checks do not replace Test A.
5. Update the Proof Ledger after each slice with changed files, decisions, criterion coverage, and expected proof. Do not mark stories `DONE`.
6. Keep changes inside the validated epic. If implementation evidence invalidates the scope, return to Phase 3 and replay every downstream phase.

**Gate:** Every eligible story is implemented, every criterion has a proof path, and the changed surface matches the validated scope.

## Phase 5 - TESTS A

Run the first complete post-implementation test pass on the current implementation:

1. Explicit PRD quality gates applicable to the changed surface.
2. Acceptance tests and regression tests for every story.
3. Cross-story and integration tests for shared contracts.
4. Configured type, build, lint, and format checks that apply to changed files.
5. The relevant full suite when the PRD requires it, the epic changes a shared or public contract, regression scope cannot be bounded, or the profile is DEEP.
6. Existing supply-chain or security checks when manifests, trust boundaries, or sensitive behavior changed.

Prefer aggregate project scripts without skipping distinct gates they do not cover. Diagnose every failure, fix its root cause, and rerun the failed command plus impacted tests. Complete the full Test A bundle again whenever a fix changes shared behavior or test infrastructure.

**Gate:** The entire Test A bundle passes on one coherent repository state and every criterion has recorded automated or explicitly identified manual proof.

## Phase 6 - REVIEW

1. Review the complete epic diff from the Phase 1 baseline against the validated scope, epic outcome, child criteria, local conventions, integration contracts, unhappy paths, security boundaries, and regression risks.
2. Apply the selected profile. Use fresh independent review when available, but keep the orchestrator responsible for validating every finding against local evidence.
3. Resolve every confirmed in-scope defect, regression, unsafe side effect, unanswered implementation question, and missing test regardless of severity. Exclude unsupported speculation and preferences that do not affect correctness or maintainability.
4. Inspect each repair and its immediate callers. Keep the review open until the final diff contains no unresolved confirmed finding.
5. If a repair changes scope assumptions, return to Phase 3. If it materially changes a contract or architecture, review the complete final diff again before proceeding.

**Gate:** The final diff is reviewed, all confirmed in-scope findings are fixed, repaired lines are inspected, and no unresolved correctness, security, scope, or regression question remains.

## Phase 7 - TESTS B

Prove that review remediation introduced no regression:

1. Rerun the complete Test A bundle on the post-review state.
2. Add and run focused regression tests for every review repair not already covered.
3. Run any additional gate made necessary by the repaired surface.

If anything fails, diagnose and fix it, return to Phase 6 to review the resulting code changes, then restart Test B. A partial rerun is not a completed second pass.

**Gate:** The complete Test B bundle and all review-specific regression tests pass on the exact code state entering VERIFY.

## Phase 8 - VERIFY

Verify the epic as a whole, not only its commands:

1. Map every acceptance criterion and epic definition-of-done clause to current `file:line`, test, command, artifact, or observed runtime evidence.
2. Confirm the final diff contains only validated epic work and preserved unrelated user changes.
3. Check cross-story composition, state transitions, error paths, security boundaries, data migrations, operational behavior, documentation, and rollback or compatibility concerns when applicable.
4. Confirm research constraints and exact-version Context7 contracts are reflected in the implementation.
5. Confirm both complete test passes apply to the final code state, every review finding is closed, and no expected artifact or status update is missing.

If VERIFY finds any error, gap, question, side effect, or regression, fix the root cause, return to the earliest affected phase, and replay every downstream gate. VERIFY passes only with zero unresolved in-scope issue.

**Gate:** All criteria and quality gates are proven on the final state with zero unresolved in-scope issue.

## Phase 9 - STATUS

1. Set a child story to `DONE` only when all its criteria are proven on the final state.
2. Set `completed_at` for newly completed stories, then roll up `stories_done`, epic status, and PRD status.
3. Set the epic to `DONE` only when every child is `DONE` or `CANCELLED` and the terminal checklist below passes.
4. Save and validate the status JSON.
5. Print `STATUS DONE` only after certification. Return a compact receipt containing the epic and profile, scope amendments, research evidence, completed stories, acceptance proof, changed files, both test bundles and results, review findings and repairs, verification evidence, and status changes.

**Gate:** The tracker says `DONE`, the certification checklist passes, and the receipt references the exact final evidence.

## Persistence and Recovery

- There is no maximum retry count. Continue while an in-scope path can make progress.
- Never repeat the same failing command with the same assumptions. After 2 retries with the same cause, identify the missing context, tool, observability, invariant, or approach, change it, and continue.
- Every code or scope repair invalidates downstream evidence. Return to the earliest affected phase and replay all later gates.
- Resolve ordinary questions autonomously. Use the smallest reversible evidence-backed choice and record it.
- Exhaust local inspection, Exa, Context7, native search fallback, alternative implementations, and narrower verification before declaring an external blocker.
- Missing credentials or permissions, unavailable required external systems with no substitute, required destructive or published actions without user approval, and evidence that only an authorized human can provide are external blockers. Report them honestly as `BLOCKED`; never fabricate `DONE`.

## Done When

- [ ] Mandatory Exa research or approved CLI fallback is recorded with primary sources.
- [ ] At least one targeted Context7 lookup is recorded and every implicated version-sensitive contract is covered.
- [ ] Research findings have been applied to and revalidated against the epic scope.
- [ ] Every non-cancelled story and acceptance criterion is proven on the final state.
- [ ] Test A passes completely before review.
- [ ] Review has zero unresolved confirmed in-scope finding.
- [ ] Test B passes completely after the final review repairs.
- [ ] VERIFY finds zero error, problem, unanswered in-scope question, unintended side effect, or regression.
- [ ] Story, epic, PRD, timestamps, and counters are consistent.
- [ ] The epic status is `DONE`.

## Error Handling

| Scenario | Action |
|---|---|
| PRD not found | Infer from the current directory and repository conventions; if absent, report attempted paths |
| Epic ID not found | With an explicit ID, report available IDs; otherwise select the first incomplete epic |
| Scope contradicts current evidence | Amend the scope within the original outcome, validate it, and continue from Phase 3 |
| Exa unavailable | Use Codex native web tools or Claude Code `WebSearch` and `WebFetch`, then record the fallback |
| Context7 unavailable or quota-limited | Run `whoami` once when relevant, use official documentation through the approved web route, record the gap, and never rely silently on memory |
| Test or verification fails | Diagnose, repair, and replay from the earliest invalidated gate |
| Review finds an issue | Fix it, inspect the repair, and continue through Test B and VERIFY |
| External blocker remains after alternatives | Save truthful `BLOCKED` status and report the exact missing authority or external state |
| Epic already `DONE` | Verify the recorded evidence and stop unless reimplementation or revalidation was explicitly requested |

## Constraints

- **Always:** Preserve the validated epic boundary, use Exa and Context7 as routed, cite research evidence, implement in dependency order, complete both test passes, repair every confirmed in-scope review issue, replay invalidated gates, preserve unrelated work, and certify status honestly.
- **Never:** Skip research because the implementation looks familiar, use model memory instead of available current docs, treat search snippets as proof when the source is needed, stop on a repairable failure, leave review findings for the user, mark manual proof as automated, weaken tests to obtain green, hide blockers, claim `DONE` with unresolved work, commit, push, publish, or perform destructive external actions without user approval.

## Examples

- `/implement-epic tasks/prd-notifications.md EP-002`
- `/implement-epic tasks/prd-billing.md EP-003 --profile deep`
- `/implement-epic tasks/prd-search.md EP-001`
