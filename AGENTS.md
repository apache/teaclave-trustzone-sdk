# Agent guidance

This file is read by automated agents (security scanners, code analyzers,
AI assistants) operating on this repository. It points them at the
human-authored references they should consult before producing output.

## Development

Before making code changes, read
[ai-development.md](./docs/ai-development.md).
It defines repository layout, toolchain requirements, workspace-specific
commands, unsafe Rust and FFI rules, testing expectations, and
completion-report requirements.

## Security

Security model: [SECURITY.md](./SECURITY.md)

Agents that scan this repository should consult `SECURITY.md` for the
project's threat model, in-scope / out-of-scope declarations, and known
non-findings before reporting issues.
