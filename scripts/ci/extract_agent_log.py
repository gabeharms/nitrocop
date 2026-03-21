#!/usr/bin/env python3
"""Extract agent conversation from a Claude Code JSONL session log.

Prints assistant text and tool call summaries as markdown.

Usage: python3 extract_agent_log.py <jsonl_path> [--max-lines N]
"""
import json
import sys


def extract(path: str, max_lines: int = 500) -> None:
    lines_printed = 0
    for line in open(path):
        if lines_printed >= max_lines:
            break
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        if ev.get("type") != "assistant":
            continue
        for block in ev.get("message", {}).get("content", []):
            if lines_printed >= max_lines:
                break
            if block.get("type") == "text" and block.get("text", "").strip():
                text = block["text"].strip()
                print(text)
                print()
                lines_printed += text.count("\n") + 2
            elif block.get("type") == "tool_use":
                name = block.get("name", "?")
                inp = block.get("input", {})
                if name == "Bash":
                    cmd = inp.get("command", "")
                    print(f"> `{name}`: `{cmd[:200]}`")
                elif name in ("Read", "Glob", "Grep"):
                    arg = inp.get("file_path") or inp.get("pattern") or ""
                    print(f"> `{name}`: `{arg[:200]}`")
                elif name == "Edit":
                    fp = inp.get("file_path", "")
                    print(f"> `{name}`: `{fp}`")
                else:
                    print(f"> `{name}`")
                print()
                lines_printed += 2


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("path", help="Path to JSONL session log")
    parser.add_argument(
        "--max-lines", type=int, default=500, help="Max output lines"
    )
    args = parser.parse_args()
    extract(args.path, args.max_lines)
