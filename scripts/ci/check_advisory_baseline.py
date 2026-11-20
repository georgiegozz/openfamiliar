#!/usr/bin/env python3
"""Compare cargo audit, cargo deny, and pnpm audit reports against a baseline.

Known IDs are printed on every run and are not a silent ignore. The job fails
when a new ID appears, when a baseline entry has expired, or when a baseline
entry is no longer present and should be removed.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import date, datetime
from pathlib import Path
from typing import Any


DEFAULT_BASELINE = Path(__file__).resolve().parent / "advisory-baseline.json"
RUSTSEC_RE = re.compile(r"RUSTSEC-\d{4}-\d+")
GHSA_RE = re.compile(r"GHSA-[0-9a-z]{4}-[0-9a-z]{4}-[0-9a-z]{4}")


class Finding:
    def __init__(self, ident: str, package: str, kind: str, source: str, title: str = "") -> None:
        self.id = ident
        self.package = package
        self.kind = kind
        self.source = source
        self.title = title

    def key(self) -> str:
        return self.id


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--baseline",
        type=Path,
        default=DEFAULT_BASELINE,
        help="Baseline JSON (default: scripts/ci/advisory-baseline.json)",
    )
    parser.add_argument("--ecosystem", choices=("rust", "javascript"), required=True)
    parser.add_argument("--cargo-audit-json", type=Path)
    parser.add_argument("--cargo-deny-json", type=Path)
    parser.add_argument("--pnpm-audit-json", type=Path)
    parser.add_argument(
        "--today",
        default=None,
        help="YYYY-MM-DD override for expiry checks (tests only)",
    )
    return parser.parse_args(argv)


def load_json_file(path: Path) -> Any:
    text = path.read_text(encoding="utf-8")
    decoder = json.JSONDecoder()
    stripped = text.lstrip()
    # Some tools emit one value; cargo-deny may emit concatenated objects.
    objects: list[Any] = []
    while stripped:
        obj, end = decoder.raw_decode(stripped)
        objects.append(obj)
        stripped = stripped[end:].lstrip()
    if not objects:
        raise ValueError(f"{path}: no JSON value")
    if len(objects) == 1:
        return objects[0]
    return objects


def parse_date(value: str) -> date:
    return datetime.strptime(value, "%Y-%m-%d").date()


def load_baseline(path: Path, ecosystem: str) -> list[dict[str, Any]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    entries = data.get(ecosystem)
    if not isinstance(entries, list):
        raise ValueError(f"{path}: missing list for ecosystem {ecosystem!r}")
    by_id: dict[str, dict[str, Any]] = {}
    for entry in entries:
        ident = entry.get("id")
        expires = entry.get("expiresOn")
        if not isinstance(ident, str) or not ident:
            raise ValueError(f"{path}: baseline entry is missing id")
        if not isinstance(expires, str):
            raise ValueError(f"{path}: {ident} is missing expiresOn")
        parse_date(expires)
        if ident in by_id:
            raise ValueError(f"{path}: duplicate baseline id {ident}")
        by_id[ident] = entry
    return list(by_id.values())


def findings_from_cargo_audit(data: Any, source: str = "cargo-audit") -> list[Finding]:
    if isinstance(data, list):
        # audit-compatible multi-db
        found: list[Finding] = []
        for item in data:
            found.extend(findings_from_cargo_audit(item, source))
        return found
    vulnerabilities = (data or {}).get("vulnerabilities") or {}
    items = vulnerabilities.get("list") or []
    found = []
    for item in items:
        advisory = item.get("advisory") or {}
        package = item.get("package") or {}
        ident = advisory.get("id")
        if not ident:
            continue
        name = package.get("name") or "?"
        version = package.get("version") or ""
        pkg = f"{name}@{version}" if version else name
        found.append(
            Finding(
                ident,
                pkg,
                "vulnerability",
                source,
                str(advisory.get("title") or ""),
            )
        )
    return found


def findings_from_cargo_deny(data: Any) -> list[Finding]:
    objects = data if isinstance(data, list) else [data]
    found: list[Finding] = []
    for obj in objects:
        if not isinstance(obj, dict):
            continue
        if obj.get("type") != "diagnostic":
            # Maybe a raw diagnostic without wrapper.
            fields = obj.get("fields") if isinstance(obj.get("fields"), dict) else obj
        else:
            fields = obj.get("fields") or {}
        if not isinstance(fields, dict):
            continue
        severity = str(fields.get("severity") or "").lower()
        if severity and severity not in {"error", "deny"}:
            continue
        advisory = fields.get("advisory") or {}
        ident = advisory.get("id") if isinstance(advisory, dict) else None
        if not ident:
            match = RUSTSEC_RE.search(json.dumps(fields))
            ident = match.group(0) if match else None
        if not ident:
            continue
        package = fields.get("package") or fields.get("krate") or {}
        if isinstance(package, dict):
            name = package.get("name") or "?"
            version = package.get("version") or package.get("vers") or ""
            pkg = f"{name}@{version}" if version and name != "?" else name
        else:
            pkg = "?"
        kind = str(fields.get("code") or "advisory")
        title = str(fields.get("message") or advisory.get("title") or "")
        found.append(Finding(ident, pkg, kind, "cargo-deny", title))
    return found


def findings_from_pnpm_audit(data: Any, severities: set[str]) -> list[Finding]:
    advisories = (data or {}).get("advisories") or {}
    found: list[Finding] = []
    if isinstance(advisories, dict):
        for _key, advisory in advisories.items():
            if not isinstance(advisory, dict):
                continue
            severity = str(advisory.get("severity") or "").lower()
            if severity not in severities:
                continue
            ident = advisory.get("github_advisory_id") or ""
            if not ident:
                match = GHSA_RE.search(json.dumps(advisory))
                ident = match.group(0) if match else str(advisory.get("id") or "")
            if not ident:
                continue
            package = str(advisory.get("module_name") or "?")
            found.append(
                Finding(
                    ident,
                    package,
                    severity,
                    "pnpm-audit",
                    str(advisory.get("title") or ""),
                )
            )
    return found


def merge_findings(findings: list[Finding]) -> list[Finding]:
    merged: dict[str, Finding] = {}
    for finding in findings:
        existing = merged.get(finding.id)
        if existing is None:
            merged[finding.id] = finding
            continue
        if finding.source not in existing.source:
            existing.source = f"{existing.source}+{finding.source}"
        if existing.package in {"?", ""} and finding.package not in {"?", ""}:
            existing.package = finding.package
        if not existing.title and finding.title:
            existing.title = finding.title
    return [merged[key] for key in sorted(merged)]


def evaluate(
    findings: list[Finding],
    baseline: list[dict[str, Any]],
    today: date,
) -> tuple[list[str], list[str], list[str], list[str]]:
    present = {item.id: item for item in findings}
    known = {entry["id"]: entry for entry in baseline}

    new_ids: list[str] = []
    expired_ids: list[str] = []
    unused_ids: list[str] = []
    known_active: list[str] = []

    for ident, finding in present.items():
        entry = known.get(ident)
        if entry is None:
            new_ids.append(ident)
            continue
        expires = parse_date(entry["expiresOn"])
        if expires < today:
            expired_ids.append(ident)
        else:
            known_active.append(ident)

    for ident, entry in known.items():
        if ident not in present:
            unused_ids.append(ident)

    return new_ids, expired_ids, unused_ids, known_active


def render_line(finding: Finding | None, ident: str, status: str, expires: str = "") -> str:
    package = finding.package if finding else "-"
    kind = finding.kind if finding else "-"
    source = finding.source if finding else "-"
    title = (finding.title if finding else "").replace("\n", " ").strip()
    expiry = f" expiresOn={expires}" if expires else ""
    suffix = f" {title}" if title else ""
    return f"{status:12} {ident:22} {package:28} {kind:14} {source}{expiry}{suffix}"


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    today = parse_date(args.today) if args.today else date.today()
    try:
        baseline = load_baseline(args.baseline, args.ecosystem)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    findings: list[Finding] = []
    try:
        if args.ecosystem == "rust":
            if args.cargo_audit_json:
                findings.extend(findings_from_cargo_audit(load_json_file(args.cargo_audit_json)))
            if args.cargo_deny_json:
                findings.extend(findings_from_cargo_deny(load_json_file(args.cargo_deny_json)))
        elif args.ecosystem == "javascript":
            if not args.pnpm_audit_json:
                print("error: --pnpm-audit-json is required", file=sys.stderr)
                return 2
            findings.extend(
                findings_from_pnpm_audit(
                    load_json_file(args.pnpm_audit_json),
                    {"high", "critical"},
                )
            )
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"error: failed to read report: {exc}", file=sys.stderr)
        return 2

    findings = merge_findings(findings)
    new_ids, expired_ids, unused_ids, known_active = evaluate(findings, baseline, today)
    present = {item.id: item for item in findings}
    known = {entry["id"]: entry for entry in baseline}

    print(f"advisory-baseline ecosystem={args.ecosystem} today={today.isoformat()}")
    print(
        "status       id                     package                      kind           source"
    )
    for ident in known_active:
        print(
            render_line(
                present[ident],
                ident,
                "known",
                known[ident]["expiresOn"],
            )
        )
    for ident in new_ids:
        print(render_line(present[ident], ident, "NEW"))
    for ident in expired_ids:
        print(
            render_line(
                present.get(ident),
                ident,
                "EXPIRED",
                known[ident]["expiresOn"],
            )
        )
    for ident in unused_ids:
        print(
            render_line(
                None,
                ident,
                "UNUSED",
                known[ident]["expiresOn"],
            )
        )

    missing_issue = [
        entry["id"]
        for entry in baseline
        if not entry.get("issue")
        and entry["id"] in set(known_active) | set(expired_ids)
    ]
    placeholder_issue = [
        entry["id"]
        for entry in baseline
        if entry.get("issue") == "<ISSUE_URL>"
        and entry["id"] in set(known_active) | set(expired_ids)
    ]
    if missing_issue:
        print(
            "note: these baseline entries have no tracking issue yet: "
            + ", ".join(missing_issue),
            file=sys.stderr,
        )
    if placeholder_issue:
        print(
            "note: replace <ISSUE_URL> with the tracking issue after it is opened: "
            + ", ".join(placeholder_issue),
            file=sys.stderr,
        )

    failed = False
    if new_ids:
        print("error: new advisories are not in the baseline: " + ", ".join(new_ids), file=sys.stderr)
        failed = True
    if expired_ids:
        print(
            "error: grace period expired; this job fails until the package is upgraded "
            "or expiresOn is extended with a written reason: "
            + ", ".join(expired_ids),
            file=sys.stderr,
        )
        failed = True
    if unused_ids:
        print(
            "error: baseline ids are no longer reported; remove them from the baseline: "
            + ", ".join(unused_ids),
            file=sys.stderr,
        )
        failed = True

    if failed:
        return 1
    print("advisory-baseline: no new, expired, or unused entries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
