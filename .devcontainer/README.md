# Devcontainer

Development container for running Claude Code and Codex against nitrocop.

## Prerequisites

- Docker Desktop for Mac
- 1Password SSH agent enabled (1Password > Settings > Developer > Enable SSH Agent)
- `~/.gitconfig` on the host with SSH commit signing configured (`gpg.format = ssh`, `commit.gpgsign = true`, `user.signingkey`)

### macOS + 1Password SSH agent setup

Configure `SSH_AUTH_SOCK` globally so all apps (including Docker Desktop) use the 1Password SSH agent. Follow [1Password's guide](https://developer.1password.com/docs/ssh/agent/compatibility/#configure-ssh_auth_sock-globally-for-every-client) to create a LaunchAgent, then log out and back in. 1Password must be unlocked before starting Docker Desktop — the SSH agent only serves keys while unlocked.

Add to `~/.zshrc`:

```bash
export DEVCONTAINER_GITHUB_TOKEN=$(gh auth token)
export DEVCONTAINER_CLAUDE_CODE_OAUTH_TOKEN=<your-token>  # from `claude setup-token`
```

## What's included

- **mise** manages Rust, Ruby, and Node toolchains
- **Claude Code** and **Codex** for AI-assisted development
- **gh** CLI for GitHub operations (auto-authenticated via `DEVCONTAINER_GITHUB_TOKEN`)
- build-essential, clang, python3, ripgrep, fd, jq, git-lfs
- Ruby build deps (libssl, libyaml, libffi, libreadline, zlib)

## Files

- `devcontainer.json` — container config, mounts, and env forwarding
- `setup.sh` — runs once after container creation (installs deps, toolchains, AI tools)
- `.claude/settings.example.json` — default Claude Code settings (copied to `~/.claude/settings.json` in container)
- `.claude/settings.json` — personal override (gitignored, takes priority over example)

## Usage

```bash
brew install devcontainer                       # one-time
devcontainer up --workspace-folder .
devcontainer exec --workspace-folder . zsh
```

Verify inside the container:

```bash
cargo build
claude --version
gh auth status
ssh-add -L          # should list 1Password SSH keys
```

## Commit signing

Commit signing works via 1Password SSH agent forwarding through Docker Desktop. The host's `~/.gitconfig` is mounted read-only, and `setup.sh` overrides `gpg.ssh.program` from `op-ssh-sign` (macOS-only) to `ssh-keygen` (which signs via the forwarded SSH agent).

Note: `git log --show-signature` will say "No signature" locally because SSH signature verification requires an `allowedSignersFile`. The signature IS present — use `git cat-file -p HEAD` to confirm. GitHub verifies signatures independently and shows the "Verified" badge.

### Troubleshooting

**`ssh-add -L` returns "The agent has no identities"** — Docker Desktop isn't forwarding the 1Password agent. Verify the 1Password LaunchAgent is installed (`ls ~/Library/LaunchAgents/com.1password.SSH_AUTH_SOCK.plist`) and that you've logged out and back in since installing it. Then fully quit Docker Desktop (menu bar icon > "Quit Docker Desktop") and reopen it.

**`ssh-add -L` returns "Permission denied"** — run `sudo chmod 666 /ssh-agent` (the `postStartCommand` should do this automatically).

**`ssh-add -L` returns "Error connecting to agent"** — the socket mount failed. Ensure Docker Desktop is running.
