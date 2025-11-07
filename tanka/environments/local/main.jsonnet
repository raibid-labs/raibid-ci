// main.jsonnet - Local environment configuration
// This is the main entry point for the local k3s environment

local config = import 'raibid/config.libsonnet';
local k8s = import 'k8s.libsonnet';

// Import chart wrappers
local redis = import 'charts/redis.libsonnet';
local gitea = import 'charts/gitea.libsonnet';
local keda = import 'charts/keda.libsonnet';
local flux = import 'charts/flux.libsonnet';

// Import application components
local server = import 'raibid/server.libsonnet';
local agent = import 'raibid/agent.libsonnet';
local secrets = import 'raibid/secrets.libsonnet';
local mirrorSetupJob = import 'raibid/mirror-setup-job.libsonnet';

// Environment configuration
local namespace = config.namespace;
local domain = 'localhost';

// Create all resources
{
  // Namespace
  namespace: k8s.namespace(namespace),

  // Infrastructure Components (WS2)

  // Redis - Job queue with Streams support
  redis: redis.new(namespace, 'redis'),

  // Gitea - Git server with OCI registry
  gitea: gitea.new(namespace, 'gitea', domain),

  // KEDA - Event-driven autoscaling
  keda: keda.new(namespace, 'keda'),

  // Flux - GitOps continuous delivery
  flux: flux.new(namespace, 'flux'),

  // Flux GitRepository CRD - Points to Gitea repo
  fluxGitRepo: flux.crds.gitRepository(
    'raibid-ci',
    namespace,
    'http://gitea.%s.svc.cluster.local:3000/raibid/raibid-ci.git' % namespace,
    'main',
    '1m0s'
  ),

  // Application Components (WS3)

  // Secrets and ConfigMaps
  config: secrets.configMap(
    namespace,
    redisUrl='redis://redis-master:6379',
    giteaUrl='http://gitea:3000'
  ),

  secrets: secrets.secret(namespace),

  // Mirror Setup Job - Sets up GitHub repository mirroring in Gitea
  mirrorSetup: mirrorSetupJob.new(namespace),

  // Server - API server
  // Note: Images pushed to localhost:3000 via Tilt, k8s pulls from short service name
  // Using short name 'gitea-http:3000' instead of FQDN for simpler resolution
  server: server.new(
    namespace,
    'raibid-server',
    'gitea-http:3000/raibid-admin/raibid-server:latest',
    replicas=1
  ),

  // Agent - Auto-scaling build agents
  agent: agent.new(
    namespace,
    'raibid-agent',
    'gitea-http:3000/raibid-admin/raibid-agent:latest',
    redisAddress='redis-master:6379',
    streamName='raibid:jobs',
    consumerGroup='raibid-agents',
    pollingInterval=10,
    maxReplicaCount=10,
    lagThreshold=5
  ),
}
