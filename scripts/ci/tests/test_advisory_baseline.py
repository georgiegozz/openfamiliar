from __future__ import annotations

import json
import sys
import tempfile
import unittest
from io import StringIO
from pathlib import Path
from unittest.mock import patch


SCRIPTS_CI = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS_CI))

from check_advisory_baseline import main  # noqa: E402


def write_json(path: Path, payload: object) -> Path:
    path.write_text(json.dumps(payload), encoding="utf-8")
    return path


class AdvisoryBaselineTests(unittest.TestCase):
    def run_main(self, argv: list[str]) -> tuple[int, str, str]:
        stdout = StringIO()
        stderr = StringIO()
        with patch("sys.stdout", stdout), patch("sys.stderr", stderr):
            code = main(argv)
        return code, stdout.getvalue(), stderr.getvalue()

    def test_known_rust_advisory_passes_before_expiry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            baseline = write_json(
                root / "baseline.json",
                {
                    "rust": [
                        {
                            "id": "RUSTSEC-2099-0001",
                            "kind": "vulnerability",
                            "package": "example",
                            "expiresOn": "2099-12-31",
                            "issue": None,
                        }
                    ],
                    "javascript": [],
                },
            )
            audit = write_json(
                root / "audit.json",
                {
                    "vulnerabilities": {
                        "list": [
                            {
                                "advisory": {
                                    "id": "RUSTSEC-2099-0001",
                                    "title": "fictional",
                                },
                                "package": {"name": "example", "version": "1.0.0"},
                            }
                        ]
                    }
                },
            )
            code, out, err = self.run_main(
                [
                    "--ecosystem",
                    "rust",
                    "--baseline",
                    str(baseline),
                    "--cargo-audit-json",
                    str(audit),
                    "--today",
                    "2026-11-12",
                ]
            )
            self.assertEqual(code, 0, err)
            self.assertIn("known", out)
            self.assertIn("RUSTSEC-2099-0001", out)

    def test_new_advisory_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            baseline = write_json(
                root / "baseline.json",
                {"rust": [], "javascript": []},
            )
            audit = write_json(
                root / "audit.json",
                {
                    "vulnerabilities": {
                        "list": [
                            {
                                "advisory": {"id": "RUSTSEC-2099-0002", "title": "new"},
                                "package": {"name": "other", "version": "2.0.0"},
                            }
                        ]
                    }
                },
            )
            code, out, err = self.run_main(
                [
                    "--ecosystem",
                    "rust",
                    "--baseline",
                    str(baseline),
                    "--cargo-audit-json",
                    str(audit),
                    "--today",
                    "2026-11-12",
                ]
            )
            self.assertEqual(code, 1)
            self.assertIn("NEW", out)
            self.assertIn("RUSTSEC-2099-0002", err)

    def test_expired_advisory_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            baseline = write_json(
                root / "baseline.json",
                {
                    "rust": [
                        {
                            "id": "RUSTSEC-2099-0003",
                            "kind": "vulnerability",
                            "package": "example",
                            "expiresOn": "2026-01-01",
                            "issue": None,
                        }
                    ],
                    "javascript": [],
                },
            )
            audit = write_json(
                root / "audit.json",
                {
                    "vulnerabilities": {
                        "list": [
                            {
                                "advisory": {"id": "RUSTSEC-2099-0003"},
                                "package": {"name": "example", "version": "1.0.0"},
                            }
                        ]
                    }
                },
            )
            code, out, err = self.run_main(
                [
                    "--ecosystem",
                    "rust",
                    "--baseline",
                    str(baseline),
                    "--cargo-audit-json",
                    str(audit),
                    "--today",
                    "2026-11-12",
                ]
            )
            self.assertEqual(code, 1)
            self.assertIn("EXPIRED", out)
            self.assertIn("RUSTSEC-2099-0003", err)
            self.assertIn("grace period expired; this job fails", err)

    def test_unused_baseline_entry_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            baseline = write_json(
                root / "baseline.json",
                {
                    "rust": [
                        {
                            "id": "RUSTSEC-2099-0004",
                            "kind": "vulnerability",
                            "package": "example",
                            "expiresOn": "2099-12-31",
                            "issue": None,
                        }
                    ],
                    "javascript": [],
                },
            )
            audit = write_json(root / "audit.json", {"vulnerabilities": {"list": []}})
            code, out, err = self.run_main(
                [
                    "--ecosystem",
                    "rust",
                    "--baseline",
                    str(baseline),
                    "--cargo-audit-json",
                    str(audit),
                    "--today",
                    "2026-11-12",
                ]
            )
            self.assertEqual(code, 1)
            self.assertIn("UNUSED", out)
            self.assertIn("RUSTSEC-2099-0004", err)

    def test_javascript_high_is_gated_and_moderate_is_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            baseline = write_json(
                root / "baseline.json",
                {
                    "rust": [],
                    "javascript": [
                        {
                            "id": "GHSA-xxxx-yyyy-zzzz",
                            "kind": "high",
                            "package": "demo",
                            "expiresOn": "2099-12-31",
                            "issue": None,
                        }
                    ],
                },
            )
            audit = write_json(
                root / "audit.json",
                {
                    "advisories": {
                        "1": {
                            "github_advisory_id": "GHSA-xxxx-yyyy-zzzz",
                            "severity": "high",
                            "module_name": "demo",
                            "title": "fictional high",
                        },
                        "2": {
                            "github_advisory_id": "GHSA-aaaa-bbbb-cccc",
                            "severity": "moderate",
                            "module_name": "other",
                            "title": "fictional moderate",
                        },
                    }
                },
            )
            code, out, err = self.run_main(
                [
                    "--ecosystem",
                    "javascript",
                    "--baseline",
                    str(baseline),
                    "--pnpm-audit-json",
                    str(audit),
                    "--today",
                    "2026-11-12",
                ]
            )
            self.assertEqual(code, 0, err)
            self.assertIn("GHSA-xxxx-yyyy-zzzz", out)
            self.assertNotIn("GHSA-aaaa-bbbb-cccc", out)


    def test_production_baseline_links_tracking_issue_and_grace_date(self) -> None:
        baseline = json.loads((SCRIPTS_CI / "advisory-baseline.json").read_text(encoding="utf-8"))
        self.assertEqual(baseline.get("trackingIssue"), "https://github.com/georgiegozz/openfamiliar/issues/31")
        entries = list(baseline["rust"]) + list(baseline["javascript"])
        self.assertGreater(len(entries), 0)
        for entry in entries:
            self.assertEqual(entry.get("issue"), "https://github.com/georgiegozz/openfamiliar/issues/31", entry.get("id"))
            self.assertEqual(entry.get("expiresOn"), "2027-02-10", entry.get("id"))

    def test_placeholder_issue_is_noted_but_does_not_fail_before_expiry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            baseline = write_json(
                root / "baseline.json",
                {
                    "rust": [
                        {
                            "id": "RUSTSEC-2099-0005",
                            "kind": "vulnerability",
                            "package": "example",
                            "expiresOn": "2099-12-31",
                            "issue": "<ISSUE_URL>",
                        }
                    ],
                    "javascript": [],
                },
            )
            audit = write_json(
                root / "audit.json",
                {
                    "vulnerabilities": {
                        "list": [
                            {
                                "advisory": {"id": "RUSTSEC-2099-0005"},
                                "package": {"name": "example", "version": "1.0.0"},
                            }
                        ]
                    }
                },
            )
            code, out, err = self.run_main(
                [
                    "--ecosystem",
                    "rust",
                    "--baseline",
                    str(baseline),
                    "--cargo-audit-json",
                    str(audit),
                    "--today",
                    "2026-11-12",
                ]
            )
            self.assertEqual(code, 0, err)
            self.assertIn("replace <ISSUE_URL>", err)


if __name__ == "__main__":
    unittest.main()
