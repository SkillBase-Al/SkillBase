# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SkillBase is a desktop GUI application (Tauri 2.0) for discovering, installing, managing, and assessing AI agent skills (SKILL.md format). Think "App Store for AI skills."

## Architecture

```
skill-manager/
├── packages/
│   ├── client/              # Tauri 2.0 desktop app
│   │   ├── src-tauri/       # Rust backend (SQLite, scanner, checker, installer, IPC)
│   │   │   ├── src/
│   │   │   │   ├── commands/      # #[tauri::command] IPC handlers
│   │   │   │   ├── db/            # SQLite layer (rusqlite)
│   │   │   │   ├── scanner/       # Local SKILL.md file scanner
│   │   │   │   ├── checker/       # Format + security + dependency checkers
│   │   │   │   ├── dedup/         # Local dedup similarity engine
│   │   │   │   ├── installer/     # Agent installation/sync
│   │   │   │   ├── api_client/    # Server REST API client
│   │   │   │   └── utils/         # Path utilities
│   │   ├── src/                   # React frontend
│   │   │   ├── components/        # UI components (shadcn/ui based)
│   │   │   ├── pages/             # DiscoverPage, InstalledPage, DedupPage, SettingsPage
│   │   │   ├── stores/            # Zustand state stores
│   │   │   ├── services/          # Tauri IPC + HTTP service wrappers
│   │   │   ├── hooks/             # Data-fetching hooks
│   │   │   └── types/             # TypeScript type definitions
│   └── server/                    # Rust backend API + crawler
│       ├── src/
│       │   ├── api/               # axum REST API routes
│       │   ├── crawler/           # GitHub/SkillNet adapters + scheduler
│       │   ├── pipeline/          # Data cleaning pipeline
│       │   ├── db/                # PostgreSQL (sqlx) schema + queries
│       │   ├── embedding/         # TF-IDF/vector similarity
│       │   ├── assessment/        # LLM quality assessment
│       │   └── llm/               # LLM provider abstraction
│       ├── migrations/            # SQL migration files
│       ├── Cargo.toml
│       └── docker-compose.yml
├── docs/                         # PRD and implementation plans
└── scripts/                      # dev.sh, build.sh
```

### Data Flow

- **Client <-> Server**: React frontend talks to Rust backend via Tauri IPC (`invoke()`). Rust backend talks to the server via HTTP REST API (`reqwest`).
- **Local storage**: Skills stored in `~/.skillbase/skills/`, indexed in `~/.skillbase/index.db` (SQLite).
- **Agent installation**: Skills are copied from `~/.skillbase/skills/` to each agent's skill directory (e.g., `~/.claude/skills/`).

### Key Schemas

**Client SQLite (5 tables):** `installed_skills`, `agent_configs`, `install_mappings`, `app_settings`, `assessment_results`

**Server PostgreSQL:** `skills` (with `vector(384)` embedding), `categories`, `skill_categories`

## Build & Dev Commands

```sh
# Install dependencies
cd packages/client && npm install

# Dev mode (full stack)
npm run dev              # concurrent: cargo run (server) + tauri dev (client)

# Dev mode (individual)
cd packages/client && npm run tauri dev    # Tauri desktop app
cd packages/server && cargo run            # Backend API server (port 3000)

# Build
npm run build                              # tauri build + cargo build --release

# Rust (client backend)
cd packages/client/src-tauri && cargo build
cd packages/client/src-tauri && cargo test

# Rust (server)
cd packages/server && cargo build
cd packages/server && cargo test

# Frontend only
cd packages/client && npm run dev          # Vite dev server on :1420
cd packages/client && npm run build        # TypeScript check + vite build

# Docker
docker compose -f packages/server/docker-compose.yml up -d
```

## Implementation Status (v1.0)

The project is in early scaffolding phase (M0). The plan has 6 milestones:

- **M0** (current): Project scaffolding — monorepo, Tauri client, server skeleton, scripts ✓
- **M1**: Rust backend core — SQLite, scanner, checkers, dedup, installer, API client, IPC commands
- **M2**: React frontend — tailwind/shadcn, pages (discover, installed, dedup, settings), stores, components
- **M3**: Server MVP — PostgreSQL, REST API (axum), crawlers (GitHub, SkillNet), pipeline, embedding
- **M4**: Integration — install/uninstall flow, enable/disable, update/sync, quality assessment integration
- **M5**: Polish — error handling, testing, CI/CD, cross-platform build, docs

## Key Conventions

- **Rust**: Modules follow the pattern `mod.rs` barrel exports, `models.rs` for data types, `repository.rs` for DB CRUD
- **React**: Zustand stores use the pattern `{ items, isLoading, error, isEmpty, fetchItems() }`
- **IPC commands**: All Tauri commands are in `commands/` module, named as `commands::{module}::{fn}`
- **Server API**: axum handlers return JSON, pagination with `page`/`per_page` params (default 20, max 100)
- **State handling**: Every UI component covers loading (skeleton), empty, error, and success states

## 安装依赖以及下载依赖包
尽可能使用国内镜像去下载
