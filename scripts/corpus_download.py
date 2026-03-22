#!/usr/bin/env python3
"""Compatibility CLI wrapper for the shared corpus download module."""

from shared.corpus_download import *  # noqa: F401,F403


if __name__ == "__main__":
    raise SystemExit(main())
