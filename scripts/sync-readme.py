#!/usr/bin/env python3
"""
Regenerate README.md from README.template.md + canonical sources.

Placeholders resolved by this script:
    {{VERSION}}         - workspace.package.version from Cargo.toml
    {{MSRV}}            - workspace.package.rust-version from Cargo.toml
    {{LANGUAGES}}       - human list derived from crates/profanite-core
                          build.rs LANGS array
    {{RUST_EXAMPLE}}    - crates/profanite-core/examples/quickstart.rs
    {{NODE_EXAMPLE}}    - crates/profanite-node/examples/quickstart.js
    {{PYTHON_EXAMPLE}}  - crates/profanite-py/examples/quickstart.py

Usage:
    python3 scripts/sync-readme.py           # write README.md
    python3 scripts/sync-readme.py --check   # exit 1 if README.md is stale
"""

from __future__ import annotations

import argparse
import difflib
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TEMPLATE = REPO / "README.template.md"
README = REPO / "README.md"
CARGO_TOML = REPO / "Cargo.toml"
BUILD_RS = REPO / "crates/profanite-core/build.rs"
RUST_EXAMPLE = REPO / "crates/profanite-core/examples/quickstart.rs"
NODE_EXAMPLE = REPO / "crates/profanite-node/examples/quickstart.js"
PYTHON_EXAMPLE = REPO / "crates/profanite-py/examples/quickstart.py"
BENCH_SNAPSHOT = REPO / "bench/STATS.md"

LANG_NAMES = {
    "en": "English",
    "es": "Spanish",
    "hi": "Hindi (romanized)",
    "fr": "French",
    "de": "German",
}


def extract_toml_value(text: str, key: str) -> str:
    # Supports `key = "value"` under [workspace.package]. A full TOML parser
    # would be overkill for three lookups; this regex matches the style
    # we use in Cargo.toml.
    pattern = rf'^\s*{re.escape(key)}\s*=\s*"([^"]+)"\s*$'
    for line in text.splitlines():
        m = re.match(pattern, line)
        if m:
            return m.group(1)
    raise ValueError(f"could not find `{key}` in Cargo.toml")


def extract_languages(build_rs: str) -> list[str]:
    # `const LANGS: &[&str] = &["en", "es", "hi", "fr", "de"];`
    m = re.search(r'LANGS\s*:\s*&\[&str\]\s*=\s*&\[(.*?)\];', build_rs, re.DOTALL)
    if not m:
        raise ValueError("could not find LANGS array in build.rs")
    return re.findall(r'"([^"]+)"', m.group(1))


def render_language_list(codes: list[str]) -> str:
    parts = []
    for code in codes:
        name = LANG_NAMES.get(code, code)
        parts.append(f"{name} (`{code}`)")
    return ", ".join(parts)


def load_example(path: Path) -> str:
    text = path.read_text().rstrip() + "\n"
    # Trim trailing newline inside the code block so rendering stays tight.
    return text.rstrip()


def render(template: str) -> str:
    version = extract_toml_value(CARGO_TOML.read_text(), "version")
    msrv = extract_toml_value(CARGO_TOML.read_text(), "rust-version")
    langs = render_language_list(extract_languages(BUILD_RS.read_text()))
    rust_example = load_example(RUST_EXAMPLE)
    node_example = load_example(NODE_EXAMPLE)
    python_example = load_example(PYTHON_EXAMPLE)
    if not BENCH_SNAPSHOT.exists():
        raise ValueError(
            f"{BENCH_SNAPSHOT.relative_to(REPO)} missing. "
            "Run `cargo run -p profanite-bench -- snapshot`."
        )
    bench_snapshot = BENCH_SNAPSHOT.read_text().strip()

    replacements = {
        "{{VERSION}}": version,
        "{{MSRV}}": msrv,
        "{{LANGUAGES}}": langs,
        "{{RUST_EXAMPLE}}": rust_example,
        "{{NODE_EXAMPLE}}": node_example,
        "{{PYTHON_EXAMPLE}}": python_example,
        "{{BENCH_SNAPSHOT}}": bench_snapshot,
    }
    out = template
    for key, value in replacements.items():
        out = out.replace(key, value)

    # Sanity: no unreplaced placeholders left.
    stragglers = re.findall(r"\{\{[A-Z_]+\}\}", out)
    if stragglers:
        raise ValueError(f"unresolved placeholders: {stragglers}")
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="exit 1 if README.md is stale; do not write",
    )
    args = ap.parse_args()

    template = TEMPLATE.read_text()
    expected = render(template)

    if args.check:
        if not README.exists():
            print("README.md missing", file=sys.stderr)
            return 1
        current = README.read_text()
        if current != expected:
            print(
                "README.md is stale. Run `python3 scripts/sync-readme.py` to regenerate.\n",
                file=sys.stderr,
            )
            diff = difflib.unified_diff(
                current.splitlines(keepends=True),
                expected.splitlines(keepends=True),
                fromfile="README.md (current)",
                tofile="README.md (expected)",
            )
            sys.stderr.writelines(diff)
            return 1
        print("README.md is in sync")
        return 0

    README.write_text(expected)
    print(f"wrote {README.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
