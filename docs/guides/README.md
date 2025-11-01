# Guides and Tutorials

This directory contains step-by-step guides and tutorials for using and developing raibid-ci.

## Getting Started Guides

### [Quick Start](./quick-start.md)
Get raibid-ci running in 5 minutes.

**Contents:**
- Prerequisites
- Installation
- First run
- Creating your first job

### [Installation Guide](./installation.md)
Detailed installation instructions for different platforms.

**Contents:**
- System requirements
- Platform-specific instructions (Linux, macOS, Windows)
- DGX Spark ARM64 build
- Verification steps

### [Configuration Guide](./configuration.md)
Complete configuration reference.

**Contents:**
- Configuration file format
- Configuration precedence
- Environment variables
- Common configurations

## Infrastructure Guides

### [Infrastructure Setup](./infrastructure-setup.md)
Complete guide to setting up the k3s cluster and all components.

**Contents:**
- k3s installation
- Gitea deployment
- Redis setup
- KEDA configuration
- Flux bootstrap
- End-to-end verification

### [k3s Cluster Setup](./k3s-setup.md)
Detailed k3s cluster configuration.

**Contents:**
- Single-node vs multi-node
- Rootless mode
- Custom configurations
- Upgrading k3s

### [Repository Mirroring](./repository-mirroring.md)
Setting up GitHub to Gitea repository synchronization.

**Contents:**
- Single repository mirroring
- Organization mirroring
- Webhook setup
- Auto-sync configuration

## Development Guides

### [Contributing Guide](./contributing.md)
How to contribute to raibid-ci.

**Contents:**
- Development workflow
- Code style guidelines
- Testing requirements
- Pull request process

### [Testing Guide](./testing.md)
Testing strategy and best practices.

**Contents:**
- Unit testing
- Integration testing
- End-to-end testing
- TDD workflow

### [Agent Development](./agent-development.md)
Creating custom CI agents.

**Contents:**
- Agent architecture
- Pipeline execution
- Custom language support
- Testing agents

### [TUI Development](./tui-development.md)
Developing TUI features.

**Contents:**
- Ratatui basics
- Widget development
- Event handling
- State management
- Testing TUI components

## Operations Guides

### [Troubleshooting](./troubleshooting.md)
Common issues and solutions.

**Contents:**
- Installation issues
- Infrastructure failures
- Performance problems
- Network connectivity
- Debugging tips

### [Error Recovery](./error-recovery.md)
Recovering from infrastructure failures.

**Contents:**
- k3s recovery
- Redis recovery
- Gitea recovery
- Flux recovery
- KEDA recovery
- Rollback procedures

### [Performance Tuning](./performance-tuning.md)
Optimizing system performance.

**Contents:**
- Agent scaling tuning
- Build cache optimization
- Network optimization
- Resource allocation

### [Backup and Restore](./backup-restore.md)
Backing up and restoring raibid-ci state.

**Contents:**
- What to backup
- Backup procedures
- Restore procedures
- Disaster recovery

## Advanced Topics

### [Pipeline Reference](./pipeline-reference.md)
Complete pipeline configuration reference.

**Contents:**
- Pipeline YAML format
- Build steps
- Environment variables
- Secrets management
- Caching configuration

### [Caching Strategy](./caching.md)
Build cache optimization.

**Contents:**
- Cache types
- Cache configuration
- Cache invalidation
- Best practices

### [Multi-Cluster Setup](./multi-cluster.md)
Running raibid-ci across multiple DGX Spark nodes.

**Contents:**
- Multi-node k3s cluster
- Redis clustering
- Load balancing
- High availability

### [Security Hardening](./security.md)
Securing your raibid-ci installation.

**Contents:**
- Network policies
- RBAC configuration
- Secret management
- Image scanning
- Audit logging

## Release Guides

### [Release Process](./release-process.md)
How to create and publish releases.

**Contents:**
- Version numbering
- Changelog generation
- Build process
- Publishing artifacts
- GitHub releases

## Migration Guides

### [Upgrading](./upgrading.md)
Upgrading raibid-ci to newer versions.

**Contents:**
- Version compatibility
- Migration steps
- Breaking changes
- Rollback procedures

## Documentation Standards

All guides should follow these conventions:

### Structure
```markdown
# Guide Title

Brief description (1-2 sentences)

## Prerequisites
- List all prerequisites
- Links to other guides if needed

## Step 1: First Step
Detailed instructions with code blocks

## Step 2: Second Step
Detailed instructions with code blocks

## Verification
How to verify the guide worked

## Troubleshooting
Common issues specific to this guide

## Next Steps
Links to related guides

## Related Documentation
Links to other relevant docs
```

### Style Guidelines
- **Imperative mood**: "Install k3s" not "Installing k3s"
- **Code blocks**: Always specify language
- **Screenshots**: Use when helpful, store in `images/`
- **Examples**: Include real, working examples
- **Testing**: All commands must be tested

### Code Block Format
```bash
# Comment explaining what this does
command --with-flags argument

# Expected output:
# Output text here
```

## Related Documentation

- [Component Documentation](../components/)
- [Architecture](../architecture/)
- [API Reference](../api/)
- [User Guide](../USER_GUIDE.md)

---

*Last Updated: 2025-11-01*
