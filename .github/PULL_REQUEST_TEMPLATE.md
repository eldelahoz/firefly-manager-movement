<!--
Title must follow: <type>(<scope>): <subject> - English, imperative, <= 100 chars.
Valid types: feat fix docs refactor test chore ci
Valid scopes: domain application infra presentation backend frontend matrix specs docs deps ci docker
-->

## What

One-paragraph summary of this change.

## Why

Link the requirements/NFRs and specs this implements. Required when touching
`docs/specs/requirements/**` or their implementation.

- Refs: RF-XX, NFR-YY
- Spec: docs/specs/requirements/...

## How verified

Mark every gate you ran. All applicable gates must be green.

- [ ] Backend changes: `wsl cargo fmt --all && wsl cargo clippy --workspace --all-targets -- -D warnings && wsl cargo test --workspace`
- [ ] Frontend changes: `pnpm lint && pnpm build && pnpm e2e`
- [ ] Spec/matrix changes: `pwsh scripts/matrix-check.ps1 -Fix` committed with no drift
- [ ] `pnpm exec commitlint --from=HEAD~<n>` green locally

## Risk

- [ ] Migrations included: tested `diesel migration run` + `revert`
- [ ] No secret exposed (`FF3_TOKEN` stays backend-only, NFR-02)
- [ ] No automatic mutation of Firefly III data without explicit user action (O4/O5)

## Checklist

- [ ] PR title follows `<type>(<scope>): <subject>` (<= 100 chars, English, imperative)
- [ ] Every commit message and this description are in English
- [ ] `Refs: RF-XX, NFR-YY` present in commits or description for spec or implementation changes
- [ ] CI gates green: `lint / lint-commits` + `lint / lint-pr-title` (`guard-main` runs on `main` after merge)
- [ ] No `--no-verify` (or hotfix justification documented below)
- [ ] Merged via this PR only - no direct push to `main`
- [ ] Merged as a merge commit (history preserved - no squash, no rebase)

Closes #

## Hotfix justification

Delete this section if not applicable. Reason `--no-verify` was used: ...
