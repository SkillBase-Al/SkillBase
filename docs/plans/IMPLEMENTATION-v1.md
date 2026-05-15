# SkillBase — 实现方案 v1.0

> 基于 [PRD-v1.md](../PRD-v1.md) 的详细实现规划

---

## 项目结构

```
skill-manager/
├── packages/
│   ├── client/                       # Tauri 2.0 桌面客户端
│   │   ├── src-tauri/                # Rust 后端
│   │   │   ├── src/
│   │   │   │   ├── main.rs
│   │   │   │   ├── lib.rs            # Tauri 入口，插件/命令注册
│   │   │   │   ├── commands/         # IPC 命令处理器
│   │   │   │   ├── db/               # SQLite 层
│   │   │   │   ├── scanner/          # 本地 Skill 扫描
│   │   │   │   ├── checker/          # 格式 + 安全检查
│   │   │   │   ├── dedup/            # 本地去重相似度计算
│   │   │   │   ├── installer/        # Agent 安装/同步
│   │   │   │   ├── api_client/       # 服务端 REST API 客户端
│   │   │   │   └── utils/
│   │   │   ├── Cargo.toml
│   │   │   ├── capabilities/default.json
│   │   │   └── tauri.conf.json
│   │   ├── src/                      # React 前端
│   │   │   ├── main.tsx
│   │   │   ├── App.tsx
│   │   │   ├── index.css
│   │   │   ├── components/
│   │   │   │   ├── ui/              # shadcn/ui
│   │   │   │   ├── layout/          # AppLayout, Sidebar, TopBar, StatusBar
│   │   │   │   ├── skill/           # SkillCard, SkillDetail, SkillGrid, SkillSearch
│   │   │   │   ├── assessment/      # AssessmentBadge, AssessmentDashboard, IssueList
│   │   │   │   ├── dedup/           # DedupGroupList, DedupGroupCard, ComparisonModal
│   │   │   │   ├── agent/           # AgentSelector, AgentConfigForm
│   │   │   │   └── shared/          # LoadingSkeleton, EmptyState, ErrorState
│   │   │   ├── pages/
│   │   │   │   ├── DiscoverPage.tsx
│   │   │   │   ├── InstalledPage.tsx
│   │   │   │   ├── DedupPage.tsx
│   │   │   │   └── SettingsPage.tsx
│   │   │   ├── stores/               # Zustand stores
│   │   │   ├── services/             # API 调用封装
│   │   │   ├── hooks/                # 数据获取 hooks
│   │   │   └── types/                # TypeScript 类型定义
│   │   ├── package.json
│   │   ├── vite.config.ts
│   │   └── index.html
│   └── server/                       # Rust 后端服务
│       ├── src/
│       │   ├── main.rs
│       │   ├── api/                  # axum REST API
│       │   ├── crawler/              # 适配器式爬虫
│       │   │   ├── scheduler.rs
│       │   │   ├── github_adapter.rs
│       │   │   └── skillnet_adapter.rs
│       │   ├── pipeline/             # 数据清洗管道
│       │   ├── db/                   # PostgreSQL + sqlx
│       │   └── embedding/            # 向量嵌入服务
│       ├── Cargo.toml
│       ├── Dockerfile
│       └── docker-compose.yml
├── docs/
│   ├── PRD-v1.md
│   └── plans/IMPLEMENTATION-v1.md
├── scripts/
│   ├── dev.sh
│   └── build.sh
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── package.json                       # npm workspace root
└── README.md
```

---

## 里程碑与依赖关系

```
M0 项目脚手架 (3-5 天)
 ├── 0.1 环境准备
 ├── 0.2 Monorepo 初始化
 ├── 0.3 Tauri 客户端脚手架
 ├── 0.4 服务端项目初始化
 └── 0.5 辅助脚本
      │
      ├──────────────────┬──────────────────┐
      ▼                  ▼                  ▼
   M1 Rust 后端核心    M2 React 前端核心   M3 服务端 MVP
   (3-4 周)           (3-4 周)           (3-4 周)
      │                  │                  │
      └────────┬─────────┘                  │
               ▼                            │
        M4 功能集成联调                      │
        (2-3 周)                            │
               │                            │
               ▼                            │
        M5 打磨 & 发布 ◄────────────────────┘
        (2-3 周)
```

M1、M2、M3 在 M0 完成后可并行执行。

---

