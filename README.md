# iciaws-cli

A command-line scaffolding tool that generates AWS serverless API projects from the [`iciaws`](https://github.com/intercci/iciaws) library ecosystem.

## Overview

`iciaws-cli` bootstraps a production-ready Rust Lambda + API Gateway project in seconds. It embeds a full template, customises it interactively, and outputs a ready-to-deploy codebase — no manual file copying or editing required.

## Features

- **Interactive setup** — prompts for project name, description, and which AWS services to include (S3, SES, SNS)
- **Smart pruning** — removes unselected service crates, client initialisation, and addon registrations automatically
- **Variable substitution** — replaces `$NAME$`, `__PROJECT_NAME__`, and `__DESCRIPTION__` placeholders across all template files
- **Zero runtime overhead** — the template is embedded at compile time via a build script; no extra dependencies at runtime beyond what the generated project needs
- **AI-ready** — includes a `CLAUDE.md` in every generated project with detailed conventions for extending models and handlers with an AI agent

## Prerequisites

- [Rust 1.96+](https://www.rust-lang.org/tools/install) (edition 2024)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html) (for the generated projects)

## Installation

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
```

The binary will be at `target/release/iciaws-cli`.

## Usage

Run the interactive wizard:

```bash
cargo run
```

or after installation:

```bash
iciaws-cli
```

You'll be guided through:

1. **Project name** — alphanumeric, hyphens, underscores only (no spaces), 1–30 characters. The tool checks that the folder doesn't already exist.
2. **Short description** — displayed in the SAM template's `Description` field.
3. **Service selection** — multi-select checkbox for S3, SES, and SNS. Unselected services are stripped from the generated Cargo.toml and src/main.rs.

After scaffolding completes:

```bash
cd my-project
cargo lambda watch          # run locally
./deploy.sh                 # deploy to AWS
```

## Generated Project Structure

Each scaffolded project follows this layout:

```
my-project/
├── Cargo.toml              # Dependencies (iciaws_* crates, Lambda runtime)
├── template.yaml           # AWS SAM deployment config
├── template-local.yaml     # Local integration test config (DynamoDB local)
├── deploy.sh               # One-command deploy script
├── rebuild.sh              # Clean + build + local test
├── run-local.sh            # Start SAM local API
├── .env                    # Environment variables
├── CLAUDE.md               # AI agent instructions for extending the project
├── src/
│   ├── main.rs             # Lambda entrypoint
│   ├── routes.rs           # Route registration
│   ├── common/             # Utilities, env vars, timestamps
│   ├── models/             # DynamoDB models with CRUD
│   └── handlers/           # HTTP request handlers
```

## Architecture

```
Client → API Gateway → Lambda → DynamoDB (single-table)
                     ↓
                 S3 / SES / SNS (optional)
```

- Single Lambda function serving all REST API routes
- Single DynamoDB table with single-table design (composite pk/sk keys)
- External cookie-based auth via `iciauth` Lambda authorizer

## License

MIT License — see [LICENSE](LICENSE).
