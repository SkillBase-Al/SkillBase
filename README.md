# SkillBase

**Multi-Agent AI Skill Management Platform** — Discover, install, manage, and assess SKILL.md files across all major AI coding assistants.

SkillBase is a cross-platform desktop application that acts as an "App Store" for AI agent skills. It helps you discover, sync, and manage `SKILL.md` files across Claude Code, Cursor, OpenCode, Windsurf, Qoder, OpenAI Codex, and any custom agent.

## Features

- **Discover** — Browse and search a skill marketplace with hundreds of open-source SKILL.md files.
- **Install & Sync** — Install skills and sync them to multiple AI agents with one click.
- **Security Scan** — Detect dangerous patterns (rm -rf, curl pipe to shell, base64 execution, etc.) before installing.
- **Format Validation** — Validate SKILL.md structure, YAML frontmatter, and semver compliance.
- **Deduplication** — Find and merge similar or duplicate skills using content-aware comparison.
- **Local Scanning** — Scan your existing skill directories and import discovered skills.
- **Batch Assessment** — Run format and security checks on all installed skills at once.
- **Custom Agents** — Configure skills directory paths for any AI agent.

## Supported AI Agents

| Agent | Path |
|-------|------|
| Claude Code | `~/.claude/skills/` |
| Cursor | `~/.cursor/rules/` |
| Windsurf | `~/.codeium/windsurf/global_workflows/` |
| Qoder | `~/.qoder/skills/` |
| OpenCode | `~/.config/opencode/skills/` |
| OpenAI Codex | `~/.codex/skills/` |

## Architecture

```
skill-manager/
├── packages/
│   ├── client/              # Tauri 2.0 desktop app (React + Rust)
│   │   ├── src-tauri/       # Rust backend (SQLite, scanner, checker, installer)
│   │   └── src/             # React frontend (shadcn/ui, Zustand, Tailwind)
│   ├── server/              # Axum REST API + GitHub crawler (PostgreSQL/pgvector)
│   └── website/             # Astro static landing page (i18n, SEO)
├── scripts/                 # dev.sh, build.sh, seed-data.sh
└── docs/                    # PRD and implementation plans
```

## Quick Start

### Desktop App

```bash
# Prerequisites: Node.js 18+, Rust 1.77+, Tauri 2.0 prerequisites

cd packages/client
npm install

# Development mode (hot-reload)
npm run tauri dev

# Production build
npm run tauri build
```

### Backend Server (optional, for marketplace)

```bash
cd packages/server

# Start PostgreSQL with pgvector
docker compose up -d

# Copy and configure environment
cp .env.example .env
# Edit .env with your GITHUB_TOKEN, LLM_API_KEY, etc.

# Run database migrations
cargo run -- migrate

# Start server
cargo run
```

### Website (landing page)

```bash
cd packages/website
npm install
npm run dev    # Development: http://localhost:4321
npm run build  # Production build → dist/
```

## Configuration

### Environment Variables (Server)

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://localhost:5432/skillbase` |
| `GITHUB_TOKEN` | GitHub PAT for crawling SKILL.md files | — |
| `LLM_PROVIDER` | LLM provider (openai/anthropic) | `openai` |
| `LLM_API_KEY` | LLM API key | — |
| `LLM_MODEL` | Model name | `gpt-4o-mini` |
| `LLM_BASE_URL` | Custom API base URL | `https://api.openai.com/v1` |

## Data Flow

```
GitHub (SKILL.md) ──crawl──→ Server DB ──REST API──→ Tauri Client ──install──→ Agent Dirs
                                │                          │
                                └── LLM Assessment ────────┘ → SQLite (local cache)
```

## License

Apache License 2.0 — see [LICENSE](LICENSE).
