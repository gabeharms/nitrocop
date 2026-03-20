#!/usr/bin/env python3
"""Generate stress-test RuboCop configs for corpus validation.

Produces two configs that surface bugs the standard corpus run misses:

1. flipped_styles.yml  — Every EnforcedStyle set to a non-default value.
   Catches "cop only works with the default style" bugs.

2. stripped_config.yml — All cops enabled with vendor defaults, no per-project
   config inheritance. Catches bugs masked by project-level config overrides.

Usage:
    python3 scripts/gen-stress-configs.py                    # generate both
    python3 scripts/gen-stress-configs.py --output-dir dir/  # custom output dir
    python3 scripts/gen-stress-configs.py --dry-run          # preview without writing
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

VENDOR_CONFIGS = [
    ("vendor/rubocop/config/default.yml", None),
    ("vendor/rubocop-rails/config/default.yml", "rubocop-rails"),
    ("vendor/rubocop-performance/config/default.yml", "rubocop-performance"),
    ("vendor/rubocop-rspec/config/default.yml", "rubocop-rspec"),
    ("vendor/rubocop-rspec_rails/config/default.yml", "rubocop-rspec_rails"),
    ("vendor/rubocop-factory_bot/config/default.yml", "rubocop-factory_bot"),
]

# The baseline config that the corpus oracle already uses
BASELINE_PATH = "bench/corpus/baseline_rubocop.yml"


def parse_enforced_styles(config_path: str) -> list[dict]:
    """Parse a vendor default.yml to find all Enforced* keys and their alternatives.

    Returns list of dicts: {cop, key, default, alternatives}
    Uses simple line-based parsing to avoid PyYAML dependency.
    """
    results = []
    path = Path(config_path)
    if not path.exists():
        return results

    current_cop = None
    enforced_key = None
    enforced_val = None
    supported_key = None
    supported_vals: list[str] = []
    in_supported = False

    for line in path.read_text().splitlines():
        # Top-level cop key (e.g., "Style/StringLiterals:")
        m = re.match(r"^([A-Z]\w+/\w+):", line)
        if m:
            # Flush previous
            if current_cop and enforced_key and enforced_val and supported_vals:
                alts = [v for v in supported_vals if v != enforced_val]
                if alts:
                    results.append({
                        "cop": current_cop,
                        "key": enforced_key,
                        "default": enforced_val,
                        "alternatives": alts,
                    })
            current_cop = m.group(1)
            enforced_key = None
            enforced_val = None
            supported_key = None
            supported_vals = []
            in_supported = False
            continue

        if not current_cop:
            continue

        # Another top-level key (AllCops:, etc.) — flush
        if re.match(r"^\S", line) and not line.startswith("#"):
            if enforced_key and enforced_val and supported_vals:
                alts = [v for v in supported_vals if v != enforced_val]
                if alts:
                    results.append({
                        "cop": current_cop,
                        "key": enforced_key,
                        "default": enforced_val,
                        "alternatives": alts,
                    })
            current_cop = None
            enforced_key = None
            enforced_val = None
            in_supported = False
            continue

        stripped = line.strip()

        # Enforced* key
        m = re.match(r"^  (Enforced\w+):\s*(.+)", line)
        if m:
            # Flush previous enforced key if any
            if enforced_key and enforced_val and supported_vals:
                alts = [v for v in supported_vals if v != enforced_val]
                if alts:
                    results.append({
                        "cop": current_cop,
                        "key": enforced_key,
                        "default": enforced_val,
                        "alternatives": alts,
                    })
            enforced_key = m.group(1)
            enforced_val = m.group(2).strip().strip("'\"")
            supported_vals = []
            in_supported = False
            continue

        # Supported* key (array start)
        m = re.match(r"^  (Supported\w+):", line)
        if m:
            in_supported = True
            supported_vals = []
            continue

        # Array item under Supported*
        if in_supported and stripped.startswith("- "):
            val = stripped[2:].strip().strip("'\"")
            supported_vals.append(val)
            continue

        # End of array
        if in_supported and not stripped.startswith("-") and not stripped.startswith("#") and stripped:
            in_supported = False

    # Flush last cop
    if current_cop and enforced_key and enforced_val and supported_vals:
        alts = [v for v in supported_vals if v != enforced_val]
        if alts:
            results.append({
                "cop": current_cop,
                "key": enforced_key,
                "default": enforced_val,
                "alternatives": alts,
            })

    return results


def generate_flipped_config(output_path: Path, dry_run: bool = False) -> int:
    """Generate a config with all EnforcedStyles flipped to non-default values."""
    all_styles = []
    for config_path, _plugin in VENDOR_CONFIGS:
        all_styles.extend(parse_enforced_styles(config_path))

    # Group by cop (some cops have multiple Enforced* keys)
    by_cop: dict[str, list[dict]] = {}
    for s in all_styles:
        by_cop.setdefault(s["cop"], []).append(s)

    lines = [
        "# Auto-generated stress-test config: all EnforcedStyles flipped to non-default.",
        "# This surfaces bugs where a cop only works with the default style.",
        f"# Generated by: python3 scripts/gen-stress-configs.py",
        f"# Covers {len(all_styles)} EnforcedStyle keys across {len(by_cop)} cops.",
        "#",
        "# Usage: layer this on top of baseline_rubocop.yml in corpus runs.",
        "#   rubocop --config bench/corpus/flipped_styles.yml ...",
        "",
        "inherit_from: baseline_rubocop.yml",
        "",
    ]

    for cop in sorted(by_cop.keys()):
        styles = by_cop[cop]
        lines.append(f"{cop}:")
        for s in styles:
            # Pick the first alternative (most different from default)
            flipped = s["alternatives"][0]
            lines.append(f"  {s['key']}: {flipped}  # default: {s['default']}")
        lines.append("")

    content = "\n".join(lines) + "\n"

    if dry_run:
        print(content)
        print(f"# Would write {len(by_cop)} cops, {len(all_styles)} style keys", file=sys.stderr)
    else:
        output_path.write_text(content)
        print(f"Wrote {output_path} ({len(by_cop)} cops, {len(all_styles)} style keys)", file=sys.stderr)

    return len(all_styles)


def generate_stripped_config(output_path: Path, dry_run: bool = False) -> int:
    """Generate a config with all cops enabled, no project config inheritance.

    This is the baseline config but with explicit instructions to ignore
    per-project .rubocop.yml files. Tests pure cop detection logic.
    """
    # Collect all cops from all vendor configs
    all_cops: dict[str, str] = {}  # cop -> plugin source
    for config_path, plugin in VENDOR_CONFIGS:
        path = Path(config_path)
        if not path.exists():
            continue
        for line in path.read_text().splitlines():
            m = re.match(r"^([A-Z]\w+/\w+):", line)
            if m:
                all_cops[m.group(1)] = plugin or "rubocop"

    lines = [
        "# Auto-generated stress-test config: all cops enabled, no project config.",
        "# Ignores per-project .rubocop.yml — tests pure cop detection logic.",
        f"# Generated by: python3 scripts/gen-stress-configs.py",
        f"# Covers {len(all_cops)} cops.",
        "#",
        "# Usage: run with --config to override project configs:",
        "#   rubocop --config bench/corpus/stripped_config.yml ...",
        "#   nitrocop --config bench/corpus/stripped_config.yml ...",
        "",
        "plugins:",
        "  - rubocop-rails",
        "  - rubocop-performance",
        "  - rubocop-rspec",
        "  - rubocop-rspec_rails",
        "  - rubocop-factory_bot",
        "",
        "AllCops:",
        "  NewCops: enable",
        "  SuggestExtensions: false",
        "  TargetRubyVersion: 4.0",
        "  TargetRailsVersion: 7.0",
        "  Exclude:",
        '    - "vendor/**/*"',
        '    - "node_modules/**/*"',
        '    - "tmp/**/*"',
        '    - "coverage/**/*"',
        '    - "dist/**/*"',
        '    - "build/**/*"',
        '    - ".git/**/*"',
        '    - ".bundle/**/*"',
        '    - ".bundles_cache/**/*"',
        '    - "db/schema.rb"',
        '    - "bin/**/*"',
        "",
    ]

    # Enable every cop explicitly
    prev_dept = None
    for cop in sorted(all_cops.keys()):
        dept = cop.split("/")[0]
        if dept != prev_dept:
            if prev_dept:
                lines.append("")
            lines.append(f"# {dept} ({all_cops[cop]})")
            prev_dept = dept
        lines.append(f"{cop}:")
        lines.append("  Enabled: true")

    lines.append("")

    content = "\n".join(lines) + "\n"

    if dry_run:
        print(content[:3000])
        print(f"... (truncated)")
        print(f"# Would write {len(all_cops)} cops", file=sys.stderr)
    else:
        output_path.write_text(content)
        print(f"Wrote {output_path} ({len(all_cops)} cops)", file=sys.stderr)

    return len(all_cops)


def main():
    parser = argparse.ArgumentParser(description="Generate stress-test RuboCop configs")
    parser.add_argument("--output-dir", type=str, default="bench/corpus",
                        help="Output directory (default: bench/corpus)")
    parser.add_argument("--dry-run", action="store_true", help="Preview without writing")
    args = parser.parse_args()

    out = Path(args.output_dir)

    print("Parsing vendor configs...", file=sys.stderr)

    n_styles = generate_flipped_config(out / "flipped_styles.yml", args.dry_run)
    n_cops = generate_stripped_config(out / "stripped_config.yml", args.dry_run)

    print(f"\nSummary:", file=sys.stderr)
    print(f"  flipped_styles.yml: {n_styles} EnforcedStyle keys flipped", file=sys.stderr)
    print(f"  stripped_config.yml: {n_cops} cops force-enabled", file=sys.stderr)


if __name__ == "__main__":
    main()
