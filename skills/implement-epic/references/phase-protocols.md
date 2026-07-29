# implement-epic phase protocols

Load this file once before mandatory RESEARCH. Reuse it for independent REVIEW.

## Web research routing

1. Inspect the tools exposed by the current runtime instead of guessing from environment variables.
2. Prefer the Exa MCP:
   - Search with `web_search_exa`.
   - Read promising primary sources with `web_fetch_exa` when available and excerpts are insufficient.
   - Write semantic queries describing the needed evidence, including the technology and version when relevant.
3. Treat Exa results as retrieval, not validation. Discard irrelevant matches, prefer current primary sources, and distinguish source facts from inference.
4. If the Exa tools are absent, unavailable, unauthorized, rate-limited, or repeatedly return unusable evidence:
   - Codex CLI: use the native web search and fetch tools exposed in the current session.
   - Claude Code: use `WebSearch`, then `WebFetch` for selected primary sources.
5. Do not use browser automation as a research fallback unless the user explicitly opted into browser control.
6. Record the route, query, source URL, claim, publication or update date when relevant, and scope or test impact.

Exa reference: [Web Search MCP](https://exa.ai/docs/reference/exa-mcp).

## Context7 documentation routing

Use Context7 separately from web research for version-sensitive library, framework, SDK, API, CLI, and cloud-service contracts.

1. Always perform at least one lookup for the primary framework, SDK, API, CLI, or platform used by the epic. Resolve installed versions from manifests, lockfiles, generated metadata, or current CLI output before querying.
2. Unless an exact Context7 ID is already known, run:

```bash
bunx ctx7@latest library <library-name> "<specific epic question>"
```

3. Select by exact name, official source, source reputation, coverage, benchmark score, and version fit. Use `/org/project/version` when the installed version is indexed.
4. Query the selected ID:

```bash
bunx ctx7@latest docs <library-id> "<specific API, behavior, migration, or configuration question>"
```

5. When the current runtime provides a configured Context7 integration instead of the CLI, use its equivalent library-resolution and documentation-query tools with the same selection rules.
6. Never include secrets, credentials, personal data, proprietary source, or private identifiers in a Context7 query.
7. If no relevant library is indexed, record the attempted query and use official documentation through Exa or the approved native web route.
8. Before reporting authentication or quota failure, run `bunx ctx7@latest whoami` once. If Context7 remains unavailable, retrieve official documentation through Exa or the approved native web route, record the fallback, and never substitute unverified model memory.

Context7 reference: [Context7 CLI documentation](https://github.com/upstash/context7/blob/master/skills/context7-cli/references/docs.md).

## Research brief

Keep research operational:

```text
Question: {scope assumption or implementation risk}
Route: {Exa | Codex native web | Claude WebSearch/WebFetch | Context7}
Evidence: {primary URL or Context7 library ID}
Finding: {exact supported claim}
Scope impact: {CONFIRMS | AMENDS | OUTSIDE_EPIC}
Implementation constraint: {concrete decision}
Test impact: {gate or case to add}
Uncertainty: {none or exact residual gap}
```

Do not retain a finding that changes neither scope, implementation, risk handling, nor tests.

## Review routing

The orchestrator always reviews the complete epic diff. Independent review adds a fresh risk axis:

| Profile | Independent review |
|---|---|
| FAST | None required |
| DEFAULT | One fresh correctness review when a reviewer is available |
| DEEP | One correctness review; add one security review only when the sensitive boundary is distinct |

If a preferred reviewer or subagent is unavailable, the orchestrator performs the same checklist directly. Delegation depth is one. Reviewers do not modify files, contact other agents, or broaden the epic.

## Independent correctness brief

```text
Independently review the final epic diff for correctness and regressions.

Proof Ledger:
{proof_ledger}

Validated scope and criteria:
{scope_and_acceptance}

Epic diff:
{changed_diff}

Read one-hop context only when the diff is insufficient. Check acceptance coverage, cross-story contracts, state transitions, error paths, concurrency, compatibility, performance cliffs, regressions, and project conventions.

Return only PASS or actionable findings:
- severity: CRITICAL | HIGH | MEDIUM | LOW
- evidence: file:line
- affected criterion or epic outcome
- concrete failure or side-effect path
- smallest complete repair

Do not report taste-only style preferences, modify files, or inspect unrelated work.
```

## Independent security brief

```text
Audit only the sensitive boundaries changed by this epic.

Proof Ledger:
{proof_ledger}

Sensitive boundaries:
{security_surfaces}

Epic diff:
{changed_diff}

Read one-hop context only when needed. Check applicable authorization, authentication, injection, secrets, validation, PII handling, SSRF, path traversal, cryptography, destructive operations, dependency risk, and LLM tool boundaries.

Return only PASS or actionable findings:
- severity: CRITICAL | HIGH | MEDIUM | LOW
- evidence: file:line
- concrete exploit or failure path
- affected criterion or invariant
- smallest complete remediation and regression test

Do not modify files or expand beyond the named boundaries.
```

## Review acceptance

The orchestrator must verify every independent finding against the cited code and local contracts before repairing it. A finding is closed only when:

1. The root cause is corrected.
2. A regression test exists when the behavior is mechanically testable.
3. Patched lines and immediate callers are inspected.
4. The final diff no longer contains the failure path.
5. Test B passes after the repair.
