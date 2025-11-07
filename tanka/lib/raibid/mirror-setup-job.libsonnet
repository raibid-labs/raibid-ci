// mirror-setup-job.libsonnet - Kubernetes Job for setting up Gitea repository mirrors
// This Job runs after Gitea is deployed and sets up mirroring from GitHub

local config = import './config.libsonnet';

{
  // Create a Job that sets up repository mirroring
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Job name (default: 'setup-gitea-mirrors')
  new(namespace, name='setup-gitea-mirrors'):: {
    apiVersion: 'batch/v1',
    kind: 'Job',
    metadata: {
      name: name,
      namespace: namespace,
      labels: {
        'app.kubernetes.io/name': 'raibid-ci',
        'app.kubernetes.io/component': 'mirror-setup',
        'app.kubernetes.io/managed-by': 'tanka',
      },
    },
    spec: {
      // Don't retry on failure - let user manually trigger
      backoffLimit: 0,
      // Clean up after 1 hour
      ttlSecondsAfterFinished: 3600,
      template: {
        metadata: {
          labels: {
            'app.kubernetes.io/name': 'raibid-ci',
            'app.kubernetes.io/component': 'mirror-setup',
          },
        },
        spec: {
          restartPolicy: 'Never',
          volumes: [
            {
              name: 'token-volume',
              emptyDir: {},
            },
          ],
          initContainers: [
            {
              name: 'create-gitea-token',
              image: 'gitea/gitea:1.21.0',
              command: ['/bin/sh', '-c'],
              args: [|||
                set -e
                echo "Creating Gitea API token via CLI..."

                # Wait for Gitea to be ready
                for i in $(seq 1 60); do
                  if wget -q -O /dev/null http://gitea-http.%(namespace)s.svc.cluster.local:3000/api/v1/version 2>/dev/null; then
                    echo "✓ Gitea API is ready"
                    break
                  fi
                  if [ $i -eq 60 ]; then
                    echo "✗ Timeout waiting for Gitea"
                    exit 1
                  fi
                  echo "Waiting for Gitea... ($i/60)"
                  sleep 5
                done

                # Use Gitea CLI to create token (doesn't require password)
                TOKEN=$(gitea --config /data/gitea/conf/app.ini admin user generate-access-token \
                  --username raibid-admin \
                  --token-name "raibid-mirror-$(date +%%s)" \
                  --scopes write:organization,write:repository,write:user,write:misc | \
                  grep "successfully created" | awk '{print $NF}')

                if [ -z "$TOKEN" ]; then
                  echo "✗ Failed to create token"
                  exit 1
                fi

                echo "$TOKEN" > /token/gitea-token
                echo "✓ Token created and saved"
              ||| % { namespace: namespace }],
              volumeMounts: [
                {
                  name: 'token-volume',
                  mountPath: '/token',
                },
              ],
            },
          ],
          containers: [
            {
              name: 'setup-mirrors',
              image: 'redis:8-alpine',
              command: ['/bin/sh', '-c'],
              args: [|||
                set -e

                # Install curl for API calls
                apk add --no-cache curl

                echo "=== Gitea Repository Mirroring Setup ==="
                echo "Waiting for Gitea API to be ready..."

                # Wait for Gitea to be ready (up to 10 minutes)
                for i in $(seq 1 120); do
                  if curl -f -s http://gitea-http.%(namespace)s.svc.cluster.local:3000/api/v1/version > /dev/null 2>&1; then
                    echo "✓ Gitea API is ready"
                    break
                  fi
                  if [ $i -eq 120 ]; then
                    echo "✗ Timeout waiting for Gitea API"
                    exit 1
                  fi
                  echo "Waiting for Gitea... ($i/120)"
                  sleep 5
                done

                # Wait for Redis to be ready
                echo "Waiting for Redis to be ready..."
                for i in $(seq 1 60); do
                  if redis-cli -h redis-master.%(namespace)s.svc.cluster.local ping > /dev/null 2>&1; then
                    echo "✓ Redis is ready"
                    break
                  fi
                  if [ $i -eq 60 ]; then
                    echo "✗ Timeout waiting for Redis"
                    exit 1
                  fi
                  echo "Waiting for Redis... ($i/60)"
                  sleep 2
                done

                # Verify credentials are available
                if [ -z "$GITHUB_TOKEN" ]; then
                  echo "⚠ GITHUB_TOKEN not set - skipping mirroring"
                  exit 0
                fi

                if [ -z "$GITEA_ADMIN_USER" ] || [ -z "$GITEA_ADMIN_PASSWORD" ]; then
                  echo "⚠ Gitea admin credentials not set - skipping mirroring"
                  exit 0
                fi

                echo "✓ Credentials available"

                # Read Gitea API token from file (created by init container)
                if [ ! -f /token/gitea-token ]; then
                  echo "✗ Gitea token file not found"
                  exit 1
                fi

                GITEA_TOKEN=$(cat /token/gitea-token)
                if [ -z "$GITEA_TOKEN" ]; then
                  echo "✗ Gitea token is empty"
                  exit 1
                fi

                echo "✓ Using Gitea API token"
                echo "Fetching raibid-labs private repositories..."

                # Fetch all private repos from raibid-labs organization
                REPOS=$(curl -s -H "Authorization: token $GITHUB_TOKEN" \
                  "https://api.github.com/orgs/raibid-labs/repos?type=private&per_page=100" | \
                  grep -o '"clone_url": "[^"]*"' | cut -d'"' -f4)

                if [ -z "$REPOS" ]; then
                  echo "⚠ No private repositories found in raibid-labs organization"
                  exit 0
                fi

                echo "Found repositories to mirror:"
                echo "$REPOS"
                echo ""

                # Create Redis Stream consumer group if it doesn't exist
                echo "Ensuring Redis Stream consumer group exists..."
                redis-cli -h redis-master.%(namespace)s.svc.cluster.local \
                  XGROUP CREATE raibid:jobs raibid-agents $ MKSTREAM 2>/dev/null || \
                  echo "  ℹ Consumer group 'raibid-agents' already exists"

                # Create mirrors in Gitea and dispatch build jobs
                echo "Creating mirrors in Gitea and dispatching build jobs..."
                MIRROR_COUNT=0
                BUILD_JOB_COUNT=0
                for CLONE_URL in $REPOS; do
                  REPO_NAME=$(echo $CLONE_URL | sed 's|.*/||' | sed 's|.git$||')
                  echo "- Mirroring: $REPO_NAME"

                  # Create mirror using Gitea Migration API (requires API token, not basic auth)
                  RESPONSE=$(curl -w "\n%%{http_code}" -X POST \
                    "http://gitea-http.%(namespace)s.svc.cluster.local:3000/api/v1/repos/migrate" \
                    -H "Authorization: token $GITEA_TOKEN" \
                    -H "Content-Type: application/json" \
                    -d "{
                      \"clone_addr\": \"$CLONE_URL\",
                      \"auth_token\": \"$GITHUB_TOKEN\",
                      \"mirror\": true,
                      \"mirror_interval\": \"8h\",
                      \"private\": true,
                      \"repo_name\": \"$REPO_NAME\",
                      \"repo_owner\": \"$GITEA_ADMIN_USER\"
                    }")

                  HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
                  MIRROR_SUCCESS=false

                  if [ "$HTTP_CODE" = "201" ] || [ "$HTTP_CODE" = "200" ]; then
                    echo "  ✓ Successfully mirrored $REPO_NAME"
                    MIRROR_COUNT=$((MIRROR_COUNT + 1))
                    MIRROR_SUCCESS=true
                  elif [ "$HTTP_CODE" = "409" ]; then
                    echo "  ℹ Mirror already exists: $REPO_NAME"
                    MIRROR_SUCCESS=true
                  else
                    echo "  ⚠ Failed to mirror $REPO_NAME (HTTP $HTTP_CODE)"
                  fi

                  # Dispatch initial build job to Redis Streams
                  if [ "$MIRROR_SUCCESS" = "true" ]; then
                    echo "  → Dispatching initial build job for $REPO_NAME"
                    JOB_ID=$(cat /proc/sys/kernel/random/uuid)
                    TIMESTAMP=$(date -u +%%Y-%%m-%%dT%%H:%%M:%%SZ)

                    # Create JSON job object matching raibid_common::jobs::Job structure
                    JOB_JSON=$(cat <<-EOF
		{
		  "id": "$JOB_ID",
		  "repo": "$GITEA_ADMIN_USER/$REPO_NAME",
		  "branch": "main",
		  "commit": "",
		  "status": "pending",
		  "started_at": "$TIMESTAMP",
		  "finished_at": null,
		  "duration": null,
		  "agent_id": null,
		  "exit_code": null
		}
		EOF
                    )

                    # Push job to Redis Stream using XADD with JSON
                    redis-cli -h redis-master.%(namespace)s.svc.cluster.local \
                      XADD raibid:jobs "*" \
                      job "$JOB_JSON" \
                      > /dev/null

                    if [ $? -eq 0 ]; then
                      echo "  ✓ Build job dispatched (ID: $JOB_ID)"
                      BUILD_JOB_COUNT=$((BUILD_JOB_COUNT + 1))
                    else
                      echo "  ⚠ Failed to dispatch build job"
                    fi
                  fi
                done

                echo ""
                echo "✓ Repository mirroring setup complete"
                echo "  Mirrored $MIRROR_COUNT repositories"
                echo "  Dispatched $BUILD_JOB_COUNT build jobs"
                echo "  Repositories available at: http://gitea-http.%(namespace)s.svc.cluster.local:3000/$GITEA_ADMIN_USER"
                echo "  Build agents will auto-scale to handle queued jobs"
              ||| % { namespace: namespace }],
              env: [
                {
                  name: 'GITHUB_TOKEN',
                  valueFrom: {
                    secretKeyRef: {
                      name: 'raibid-credentials',
                      key: 'github-token',
                      optional: true,
                    },
                  },
                },
                {
                  name: 'GITEA_ADMIN_USER',
                  valueFrom: {
                    secretKeyRef: {
                      name: 'raibid-credentials',
                      key: 'gitea-admin-user',
                      optional: true,
                    },
                  },
                },
                {
                  name: 'GITEA_ADMIN_PASSWORD',
                  valueFrom: {
                    secretKeyRef: {
                      name: 'raibid-credentials',
                      key: 'gitea-admin-password',
                      optional: true,
                    },
                  },
                },
              ],
              volumeMounts: [
                {
                  name: 'token-volume',
                  mountPath: '/token',
                  readOnly: true,
                },
              ],
              resources: {
                requests: {
                  cpu: '100m',
                  memory: '128Mi',
                },
                limits: {
                  cpu: '500m',
                  memory: '256Mi',
                },
              },
            },
          ],
        },
      },
    },
  },
}
