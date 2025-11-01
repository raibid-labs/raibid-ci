# raibid-ci Documentation

Welcome to the raibid-ci documentation. This directory is organized by component and purpose to help you quickly find what you need.

## Documentation Structure

```
docs/
├── README.md                 # This file - navigation guide
├── architecture/             # High-level system design
├── components/              # Component-specific documentation
│   ├── server/              # API server documentation
│   ├── cli/                 # CLI application documentation
│   ├── tui/                 # Terminal UI documentation
│   ├── agent/               # CI agent documentation
│   └── infrastructure/      # Infrastructure setup & operations
├── guides/                  # Tutorials and how-to guides
├── api/                     # API specifications and contracts
├── templates/               # Documentation templates
├── diagrams/                # Architecture diagrams
├── workstreams/            # Development workstream planning
└── work/                    # Project planning documents
```

## Quick Links

### Getting Started
- [User Guide](./USER_GUIDE.md) - Complete end-user documentation
- [Installation Guide](./guides/installation.md) - Setting up raibid-ci
- [Quick Start](./guides/quick-start.md) - Get running in 5 minutes

### Component Documentation
- [Server API](./components/server/README.md) - API server architecture and implementation
- [CLI](./components/cli/README.md) - Command-line interface
- [TUI](./components/tui/README.md) - Terminal user interface
- [CI Agents](./components/agent/README.md) - Build agent implementations
- [Infrastructure](./components/infrastructure/README.md) - k3s, Gitea, Redis, KEDA, Flux

### Architecture
- [System Architecture](./architecture/system-overview.md) - High-level architecture
- [Data Flow](./architecture/data-flow.md) - How data moves through the system
- [Scaling Strategy](./architecture/scaling.md) - Auto-scaling design
- [Security Model](./architecture/security.md) - Authentication and authorization

### API Documentation
- [REST API Reference](./api/rest-api.md) - HTTP endpoints
- [WebSocket API](./api/websocket.md) - Real-time updates
- [Redis Streams Protocol](./api/redis-streams.md) - Job queue format

### Guides & Tutorials
- [Infrastructure Setup](./guides/infrastructure-setup.md) - Setting up k3s cluster
- [Agent Development](./guides/agent-development.md) - Creating custom agents
- [Repository Mirroring](./guides/repository-mirroring.md) - GitHub to Gitea sync
- [Troubleshooting](./guides/troubleshooting.md) - Common issues and solutions
- [Error Recovery](./error-recovery.md) - Handling infrastructure failures

### Development
- [Contributing Guide](./guides/contributing.md) - How to contribute
- [Development Workflow](./workstreams/START_HERE.md) - Multi-agent development
- [Testing Strategy](./guides/testing.md) - Unit, integration, and E2E tests
- [Release Process](./guides/release-process.md) - Publishing new versions

### Infrastructure Components
- [k3s Setup](./components/infrastructure/k3s.md) - Kubernetes cluster
- [Gitea Setup](./components/infrastructure/gitea.md) - Git server + OCI registry
- [Redis Setup](./components/infrastructure/redis.md) - Job queue
- [KEDA Setup](./components/infrastructure/keda.md) - Autoscaling
- [Flux Setup](./components/infrastructure/flux.md) - GitOps delivery

### Research & Planning
- [Technology Research](./technology-research.md) - Technology evaluation
- [Orchestration Design](./ORCHESTRATION.md) - Multi-agent orchestration
- [Project Plan](./work/plan.md) - Milestone roadmap

## Documentation Standards

### Writing Style
- **Concise**: Use bullet points and short paragraphs
- **Scannable**: Include headers, code blocks, and lists
- **Example-driven**: Show code examples for all features
- **Complete**: Include all edge cases and error scenarios

### Markdown Conventions
- Use `## Heading 2` for major sections
- Use `### Heading 3` for subsections
- Code blocks must specify language: ` ```rust `, ` ```bash `, ` ```yaml `
- Include links to related documentation
- Use admonitions for important notes: `> **Note:** ...`

### Documentation Templates
Use these templates for consistency:
- [Component README](./templates/component-readme.md)
- [API Endpoint](./templates/api-endpoint.md)
- [Guide Template](./templates/guide.md)
- [Architecture Decision Record](./templates/adr.md)

### Diagrams
- Use Mermaid for all diagrams
- Store in `diagrams/` directory
- Include source code in markdown files
- Keep diagrams simple and focused

## For Contributors

### Adding New Documentation
1. Choose the appropriate directory based on content type
2. Use the relevant template from `templates/`
3. Follow the documentation standards above
4. Update this README if adding new sections
5. Run documentation CI checks (see below)

### Documentation CI
All documentation changes are validated:
- Markdown linting (markdownlint)
- Link checking (broken link detection)
- Code block validation (syntax checking)
- Diagram rendering (Mermaid validation)

Run locally:
```bash
# Lint markdown files
cargo install mdbook-mermaid
mdbook test

# Check links
npm install -g markdown-link-check
find docs -name "*.md" -exec markdown-link-check {} \;
```

## Project Context

**raibid-ci** is a DGX Spark Personal CI Agent Pool - an ephemeral, auto-scaling build system for cross-platform native compilation on NVIDIA DGX Spark.

### Key Features
- **Ephemeral agents**: Scale to zero when idle
- **Auto-scaling**: KEDA-driven scaling based on job queue
- **TUI-first**: Terminal interface for all management
- **GitOps**: Flux manages all deployments
- **Plugin-based**: Extensible agent architecture

### Target Platform
- **Hardware**: NVIDIA DGX Spark (ARM64)
- **OS**: Ubuntu 22.04 LTS
- **Resources**: 20 cores, 128GB RAM, 4TB NVMe, 200 Gb/s network

## Need Help?

- **User questions**: See [User Guide](./USER_GUIDE.md)
- **Installation issues**: See [Troubleshooting](./guides/troubleshooting.md)
- **Development help**: See [Contributing Guide](./guides/contributing.md)
- **Bug reports**: [GitHub Issues](https://github.com/raibid-labs/raibid-ci/issues)

---

*Last Updated: 2025-11-01*
