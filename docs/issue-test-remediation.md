# Test Remediation: Fix or Remove Commented Tests

## Priority: Medium
## Complexity: Medium

## Description

During the repository restructuring (WS-00), 11 tests were temporarily commented out to unblock PR merges. These tests need to be investigated and either:
1. Fixed and re-enabled
2. Rewritten to work in the new structure
3. Determined to be obsolete and removed

## Commented Tests Inventory

### Infrastructure Retry Tests (crates/common/src/infrastructure/retry.rs)

**1. test_retry_fatal_error_no_retry (line ~400)**
- Issue: Type annotations needed for `Result<_, InfraError>`
- Error: `error[E0282]: type annotations needed`
- Fix approach: Add explicit type annotations to closure return type

**2. test_retry_exhausted (line ~420)**
- Issue: Type annotations needed for `Result<_, InfraError>`
- Error: Same as above
- Fix approach: Add explicit type annotations

**3. test_async_retry_success (line ~445)**
- Issue: Captured variable cannot escape FnMut closure body
- Error: `error: captured variable cannot escape 'FnMut' closure body`
- Fix approach: Restructure test to avoid capturing mutable state in async closure, or use Arc<Mutex<>>

**4. test_async_poll_until_success_after_retries (line ~492)**
- Issue: Captured variable cannot escape FnMut closure body
- Error: Same as above
- Fix approach: Same as #3

### Infrastructure Rollback Tests (crates/common/src/infrastructure/rollback.rs)

**5. test_rollback_manager_commit (line ~392)**
- Issue: Borrow of moved value after `commit()` consumes manager
- Error: `error[E0382]: borrow of moved value: manager`
- Fix approach: Restructure test to not access manager after commit, or change commit() to not consume self

### K3s Platform Tests (crates/common/src/infrastructure/k3s.rs)

**6. test_platform_detection (line ~538)**
- Issue: Only supports ARM64 Linux and macOS, fails on x86_64 (GitHub Actions)
- Error: `Unsupported platform: linux x86_64`
- Fix approach: Add platform check to skip on x86_64, or add x86_64 support

**7. test_installer_creation (line ~562)**
- Issue: Same as above
- Fix approach: Same as #6

**8. test_download_binary (line ~567)**
- Issue: Same as above
- Fix approach: Same as #6

**9. test_download_checksums (line ~584)**
- Issue: Same as above
- Fix approach: Same as #6

**10. test_verify_checksum_success (line ~600)**
- Issue: Same as above
- Fix approach: Same as #6

**11. test_verify_checksum_failure (line ~626)**
- Issue: Same as above
- Fix approach: Same as #6

## Tasks

- [ ] Review each commented test and determine remediation approach
- [ ] Fix retry tests with type annotation issues (tests 1-2)
- [ ] Fix async closure capture issues (tests 3-4)
- [ ] Fix rollback manager borrow issue (test 5)
- [ ] Add platform detection/skipping for k3s tests OR add x86_64 support (tests 6-11)
- [ ] Re-enable all fixed tests
- [ ] Remove any tests determined to be obsolete
- [ ] Verify all tests pass in CI

## Acceptance Criteria

- [ ] All commented tests are either fixed and passing, or removed with justification
- [ ] No TODOs remain in test code referencing "Issue #TBD"
- [ ] CI passes with all tests enabled or properly skipped
- [ ] Test coverage is maintained or improved

## Technical Notes

### Retry/Async Tests
The async closure issues are due to Rust's borrow checker and async closure limitations. Consider using:
- `Arc<Mutex<T>>` for shared mutable state
- Restructuring tests to avoid mutable captures
- Using channels for state updates

### K3s Platform Tests
GitHub Actions runners use x86_64. Options:
1. Add `#[cfg(target_arch = "aarch64")]` to skip on x86_64
2. Add x86_64 binary support to k3s module
3. Mock the platform detection in tests

## References

- PR #78: https://github.com/raibid-labs/raibid-ci/pull/78
- PR #79: https://github.com/raibid-labs/raibid-ci/pull/79
- Test fixes commit: fd2fc6e (retry/rollback) and f9d40c6 (k3s)
