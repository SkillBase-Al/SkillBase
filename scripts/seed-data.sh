#!/usr/bin/env bash
# Seed initial skills from Anthropic official repository
set -euo pipefail

SKILLS_DIR="$HOME/.skillbase/skills"
mkdir -p "$SKILLS_DIR"

echo "Seeding skills into $SKILLS_DIR ..."

# A list of well-known skills to seed (minimal SKILL.md examples)
declare -A SKILLS
SKILLS["code-review"]='---
name: code-review
description: Comprehensive code review assistant that analyzes pull requests for bugs, security issues, performance problems, and code style violations. Provides detailed feedback with specific line-level annotations.
version: 1.0.0
author: SkillBase
tags:
  - code-review
  - development
  - quality
---

# Code Review

Analyzes code changes and provides thorough review feedback.

## Usage

Ask the agent to review your code changes or a specific file.

## Requirements

- Git repository with changes to review
'

SKILLS["pdf-processing"]='---
name: pdf-processing
description: Extract text, tables, and metadata from PDF documents. Supports batch processing of multiple PDF files with configurable output formats including Markdown and JSON.
version: 1.0.0
author: SkillBase
tags:
  - pdf
  - document
  - text-extraction
requires:
  - pdftotext
---

# PDF Processing

Extract and process content from PDF files.

## Usage

Provide a PDF file path to extract its contents.

## Requirements

- pdftotext (part of poppler-utils)
'

SKILLS["commit-message"]='---
name: commit-message
description: Generate conventional commit messages based on staged git diff analysis. Follows Conventional Commits specification with support for custom scopes and change types.
version: 1.0.0
author: SkillBase
tags:
  - git
  - commit
  - conventional-commits
---

# Commit Message Generator

Analyzes git diff and generates conventional commit messages.

## Usage

Stage your changes and ask the agent to generate a commit message.
'

SKILLS["unit-test-generator"]='---
name: unit-test-generator
description: Automatically generate comprehensive unit tests for your codebase. Analyzes function signatures, input/output patterns, and edge cases to create thorough test coverage.
version: 1.0.0
author: SkillBase
tags:
  - testing
  - unit-tests
  - quality
---

# Unit Test Generator

Creates unit tests for your functions and modules.

## Usage

Select a function or file to generate tests for.

## Requirements

- Appropriate test framework for your language
'

SKILLS["api-documenter"]='---
name: api-documenter
description: Generate OpenAPI/Swagger documentation from code comments and route definitions. Supports Express, FastAPI, and other popular frameworks with automatic schema detection.
version: 1.0.0
author: SkillBase
tags:
  - api
  - documentation
  - openapi
---

# API Documenter

Generates API documentation from your code.

## Usage

Point the agent to your API route definitions to generate documentation.
'

for name in "${!SKILLS[@]}"; do
    dir="$SKILLS_DIR/$name"
    mkdir -p "$dir"
    echo "${SKILLS[$name]}" > "$dir/SKILL.md"
    echo "  ✓ Seeded: $name"
done

echo ""
echo "Seeded ${#SKILLS[@]} skills successfully."
