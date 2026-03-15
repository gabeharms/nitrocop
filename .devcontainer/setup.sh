#!/bin/bash
set -e

export DEBIAN_FRONTEND=noninteractive

# System dependencies
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
  build-essential \
  clang \
  curl \
  fd-find \
  git-lfs \
  jq \
  libffi-dev \
  libreadline-dev \
  libssl-dev \
  libyaml-dev \
  locales \
  python3 \
  ripgrep \
  zlib1g-dev
sudo rm -rf /var/lib/apt/lists/*

# Locale (prevents perl/bundler warnings)
sudo locale-gen en_US.UTF-8

# Install mise and add shims to PATH
curl https://mise.jdx.dev/install.sh | sh
export PATH="$HOME/.local/share/mise/shims:$HOME/.local/bin:$PATH"
eval "$(mise activate bash)"

# Toolchains via mise (+ node for codex, not in project mise.toml)
mise settings ruby.compile=false
mise trust
mise use -g node@lts
mise install
mise reshim

# Persist mise to all shells
echo 'export PATH="$HOME/.local/share/mise/shims:$HOME/.local/bin:$PATH"' >> ~/.zshenv
echo 'eval "$(~/.local/bin/mise activate zsh)"' >> ~/.zshrc
echo 'export PATH="$HOME/.local/share/mise/shims:$HOME/.local/bin:$PATH"' >> ~/.bashrc
echo 'eval "$(~/.local/bin/mise activate bash)"' >> ~/.bashrc

# Claude Code config
mkdir -p ~/.claude
cp .devcontainer/.claude/settings.json ~/.claude/settings.json 2>/dev/null \
  || cp .devcontainer/.claude/settings.example.json ~/.claude/settings.json

# Shell aliases
echo 'alias ccd="claude --dangerously-skip-permissions"' >> ~/.zshrc
echo 'alias cxd="codex --full-auto"' >> ~/.zshrc

# Codex config
mkdir -p ~/.codex
cp .devcontainer/.codex/config.toml ~/.codex/config.toml 2>/dev/null \
  || cp .devcontainer/.codex/config.example.toml ~/.codex/config.toml

# Git submodules
git submodule update --init

echo ""
echo "Setup complete. Run:"
echo "  devcontainer exec --workspace-folder . zsh"