## Milestone 0: 项目脚手架与基础设施

**依赖：** 无 | **估算：** 3-5 天

### 0.1 环境准备
- 安装 Tauri CLI v2：`cargo install tauri-cli --version "^2"`
- 确认 Rust 1.80+、Node 22+、npm 10+
- **复杂度：** S

### 0.2 Monorepo 根目录初始化
- 根 `package.json`（workspaces: `["packages/*"]`）
- `.gitignore`、`tsconfig.base.json`、`.prettierrc`
- Git 初始化
- **关键文件：** `package.json`、`.gitignore` | **复杂度：** S

### 0.3 Tauri 2.0 + React 客户端脚手架
- `npm create tauri-app` 模板 React+TypeScript
- 安装依赖：`@tauri-apps/api@^2`、zustand、Tailwind CSS v4、shadcn/ui
- 添加 shadcn 组件（button、card、dialog、tabs、badge、sidebar、switch、skeleton、toast 等）
- 配置 Tauri 权限（capabilities/default.json）
- **关键文件：** `packages/client/` 全部脚手架文件 | **复杂度：** M

### 0.4 服务端项目初始化
- `cargo init packages/server`
- Cargo.toml 添加依赖：axum、tokio、serde、sqlx（postgres）、reqwest、tower-http、tracing、uuid、dotenvy
- axum 骨架 + health 端点 + CORS 中间件
- `docker-compose.yml`（pgvector + API 服务）
- **关键文件：** `packages/server/Cargo.toml`、`docker-compose.yml` | **复杂度：** S

### 0.5 辅助脚本
- `scripts/dev.sh`：启动 docker-compose + 服务端 + 客户端开发模式
- `scripts/build.sh`：构建服务端 Docker 镜像 + Tauri 桌面包
- **复杂度：** S

---

## Milestone 1: Tauri Rust 后端核心

**依赖：** M0 | **可并行：** M2、M3 | **估算：** 3-4 周

### 1.1 应用目录与路径工具
- 定义 `~/.skillbase/` 标准目录结构（skills/、skillsets/、index.db、agents.json）
- 实现 `utils/paths.rs` 路径解析函数
- 在 `lib.rs` setup hook 中自动创建目录
- **关键文件：** `src-tauri/src/utils/paths.rs` | **复杂度：** S

### 1.2 SQLite 数据库层
- 添加 `rusqlite = { features = ["bundled"] }`
- Schema（5 表）：`installed_skills`、`agent_configs`、`install_mappings`、`app_settings`、`assessment_results`
- 实现 `db/migrations.rs`、`models.rs`、`repository.rs`（CRUD 方法）
- **关键文件：** `src-tauri/src/db/` | **复杂度：** M

### 1.3 SKILL.md 解析器
- 解析 YAML frontmatter（`---` 分隔符）
- 定义 `SkillMetadata` 结构体（覆盖 agentskills.io 规范字段）
- 错误处理：缺少 frontmatter、YAML 格式异常、必填字段缺失
- **关键文件：** `src-tauri/src/checker/format_checker.rs` | **复杂度：** S

### 1.4 格式合规检查器
- 校验字段：name（小写、连字符）、description（>= 50 字符）、version（semver）
- 评分 0-100，按字段重要性加权
- 返回 `Vec<Issue>`：severity + field + message
- **关键文件：** `src-tauri/src/checker/format_checker.rs` | **复杂度：** M

### 1.5 安全检查器
- 扫描 `scripts/` 下文件：rm -rf /、curl | sh、base64 编码命令、eval、exec、已知恶意 URL
- 分级：Safe / Warning / Dangerous
- **关键文件：** `src-tauri/src/checker/security_checker.rs` | **复杂度：** M

### 1.5b 依赖完整性检查器
- 文本层面扫描 SKILL.md + scripts/ 文件，提取依赖声明关键字：
  - `pip install`、`npm install`、`brew install`、`apt-get`、`cargo install`、`go install` 等
  - `requires:`、`dependencies:`、`depends on:` 等元数据标记
- 检查声明是否包含依赖名称（不验证可安装性）
- 评分：有依赖声明且包含名称 = 通过，有依赖声明但缺少名称/版本 = 警告，无任何依赖声明 = 不适用
- **关键文件：** `src-tauri/src/checker/dep_checker.rs` | **复杂度：** S

