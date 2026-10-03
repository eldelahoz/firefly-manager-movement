#Requires -Version 7.0
# Blocks pushes that update remote 'main' (CONTRIBUTING.md - Git Workflow).
# Invoked by lefthook pre-push; receives the git pre-push ref list on stdin:
#   <local-ref> <local-oid> <remote-ref> <remote-oid>
# Manual invocation (no redirected stdin) falls back to a HEAD-branch heuristic.

$block = $false

if ([Console]::IsInputRedirected) {
    while ($null -ne ($line = [Console]::In.ReadLine())) {
        $cols = $line -split '\s+'
        if ($cols.Count -ge 3 -and $cols[2] -eq 'refs/heads/main' -and $cols[1] -notmatch '^0+$') {
            $block = $true
        }
    }
}
else {
    $head = git symbolic-ref --quiet HEAD 2>$null
    if ($head -eq 'refs/heads/main') {
        $ahead = git rev-list --count 'origin/main..main' 2>$null
        if ($LASTEXITCODE -eq 0 -and [int]$ahead -gt 0) { $block = $true }
    }
}

if ($block) {
    Write-Error @'
Direct push to 'main' is not allowed (see CONTRIBUTING.md - Git Workflow).

Use the PR workflow instead:
  git switch -c <type>/<scope>-<short-description>
  git push -u origin HEAD
  gh pr create

Emergency bypass is restricted to documented hotfixes. --no-verify is not a
substitute: CI and GitHub branch protection will still reject direct pushes.
'@
    exit 1
}
exit 0
