#!/usr/bin/env python3
"""Tests for resolve_agent_secrets.py."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[3] / "scripts" / "workflows"))
import resolve_agent_secrets


def test_writes_secret_files(tmp_path, monkeypatch):
    """Writes target and source-name files when they differ."""
    backend_env = tmp_path / "backend.env"
    backend_env.write_text(
        "cli=claude\n"
        "secret_MINIMAX_API_KEY=ANTHROPIC_AUTH_TOKEN\n"
    )
    secrets_dir = tmp_path / "secrets"
    monkeypatch.setenv("_SECRET_MINIMAX_API_KEY", "my-minimax-key")

    resolve_agent_secrets.resolve(str(backend_env), str(secrets_dir))

    assert (secrets_dir / "ANTHROPIC_AUTH_TOKEN").read_text() == "my-minimax-key"
    assert (secrets_dir / "MINIMAX_API_KEY").read_text() == "my-minimax-key"


def test_skips_when_source_and_target_match(tmp_path, monkeypatch):
    """Only writes one file when source == target."""
    backend_env = tmp_path / "backend.env"
    backend_env.write_text("secret_ANTHROPIC_API_KEY=ANTHROPIC_API_KEY\n")
    secrets_dir = tmp_path / "secrets"
    monkeypatch.setenv("_SECRET_ANTHROPIC_API_KEY", "my-key")

    resolve_agent_secrets.resolve(str(backend_env), str(secrets_dir))

    assert (secrets_dir / "ANTHROPIC_API_KEY").read_text() == "my-key"
    assert len(list(secrets_dir.iterdir())) == 1


def test_preserves_multiline_json(tmp_path, monkeypatch):
    """Multi-line JSON secrets are preserved byte-for-byte."""
    backend_env = tmp_path / "backend.env"
    backend_env.write_text("secret_CODEX_AUTH_JSON=CODEX_AUTH_JSON\n")
    secrets_dir = tmp_path / "secrets"
    json_value = '{\n  "access_token": "gho_xxx",\n  "token_type": "bearer"\n}'
    monkeypatch.setenv("_SECRET_CODEX_AUTH_JSON", json_value)

    resolve_agent_secrets.resolve(str(backend_env), str(secrets_dir))

    assert (secrets_dir / "CODEX_AUTH_JSON").read_text() == json_value


def test_skips_empty_secret(tmp_path, monkeypatch):
    """Empty secrets produce no files."""
    backend_env = tmp_path / "backend.env"
    backend_env.write_text("secret_MINIMAX_API_KEY=ANTHROPIC_AUTH_TOKEN\n")
    secrets_dir = tmp_path / "secrets"
    monkeypatch.setenv("_SECRET_MINIMAX_API_KEY", "")

    resolve_agent_secrets.resolve(str(backend_env), str(secrets_dir))

    assert len(list(secrets_dir.iterdir())) == 0


def test_ignores_non_secret_lines(tmp_path, monkeypatch):
    """Non-secret_ lines are ignored."""
    backend_env = tmp_path / "backend.env"
    backend_env.write_text(
        "cli=codex\n"
        "env_ANTHROPIC_MODEL=claude-opus-4-6\n"
        "secret_CODEX_AUTH_JSON=CODEX_AUTH_JSON\n"
        "setup_cmd=echo hello\n"
    )
    secrets_dir = tmp_path / "secrets"
    monkeypatch.setenv("_SECRET_CODEX_AUTH_JSON", '{"token":"x"}')

    resolve_agent_secrets.resolve(str(backend_env), str(secrets_dir))

    assert len(list(secrets_dir.iterdir())) == 1
    assert (secrets_dir / "CODEX_AUTH_JSON").read_text() == '{"token":"x"}'
