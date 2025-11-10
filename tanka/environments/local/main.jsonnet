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
  // Note: Tilt will automatically add registry prefix (localhost:5000 for build/push,
  // k3d-registry:5000 for k8s deployment via default_registry config)
  server: server.new(
    namespace,
    'raibid-server',
    'raibid-server:latest',
    replicas=1
  ),

  // Agent - Auto-scaling build agents
  // Note: Using k3d-registry prefix since Tanka deploys directly (not via Tilt)
  agent: agent.new(
    namespace,
    'raibid-agent',
    'k3d-registry:5000/raibid-agent:latest',
    redisAddress='redis-master:6379',
    streamName='raibid:jobs',
    consumerGroup='raibid-agents',
    pollingInterval=10,
    minReplicaCount=1,
    maxReplicaCount=10,
    lagThreshold=1,
    env={
      GITEA_HOST: 'gitea-http.%s.svc.cluster.local:3000' % namespace,
    }
  ),
}
