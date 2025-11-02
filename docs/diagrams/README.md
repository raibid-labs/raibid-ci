# Raibid-CI Architecture Diagrams

This directory contains comprehensive Mermaid diagrams documenting the raibid-ci DGX Spark CI Agent Pool architecture.

## Diagram Overview

### 1. System Architecture (`system-architecture.mmd`)
**Purpose**: High-level view of all system components and their relationships

**Key Elements**:
- NVIDIA DGX Spark hardware layer (ARM64, Grace Hopper)
- k3s cluster namespaces (gitops-system, keda-system, raibid-ci)
- Core services (Gitea, Redis, Flux, KEDA)
- Ephemeral CI agent pods
- Persistent storage volumes
- External integrations (GitHub, TUI client)

**Use Case**: Understanding overall system topology and component interactions

---

### 2. Build Workflow (`build-workflow.mmd`)
**Purpose**: End-to-end flow of a CI build job from code push to completion

**Key Stages**:
1. Developer push to GitHub
2. GitHub webhook triggers Gitea mirror
3. Job submitted to Redis Stream
4. KEDA detects job and scales up agent pod
5. Agent executes build pipeline
6. Results published to Gitea OCI registry
7. Agent terminates, KEDA scales down

**Use Case**: Troubleshooting build pipelines and understanding auto-scaling behavior

---

### 3. Component Interactions (`component-interactions.mmd`)
**Purpose**: Detailed sequence diagram showing API calls and data flow between components

**Key Interactions**:
- TUI client ↔ Rust API Server
- Rust API ↔ Redis Streams (job queue operations)
- KEDA ↔ Kubernetes API (scaling decisions)
- CI Agent ↔ Gitea (code fetch, image push/pull)
- Flux ↔ Gitea (GitOps reconciliation)

**Use Case**: Debugging integration issues and understanding timing/sequencing

---

### 4. Deployment Architecture (`deployment-architecture.mmd`)
**Purpose**: Kubernetes deployment topology with resource allocation and networking

**Key Details**:
- k3s single-node cluster on DGX Spark
- Namespace organization (kube-system, gitops-system, keda-system, raibid-ci)
- Deployment vs StatefulSet patterns
- PersistentVolumeClaim bindings
- Network policies and ingress routing
- Resource requests/limits (CPU, RAM, GPU)
- NodePort and ClusterIP services

**Use Case**: Infrastructure planning, resource optimization, network troubleshooting

---

## Viewing the Diagrams

### Option 1: Mermaid Live Editor
1. Visit https://mermaid.live/
2. Copy/paste diagram code
3. View rendered diagram and export as PNG/SVG

### Option 2: GitHub Rendering
GitHub automatically renders `.mmd` files in the web interface.

### Option 3: VS Code Extension
1. Install "Mermaid Preview" extension
2. Open `.mmd` file
3. Use command palette: "Mermaid: Preview Diagram"

### Option 4: Mermaid CLI
```bash
# Install Mermaid CLI
npm install -g @mermaid-js/mermaid-cli

# Render to PNG
mmdc -i system-architecture.mmd -o system-architecture.png

# Render to SVG
mmdc -i build-workflow.mmd -o build-workflow.svg -b transparent
```

---

## Diagram Conventions

### Color Coding

- **Ephemeral Components** (orange, dashed): CI agent pods that auto-scale to zero
- **Persistent Services** (blue/green): Long-running deployments and stateful sets
- **External Systems** (purple): GitHub, TUI client, external networks
- **Hardware** (gray): DGX Spark physical resources
- **Storage** (purple): PersistentVolumes and PVCs
- **Network** (cyan): Services, ingress, network policies

### Arrows

- **Solid arrows** (`-->`, `->>`, `->>`): Active data flow or API calls
- **Dotted arrows** (`-.->`, `-->>` in sequence): Configuration, mounting, or async operations
- **Bidirectional**: Request/response pairs

---

## Architecture Highlights

### Scale-to-Zero Design
- CI agents start at 0 replicas
- KEDA monitors Redis Stream queue length
- Agents spawn on-demand and terminate after job completion
- Typical scale-up time: ~5-10 seconds

### ARM64 Optimization
- All container images built for ARM64 (NVIDIA Grace CPU)
- OCI registry in Gitea caches layers locally
- Reduces build times by 40-60% vs remote registries

### GitOps Workflow
- Flux continuously reconciles cluster state from Gitea
- Infrastructure as Code in Git
- Automated rollbacks and drift detection

### Resource Isolation
- Separate namespaces for system components
- Network policies restrict cross-namespace traffic
- GPU allocation controlled via resource limits

---

## Maintenance

### Updating Diagrams

When updating diagrams:
1. Edit the `.mmd` file directly
2. Validate syntax at https://mermaid.live/
3. Update this README if adding new diagrams
4. Commit with descriptive message (e.g., "docs: add GPU scheduling to deployment diagram")

### Diagram Versioning

Diagrams follow the project version:
- **Breaking architecture changes**: Commit with `[breaking]` tag
- **Minor updates**: Standard commit
- **Clarifications**: `docs:` prefix in commit message

---

## Related Documentation


---

## Questions & Feedback

For questions about the architecture diagrams:
1. Open an issue with `[docs]` label
2. Reference the specific diagram and section
3. Provide context for your use case

For diagram feature requests:
- Suggest new diagram types (e.g., disaster recovery flow)
- Propose alternative visualizations
- Report rendering issues