### 1.6 本地 Scanner
- 遍历 `~/.skillbase/skills/`（使用 ignore crate）
- 检测新增、修改（mtime/content-hash）、删除
- 支持扫描**任意目录**：接收外部路径参数，扫描其中的 SKILL.md 文件（用于首次启动向导的"导入已有 Skill"功能）
- **关键文件：** `src-tauri/src/scanner/skill_scanner.rs` | **复杂度：** M

### 1.7 本地去重引擎
- 名称归一化 → Levenshtein 距离 + Jaccard 相似度 → 综合评分
- 阈值：> 0.8 高度相似，> 0.6 可能相似
- 描述相似度：批量调服务端 API 计算向量余弦值
- **关键文件：** `src-tauri/src/dedup/local_dedup.rs` | **复杂度：** M

### 1.8 Agent 安装器
- 支持的 Agent 类型：ClaudeCode、Cursor、Windsurf、CodexCLI、GitHub Copilot
- 操作：install（复制到各 Agent 目录）、uninstall、update、sync
- 维护 `install_mappings` 表（Skill ↔ Agent M:N 关系）
- **关键文件：** `src-tauri/src/installer/agent_installer.rs` | **复杂度：** M

### 1.9 市场 API 客户端
- reqwest 客户端调用服务端 REST API
- 函数：list_skills、search_skills、get_skill_detail、compute_similarity
- 可配置 base URL、超时（5s）、重试（1 次）、响应缓存（10 分钟 TTL）
- **关键文件：** `src-tauri/src/api_client/market_client.rs` | **复杂度：** S

### 1.10 Tauri IPC 命令
- 将所有模块封装为 `#[tauri::command]`
- 命令清单：get_installed_skills、install_skill、uninstall_skill、update_skill、toggle_skill_enabled、scan_local_skills、assess_format、assess_security、batch_assess、run_dedup、get_agents、add_agent、update_agent、delete_agent、search_market、get_skill_detail 等
- 通过 `generate_handler![]` 注册
- **关键文件：** `src-tauri/src/commands/` | **复杂度：** L

---

## Milestone 2: React 前端核心

**依赖：** M0 | **可并行：** M1、M3 | **估算：** 3-4 周

### 2.1 Tailwind + shadcn/ui + 主题
- Tailwind CSS v4 配置、shadcn 组件添加、CSS 变量主题、暗色模式
- **复杂度：** S

### 2.2 TypeScript 类型定义
- types/skill.ts、agent.ts、assessment.ts、settings.ts
- IPC 负载类型（与 Rust 命令参数对应）
- **复杂度：** S

### 2.3 布局壳 + 首次启动引导
- AppLayout（CSS grid）、Sidebar（导航）、TopBar、StatusBar
- react-router-dom 路由：/discover、/installed、/dedup、/settings
- 导航项（v1.0）：发现、已安装、去重、设置（收藏和集合入口 v1.0 隐藏，v1.5 开放）
  - 检测首次启动（无 Agent 配置 + 无已安装 Skill 时自动弹出）
  - Step 1：配置 AI Agent 路径（选择本地已有 Agent 的安装目录或项目目录，支持多选）
  - Step 2：扫描本地已有 SKILL.md（让用户选择要扫描的文件夹，扫描其中的 SKILL.md 文件）
  - Step 3：导入扫描到的 Skill 到 SkillBase（展示扫描结果，用户勾选要导入的，确认后加入 ~/.skillbase/skills/）
- **判断重入：** 后续启动时若已有配置和 Skill 则不弹出
- **复杂度：** M

### 2.4 Zustand Stores
- uiStore、settingsStore、agentStore、marketStore、installedStore、assessmentStore、dedupStore
- 统一模式：`{ items, isLoading, error, isEmpty, fetchItems() }`
- **复杂度：** M

### 2.5 Tauri IPC 服务层
- `services/tauri.ts`：invoke() 的类型安全封装
- `services/api.ts`：直接 HTTP 调用服务端 API
- **复杂度：** S

### 2.6 共享组件
- LoadingSkeleton（卡片/列表/详情变体）
- EmptyState（图标+标题+描述+CTA）
- ErrorState（错误信息+重试按钮）
- ConfirmDialog、Toast
- **复杂度：** S

### 2.7 设置页面
- Agent 标签页：列表 + 添加/编辑/删除表单（路径用文件选择器）
- 存储标签页、网络标签页（服务端 URL + 代理）、关于
- **状态覆盖：** 加载骨架屏、空列表（提示添加）、错误内联提示
- **复杂度：** M

