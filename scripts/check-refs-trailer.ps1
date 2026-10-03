#Requires -Version 7.0
# Enforces the "Refs: RF-XX[, NFR-YY]" footer when a commit stages files under
# docs/specs/requirements/** (AGENTS.md Documentation Sync, rule 3).
# Invoked by lefthook commit-msg; $MessageFile binds to the first hook argument.

param([Parameter(Mandatory)][string]$MessageFile)

$staged = git diff --cached --name-only
$touchesSpec = @($staged | Where-Object { $_ -like 'docs/specs/requirements/*' })

if ($touchesSpec.Count -eq 0) { exit 0 }

$msg = Get-Content $MessageFile -Raw -Encoding UTF8
if ($msg -notmatch '(?m)^Refs:\s+(RF-\d+|NFR-\d+)(\s*,\s*(RF-\d+|NFR-\d+))*\s*$') {
    Write-Error @"
Commits touching requirement docs must carry a 'Refs: RF-XX[, NFR-YY]' footer.

Requirement files staged in this commit:
$($touchesSpec -join "`n")

Add a footer like:

  Refs: RF-06, NFR-02

See AGENTS.md - Documentation Sync (rule 3) and scripts/matrix-check.ps1.
"@
    exit 1
}
exit 0
