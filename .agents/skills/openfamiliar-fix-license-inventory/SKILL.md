---
name: openfamiliar-fix-license-inventory
description: Regenerate or repair docs/legal/dependency-licenses.json after a lockfile or dependency change. Use when licenses:check fails, Dependabot updates a lockfile, or the rust package array looks reordered.
---

# Fix the license inventory

`pnpm licenses:check` regenerates `docs/legal/dependency-licenses.json` and
requires `git diff --exit-code` on that file. The generator is
`scripts/legal/Export-DependencyLicenses.ps1`. It runs
`cargo metadata --format-version 1 --locked` and
`pnpm licenses list --prod --json`, sorts the results, and formats the file
with prettier.

The source of truth is **Windows**. CI job `js` runs on `windows-latest` and
invokes Windows PowerShell 5.1, whose `Sort-Object` uses word sort (hyphens
in crate names are ignored). PowerShell 7 on Linux uses ICU comparison and
can shuffle the `rust` array of the same package set. Details and a
confirmed 86-entry positional example are in
[verification.md](../../../docs/development/verification.md).

## Workflow

1. Confirm `Cargo.lock` and `pnpm-lock.yaml` are the versions you intend.
   The script uses `--locked`; it will fail if the Cargo lockfile is stale.
2. On Windows, from the repository root, regenerate and inspect the diff.
3. Review new or changed licenses against
   [allowed-licenses.md](../../../docs/legal/allowed-licenses.md).
4. If the only diff is a reorder of existing `rust` names, you are probably
   on Linux or on PowerShell 7. Discard that file and regenerate on Windows.
5. Run `pnpm licenses:check` on Windows before you commit.

## PowerShell (Windows)

```powershell
pnpm licenses:audit
git diff -- docs/legal/dependency-licenses.json
pnpm licenses:check
```

Do not hand-edit the JSON to "fix" order. Let the script rewrite it.

## Python

This helper detects a hyphen-sort shuffle. It does not regenerate the file.

```python
import json
from pathlib import Path

rows = json.loads(Path("docs/legal/dependency-licenses.json").read_text(encoding="utf-8"))["rust"]
names = [row["name"] for row in rows]
# Hyphen-significant order (Linux pwsh / ICU style).
icu = sorted(names)
# Word-sort approximation: ignore '-' like Windows PowerShell 5.1 word sort.
word = sorted(names, key=lambda name: name.replace("-", ""))
print("entries:", len(names))
print("matches ICU / hyphen-significant sort:", names == icu)
print("matches hyphen-stripped word-sort approximation:", names == word)
if names != icu and names == word:
    print("Looks like the Windows CI order. Keep this file.")
elif names == icu and names != word:
    print("Looks like a Linux pwsh regeneration. Do not commit; regenerate on Windows.")
```

The approximation is a check, not a reimplementation of `Sort-Object`. If it
is inconclusive, compare with a Windows run or with the `js` CI artifact.

## Acceptance checks

```powershell
pnpm licenses:check
git diff --exit-code -- docs/legal/dependency-licenses.json
```

On Linux, a failing `licenses:check` whose diff is only order is not a
license regression. Report that limitation and regenerate on Windows.

## Limits

- Do not invent licenses or replace `NOASSERTION` without checking the crate
  or package metadata.
- Do not run the audit against an unlocked `Cargo.lock`.
- Do not change `.github/workflows/` to "fix" the sort on Linux.