### 2.8 已安装 Skill 页面
- 搜索栏、视图切换（网格/列表）、安全筛选、批量操作栏
- SkillCard（名称、描述预览、安全徽章、格式评分、开关）
- SkillDetail 侧面板（元数据、评估分数、已安装 Agent、操作按钮）
- AgentSelector 多选组件
- **状态覆盖：** 加载骨架网格、空状态（引导到发现页）、错误提示
- **复杂度：** L

### 2.9 发现（市场）页面
- 搜索（300ms 防抖）、分类筛选标签、排序（趋势/最新/高分）
- 分类横向滚动标签、精选推荐区、主内容网格、分页
- SkillDetail 侧面板（市场版：安装按钮 + Agent 选择器）
- **状态覆盖：** 加载骨架、无匹配结果、网络错误（显示缓存数据 + 离线徽章）
- **复杂度：** L

### 2.10 去重检测页面
- "检查重复"按钮 + 扫描状态指示器
- 结果列表：摘要 + DedupGroupCard（相似度百分比 + 双卡对比 + 操作按钮）
- ComparisonModal（并排对比 SKILL.md + 差异高亮）
- **状态覆盖：** 加载进度条、扫描前空态、无重复结果、扫描失败
- **复杂度：** M

---

## Milestone 3: 服务端 MVP

**依赖：** M0 | **可并行：** M1、M2 | **估算：** 3-4 周

### 3.1 PostgreSQL Schema 与迁移
```sql
-- skills 表：所有数据源统一索引
-- categories 表：分类
-- skill_categories 表：M:N 关联
-- pgvector 扩展：ALTER TABLE skills ADD COLUMN embedding vector(384)
-- 索引：name, source, license, category, rating DESC, install_count DESC
```
- **关键文件：** `server/src/db/`、`server/migrations/*.sql` | **复杂度：** M

### 3.2 REST API 端点（axum）
| 方法 | 路径 | 用途 |
|------|------|------|
| GET | `/api/v1/health` | 健康检查 |
| GET | `/api/v1/stats` | 聚合统计 |
| GET | `/api/v1/skills` | 分页 Skill 列表 |
| GET | `/api/v1/skills/search` | 关键词搜索 |
| GET | `/api/v1/skills/:id` | Skill 详情 |
| GET | `/api/v1/skills/:id/similar` | 相似 Skill |
| GET | `/api/v1/categories` | 分类列表 |
| POST | `/api/v1/skills/similarity` | 批量描述相似度 |
| POST | `/api/v1/crawl/trigger` | 手动触发爬取 |

- CORS 中间件、分页（默认 20/页，最大 100）、限速（100 req/min）
- **关键文件：** `server/src/api/` | **复杂度：** M

### 3.3 爬虫框架与调度器
- `tokio::time::interval` + ±15 分钟随机偏移，每 6 小时运行
- 首次部署时自动触发全量爬取（不等待定时器）
- 每个周期：运行所有适配器 → 管道处理 → upsert 入库
- 优雅关闭（SIGTERM 完成当前周期）
- **关键文件：** `server/src/crawler/{mod,scheduler}.rs` | **复杂度：** M

### 3.4 GitHub API 适配器
- GitHub REST API v3 搜索 `filename:SKILL.md license:mit,apache-2.0`
- 分页（100/页，最多 10 页），获取原始 SKILL.md、解析元数据
- 许可证二次过滤、限速处理（`X-RateLimit-Remaining`、退避）、指数退避重试
- 可配置：GITHUB_TOKEN、GITHUB_SEARCH_PER_PAGE、GITHUB_MAX_PAGES
- **关键文件：** `server/src/crawler/github_adapter.rs` | **复杂度：** L

### 3.5 SkillNet API 适配器
- 调用 SkillNet 公开搜索 API + 5 维评分 API
- 将 5 维评分映射到本地字段
- **关键文件：** `server/src/crawler/skillnet_adapter.rs` | **复杂度：** M

### 3.6 数据管道
```
适配器输出 → 去重器(hash) → 许可证过滤 → 格式校验 → 安全检查 → DB 写入
```
- 去重：SHA-256 content hash + source 联合唯一
- **关键文件：** `server/src/pipeline/` | **复杂度：** M

