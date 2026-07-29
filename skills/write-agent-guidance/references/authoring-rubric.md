# Agent Guidance Authoring Rubric

Use this rubric to decide what belongs in an instruction chain and to preserve
equivalent behavior across Codex and Claude Code.

## Contents

- [Provider semantics](#provider-semantics)
- [Placement test](#placement-test)
- [Authoring test](#authoring-test)
- [Bridge patterns](#bridge-patterns)
- [Acceptance checks](#acceptance-checks)

## Provider semantics

### Codex

- Codex loads instructions once per run.
- Global discovery selects the first non-empty
  `$CODEX_HOME/AGENTS.override.md` or `$CODEX_HOME/AGENTS.md`.
- Project discovery walks from the project root to the working directory. In
  each directory it selects at most one file in this order:
  `AGENTS.override.md`, `AGENTS.md`, then configured fallback names.
- Files are concatenated root to leaf. Nearer project guidance appears later.
- The combined project chain stops at `project_doc_max_bytes`, 32 KiB by
  default.

Source: [OpenAI, Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md)

### Claude Code

- Claude Code discovers `CLAUDE.md`, not `AGENTS.md`.
- A line such as `@AGENTS.md` imports the target at session start; remaining
  `CLAUDE.md` content is appended.
- Relative imports resolve from the importing file. Imports outside a project
  may require first-use approval at project scope.
- Claude walks from the filesystem root toward the working directory and
  concatenates applicable `CLAUDE.md` and `CLAUDE.local.md` files. Nested files
  can load when Claude works in their subtree.
- Contradictory instructions may be followed arbitrarily. Imported files still
  consume context. Anthropic recommends fewer than 200 lines per `CLAUDE.md`.
- Use `.claude/rules/` for modular or path-scoped Claude-only rules and skills
  for task-specific procedures.

Source: [Anthropic, How Claude remembers your project](https://code.claude.com/docs/en/memory)

## Placement test

Keep a candidate only when it is durable, changes agent behavior, is not cheap
to infer, has a narrow valid scope, and can be checked. Then place it once:

| Candidate | Destination |
|---|---|
| One-off task constraint | Current prompt |
| Personal default across repositories | Global `AGENTS.md` |
| Shared durable repository rule | Root `AGENTS.md` |
| Subtree-specific rule | Nested `AGENTS.md` plus Claude bridge |
| Claude-only path rule | `.claude/rules/*.md` |
| Long architecture or policy knowledge | Linked repository documentation |
| Repeatable task procedure | Skill |
| Objective invariant | Test, type, linter, hook, or CI |
| Codex runtime setting | `.codex/config.toml` or another Codex surface |

Reject:

- repository tours that duplicate discoverable structure;
- generic advice such as "write clean code" or "be careful";
- exhaustive style rules already enforced by tooling;
- guessed commands, architecture, ownership, or product behavior;
- contradictory layers that depend on model precedence;
- temporary work state, credentials, PII, and machine-specific paths in shared
  repository guidance.

## Authoring test

Grade every retained line:

1. **Specificity**: name the exact behavior, boundary, command, or artifact.
2. **Fidelity**: preserve semantic strength; do not infer bans or permissions
   from a preference, observed pattern, or unmentioned alternative.
3. **Scope**: state where the rule applies.
4. **Actionability**: give the preferred action, not only a prohibition.
5. **Safe path**: for a guardrail, name the valid alternative or exception.
6. **Evidence**: tie factual claims to repository artifacts or user authority.
7. **Verification**: identify an inspectable completion signal.
8. **Uniqueness**: keep the meaning in one authoritative location.
9. **Context value**: remove the line if the agent behaves the same without it.

Prefer concise headings and short rules. Explain rationale only when it prevents
misapplication. Put consequential guidance before lower-risk conventions.

## Bridge patterns

Repository pair in the same directory:

```md
@AGENTS.md
```

Repository pair with a necessary Claude-only tail:

```md
@AGENTS.md

## Claude Code

- Apply the provider-specific rule here.
```

For global files in different configuration directories, compute and verify the
relative path. For every nested canonical file intended for both agents, create
the corresponding nested Claude bridge. Prefer imports over symlinks for
portability and for the ability to add a provider-specific tail.

## Acceptance checks

- The canonical source and all effective scopes are named.
- Every command, linked file, and import target exists.
- The import graph is acyclic.
- No effective chain contains duplicated or contradictory meanings.
- Shared guidance contains no secret, PII, or private machine-only detail.
- Each Claude-loaded file remains below the 200-line target.
- The combined Codex project chain remains below its configured byte limit.
- Root and representative nested directories produce the intended effective
  rules in fresh sessions when live validation is authorized.

Do not assume guidance improves performance. Recent empirical work reports
mixed outcomes: one study found higher inference cost without general success
gains, while another associated `AGENTS.md` with lower median runtime and token
usage. Favor non-standard practices that agents cannot infer, and use
representative A/B tasks when the performance impact matters.

Sources:
[Gloaguen et al., 2026](https://arxiv.org/abs/2602.11988);
[Lulla et al., 2026](https://arxiv.org/abs/2601.20404).
