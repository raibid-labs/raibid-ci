# End-to-End Tests

E2E tests validate complete workflows.

## Running

```bash
# Requires external services
TEST_EXTERNAL=1 cargo test --test 'e2e_*' -- --ignored
```

## Requirements

- Docker
- k3s or kubectl
- Git
- Network access