### 3.7 嵌入服务
- POST `/api/v1/skills/similarity` 接收 `{ descriptions: Vec<String> }`
- MVP：TF-IDF 余弦相似度（无外部依赖）
- 未来：ONNX Runtime + all-MiniLM-L6-v2
- **关键文件：** `server/src/embedding/` | **复杂度：** M

### 3.8 Docker Compose 与部署配置
- docker-compose.yml：pgvector:pg16 + API 服务 + 持久化卷
- 多阶段 Dockerfile、.env.example、seed-data.sh
- 种子数据脚本：预置 Anthropic 官方仓库 Skill（~20 个）及常用社区 SKILL.md，确保首次启动时市场不为空
- **复杂度：** M

### 3.9 LLM 评估服务

用户在安装 Skill 到本地时，客户端自动触发 LLM 质量评估。

**评估维度：**
- `description_score`：描述清晰度、完整性（0-100）
- `instructions_score`：指令步骤清晰度、可执行性（0-100）
- `scripts_score`：脚本/代码质量、正确性（0-100）
- `total_score`：综合评分（加权平均）

**API 端点：**
- `POST /api/v1/assess` — 接收 `{ skill_content: String }`，返回三维度评分 + 总评分 + 评估详情

**LLM Provider 配置：**
- 服务端后台可配置多个 Provider（OpenAI / Anthropic / 其他兼容 API）
- 同时只启用一个 Provider
- 每个 Provider 需配置：API Key、模型名、base URL
- Token 用量追踪：每次评估记录 prompt_tokens + completion_tokens

**额度管理：**
- 基于 Token 数的免费额度配置（如 10 万 tokens/月/用户）
- 超额后客户端弹窗提示绑定个人 API Key
- 管理员可在服务端后台调整额度配置

**关键文件：** `server/src/assessment/`、`server/src/llm/` | **复杂度：** M

---

## Milestone 4: 功能集成联调

**依赖：** M1、M2、M3 | **估算：** 2-3 周

### 4.1 安装/卸载流程联调
- 发现页 → 详情 → 安装 → 多选 Agent → 下载/校验/扫描/多目录写入/DB 记录
- UI：加载旋转器、进度条、成功 Toast、内联错误
- **复杂度：** M

### 4.2 启用/禁用
- 开关切换 → 更新 DB enabled 字段 → 重命名 Agent 目录副本（禁用后缀）
- UI：乐观更新、禁用状态渲染（低透明度 + 标签）
- **复杂度：** S

### 4.3 更新/同步流程
- 本地变更检测（mtime）+ 服务端版本检查 → "更新可用"徽章
- 用户触发 → 重新下载/校验 → 覆盖安装到各 Agent
- **复杂度：** M

### 4.4 去重检测集成
- 用户点击"扫描" → 本地名称相似度 + 服务端描述相似度 → 分组渲染
- 操作：保留两者、删除、对比（差异高亮）
- **复杂度：** M

### 4.5 质量评估集成
- "全部扫描" → 格式检查 + 安全检查 → 写入 assessment_results
- UI：仪表盘（总数、通过率、平均分、安全分布）、分数徽章、问题列表
- **复杂度：** M

### 4.6 设置持久化与边界情况
- 跨重启持久化、首次启动引导、异常恢复、并发操作保护
- **复杂度：** M

---

## Milestone 5: 打磨、测试与发布

**依赖：** M4 | **估算：** 2-3 周

### 5.1 错误处理审计
- IPC 命令返回具体错误码、前端友好提示
- 边界情况：磁盘满、权限拒绝、文件锁定、超时、数据损坏
- 文件日志（`~/.skillbase/logs/`）
- **复杂度：** M

### 5.2 UI 打磨
- 加载骨架屏、进度条、空状态、错误状态全覆盖
- 离线检测、快捷键、过渡动画、明暗主题一致性
- **复杂度：** M

### 5.3 测试
- Rust：DB（内存 SQLite）、格式检查器、安全检查器、去重引擎、安装器
- React：Store 逻辑、组件渲染（Vitest + RTL）
- 集成：Tauri 命令测试、服务端 API 端点测试
- **复杂度：** L

### 5.4 CI/CD
- `ci.yml`：lint、cargo build、cargo test、clippy、vitest
- `release.yml`：服务端 Docker 镜像 + Tauri 桌面包（macOS/Linux/Windows）
- **复杂度：** M

