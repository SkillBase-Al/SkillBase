# SkillBase

**Multi-Agent AI Skill Management Platform** — Discover, install, manage, and assess SKILL.md files across all major AI coding assistants.

SkillBase is a cross-platform desktop application that acts as an "App Store" for AI agent skills. It helps you discover, sync, and manage `SKILL.md` files across Claude Code, Cursor, OpenCode, Windsurf, Qoder, OpenAI Codex, and any custom agent.

## Features

- **Discover** — Browse and search a skill marketplace with hundreds of open-source SKILL.md files from curated GitHub repositories (e.g., `anthropics/skills`).
- **Install & Sync** — Install skills and sync them to multiple AI agents with one click.
- **Security Scan** — Detect dangerous patterns (rm -rf, curl pipe to shell, base64 execution, etc.) before installing.
- **Format Validation** — Validate SKILL.md structure, YAML frontmatter, and semver compliance.
- **Quality Assessment** — LLM-powered format scoring, safety level analysis, and security issue detection.
- **Deduplication** — Find and merge similar or duplicate skills using content-aware comparison (TF-IDF cosine similarity).
- **Local Scanning** — Scan your existing skill directories and import discovered skills.
- **Conflict Resolution** — Detect and resolve skill conflicts between agents.
- **Custom Agents** — Configure skills directory paths for any AI agent.
- **Admin Panel** — Web-based admin dashboard with usage stats, feedback management, and crawled skill inventory.
- **Telemetry** — Anonymous DAU/pageview tracking for server analytics.

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
│   │   ├── src-tauri/       # Rust backend (SQLite, scanner, checker, installer, IPC)
│   │   └── src/             # React frontend (shadcn/ui, Zustand, Tailwind)
│   ├── server/              # Axum REST API + GitHub/SkillNet crawler (PostgreSQL/pgvector)
│   ├── admin/               # React admin SPA (Vite, Recharts, Tailwind)
│   └── website/             # Astro static landing page (i18n, SEO)
├── .github/workflows/       # CI + Release (cross-platform Tauri builds)
├── scripts/                 # dev.sh, build.sh, seed-data.sh
└── docs/                    # PRD and implementation plans
```

### Data Flow

```
GitHub (SKILL.md) ──crawl──→ Server DB ──REST API──→ Tauri Client ──install──→ Agent Dirs
                                │                          │
                                ├── LLM Assessment ────────┤
                                └── Admin Panel ───────────┘ → SQLite (local cache)
```

## Quick Start

### Desktop App

```bash
# Prerequisites: Node.js 18+, Rust 1.77+, Tauri 2.0 prerequisites

cd packages/client
npm install --registry=https://registry.npmmirror.com

# Development mode (hot-reload)
npm run tauri dev

# Production build
npm run tauri build
```

### Backend Server (for marketplace & admin)

```bash
cd packages/server

# Start PostgreSQL with pgvector
docker compose up -d

# Copy and configure environment
cp .env.example .env
# Edit .env with your GITHUB_TOKEN, CRAWL_REPOS, LLM_API_KEY, etc.

# Start server (auto-runs migrations)
cargo run
```

### Admin Panel

```bash
cd packages/admin
npm install --registry=https://registry.npmmirror.com
npm run dev          # Development: http://localhost:5173
npm run build        # Production build → dist/
```

### Website (landing page)

```bash
cd packages/website
npm install --registry=https://registry.npmmirror.com
npm run dev          # Development: http://localhost:4321
npm run build        # Production build → dist/
```

## Configuration

### Environment Variables (Server)

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://localhost:5432/skillbase` |
| `GITHUB_TOKEN` | GitHub PAT for crawling SKILL.md files | — |
| `CRAWL_REPOS` | Comma-separated `owner/repo` to crawl (e.g., `anthropics/skills`) | — |
| `LLM_PROVIDER` | LLM provider (openai/anthropic/MiniMax) | `openai` |
| `LLM_API_KEY` | LLM API key | — |
| `LLM_MODEL` | Model name | `gpt-4o-mini` |
| `LLM_BASE_URL` | Custom API base URL | `https://api.openai.com/v1` |
| `ADMIN_USERNAME` | Admin panel login username | `admin` |
| `ADMIN_PASSWORD` | Admin panel login password | `****` |
| `SERVER_HOST` | Bind address | `0.0.0.0` |
| `SERVER_PORT` | Bind port | `3007` |



## License

Apache License 2.0 — see [LICENSE](LICENSE).
