# raibid-ci Documentation

Welcome to the raibid-ci documentation. This directory is organized by component and purpose.

## Documentation Structure

```
docs/
├── README.md                 # This file
├── architecture/             # High-level system design
├── components/              # Component-specific docs
├── guides/                  # Tutorials and how-tos
├── api/                     # API specifications
├── templates/               # Documentation templates
├── diagrams/                # Architecture diagrams
├── workstreams/            # Development planning
└── work/                    # Project planning
```

## Quick Links

### Getting Started
- [User Guide](./USER_GUIDE.md) - Complete end-user documentation

### Component Documentation
- [Infrastructure](./components/infrastructure/README.md) - k3s, Gitea, Redis, KEDA, Flux

### Architecture
- [Orchestration](./architecture/orchestration.md) - Multi-agent development
- [Event-Driven Orchestration](./architecture/event-driven-orchestration.md) - Event-based architecture

### Guides
- [Error Recovery](./guides/error-recovery.md) - Infrastructure failure recovery

### Infrastructure
- [Gitea Setup](./components/infrastructure/gitea.md) - Git server + OCI registry
- [Redis Deployment](./components/infrastructure/redis-deployment.md) - Job queue
- [Redis Usage](./components/infrastructure/redis-usage.md) - Redis Streams
- [KEDA Setup](./components/infrastructure/keda.md) - Autoscaling

### Development
- [Development Workflow](./workstreams/START_HERE.md) - Multi-agent development

## For Contributors

Run documentation validation: `./scripts/check-docs.sh`

---

*Last Updated: 2025-11-01*
*Organized by component for Issue #44*