### 5.5 跨平台构建验证
- macOS universal binary、Linux AppImage、Windows MSI
- 各平台路径/权限/沙箱适配
- **复杂度：** M

### 5.6 文档
- README.md、CONTRIBUTING.md、开发指南、用户手册
- **复杂度：** S

---

## 服务端 API 端点汇总

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/health` | 健康检查 |
| GET | `/api/v1/stats` | 聚合统计（总数、各源数量、平均分） |
| GET | `/api/v1/skills?page=&per_page=&category=&sort=` | 分页列表（默认 20/页） |
| GET | `/api/v1/skills/search?q=&category=&license=` | 关键词搜索 |
| GET | `/api/v1/skills/:id` | 完整详情 |
| GET | `/api/v1/skills/:id/similar` | 相似 Skill |
| GET | `/api/v1/categories` | 全部分类 |
| POST | `/api/v1/skills/similarity` | 批量描述相似度（body: `{descriptions: Vec<String>}`） |
| POST | `/api/v1/crawl/trigger` | 手动触发爬取 |
| POST | `/api/v1/assess` | LLM 质量评估（body: `{skill_content: String}`）|

## 客户端 SQLite Schema（5 表）

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `installed_skills` | 本地已安装 Skill | id, name, source, installed_at, enabled |
| `agent_configs` | Agent 路径配置 | id, name, agent_type, skill_dir |
| `install_mappings` | Skill ↔ Agent 映射 | skill_id, agent_id, installed_at |
| `app_settings` | 键值设置 | key (UNIQUE), value |
| `assessment_results` | 评估结果 | skill_id, dimension, score, issues (JSON) |

## 服务端 PostgreSQL Schema

| 表名 | 用途 | 关键字段 |
|------|------|----------|
| `skills` | 统一 Skill 索引 | id(UUID), name, description, source, source_url, license, content_hash, skill_md_content (TEXT), safety_level, format_score, quality_score, embedding (vector(384)) |
| `categories` | 分类 | id, name, display_name |
| `skill_categories` | M:N 关联 | skill_id, category_id |

## Rust 依赖清单

### 客户端（Tauri Rust 后端）
```
tauri = "2", tauri-plugin-{shell,fs,dialog} = "2"
serde = "1", serde_json = "1", serde_yaml = "0.9"
rusqlite = { version = "0.31", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
syntect = "5", ignore = "0.4"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 服务端（Rust 后端 API + 爬虫）
```
axum = "0.7", tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = "1", serde_json = "1"
sqlx = { version = "0.7", features = ["postgres", "uuid", "chrono", "migrate"] }
reqwest = { version = "0.12", features = ["json"] }
tracing = "0.1", tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
```

## 关键文件索引

| 优先级 | 文件 | 说明 |
|--------|------|------|
| P0 | `packages/client/src-tauri/src/lib.rs` | Tauri 入口、插件注册、命令注册、启动钩子 |
| P0 | `packages/client/src-tauri/src/db/repository.rs` | SQLite CRUD 基础，所有功能依赖 |
| P0 | `packages/client/src-tauri/src/commands/` | IPC 接口层，变更成本高 |
| P0 | `packages/client/src/pages/DiscoverPage.tsx` | 最复杂页面：搜索/筛选/分页/详情面板 |
| P0 | `packages/client/src/pages/InstalledPage.tsx` | 最复杂页面：列表/评估/批量操作/更新 |
| P1 | `packages/server/src/crawler/github_adapter.rs` | 主要数据来源，处理限速/分页/重试 |
| P1 | `packages/client/src-tauri/src/installer/agent_installer.rs` | 多 Agent 安装/同步核心 |
| P1 | `packages/server/src/api/skills.rs` | 服务端核心 API 端点 |

## 风险与缓解

| 风险 | 可能性 | 影响 | 缓解方案 |
|------|--------|------|----------|
| SkillNet API 不稳定 | 中 | 高 | 适配器可独立禁用，仅 GitHub 数据也能运行 |
| Tauri 2.0 API 变更 | 低 | 中 | 锁定版本，关注 changelog |
| pgvector 部署复杂 | 中 | 低 | MVP 用 TF-IDF，不依赖 pgvector |
| GitHub API 限速 | 中 | 高 | 使用 Token（5000 req/hr），智能退避，缓存 |
| 跨平台文件系统差异 | 低 | 中 | 全程使用 `std::path::Path`，CI 多平台验证 |
