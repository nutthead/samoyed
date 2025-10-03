# GitHub Actions Workflows Documentation

## Overview

This repository uses a modern, security-focused CI/CD pipeline designed for Rust projects with comprehensive cross-platform support and fully automated release management.

**Pipeline Components:**
- **1 CI Workflow**: Comprehensive testing, security scanning, and installer validation
- **3 Release Workflows**: Automated PR creation → Tagging → Building & Publishing
- **Full Automation**: From commit to published release with zero manual intervention

## Workflows

### 1. CI (`ci.yml`)

**Purpose**: High-performance continuous integration with intelligent caching and parallel execution.

**Triggers**:
- Pull requests (opened, synchronized, reopened)
- Pushes to master/main
- Merge queue requests
- Manual dispatch
- Weekly security audits (Monday 00:00 UTC)

**Jobs**:

1. **Quick Checks** (`quick-check`)
   - Fast-fail on formatting and linting issues
   - Runs clippy with strict warnings
   - Validates documentation builds

2. **Security Audit** (`security`)
   - Automated vulnerability detection with cargo-audit and cargo-deny
   - Uploads audit results as artifacts
   - Continues on error to not block PRs

3. **Installer Script Tests** (`installer-test`) 🆕
   - Validates `install.sh` and `uninstall.sh` with shellcheck
   - Tests platform detection on Ubuntu, Fedora (containers), and macOS (native)
   - Verifies installation and uninstallation workflows
   - Tests across multiple Linux distributions using Docker

4. **Cross-Platform Tests** (`test`)
   - Linux: x86_64 (glibc and musl)
   - macOS: ARM64 (Apple Silicon)
   - Windows: x86_64 (MSVC)
   - Runs unit tests with `--test-threads=1`
   - Executes integration tests in `tests/integration/`

5. **Code Coverage** (`coverage`)
   - Generates coverage reports with cargo-tarpaulin
   - Uploads to Codecov
   - Posts detailed coverage summary on PRs (non-fork only)

6. **CI Success** (`ci-success`)
   - Aggregates all job results
   - Required status check for merge queues
   - Fails if any required job fails

**Key Optimizations**:
- Concurrent job execution with dependency management
- Platform-specific caching strategies
- cargo-binstall for faster tool installation
- Single-threaded test execution (project requirement)
- Standardized Rust 1.90.0 toolchain

### 2. Release PR (`release-pr.yml`)

**Purpose**: Automated release pull request creation using release-plz.

**Triggers**:
- Push to master branch (after conventional commits are merged)

**Features**:
- **Automatic Version Bumping**: Analyzes commit messages (conventional commits) to determine version changes
- **Changelog Generation**: Creates comprehensive changelogs from commit history
- **Dependency Updates**: Reviews and suggests dependency updates
- **Release PR**: Opens or updates a pull request with all release changes

**Workflow**:
1. Monitors commits to master for changes requiring a release
2. Analyzes conventional commit messages (feat:, fix:, etc.)
3. Determines appropriate version bump (major, minor, patch)
4. Updates `Cargo.toml` with new version
5. Generates changelog entries
6. Opens/updates a release PR for maintainer review

### 3. Release Tag (`release-plz.yml`)

**Purpose**: Automated git tag creation when release PRs are merged.

**Triggers**:
- Push to master branch (specifically when release PR is merged)

**Features**:
- **Version Change Detection**: Monitors `Cargo.toml` for version changes
- **Automatic Tagging**: Creates annotated git tags (e.g., `v0.2.1`)
- **Selective Execution**: Only runs when version actually changes
- **Bot Prevention**: Skips when commits are from github-actions[bot]

**Workflow**:
1. Detects when `Cargo.toml` is modified on master
2. Compares previous version with current version
3. If version changed, creates annotated tag `v{version}`
4. Tag push triggers the Release Pipeline workflow

### 4. Release Pipeline (`release.yml`)

**Purpose**: Build, sign, and publish releases with SLSA Level 3 attestation.

**Triggers**:
- Push of version tags (e.g., `v*.*.*`)
- Manual workflow dispatch with optional version override

**Features**:
- **Automated Release PRs**: release-plz creates PRs with version bumps and changelogs
- **SLSA Level 3 Compliance**: Sigstore/cosign signing with provenance attestation
- **Cross-Platform Builds**:
  - Linux: x86_64, aarch64, armv7 (glibc and musl variants)
  - macOS: Intel, Apple Silicon, Universal Binary
  - Windows: x86_64, ARM64
  - Experimental: RISC-V
- **Security Artifacts**: SBOM generation (CycloneDX format), vulnerability reports
- **Keyless Signing**: GitHub OIDC integration with Fulcio
- **Automated Publishing**: GitHub Releases and crates.io

**Build Matrix**:
```yaml
Platforms:
- x86_64-unknown-linux-gnu (native)
- x86_64-unknown-linux-musl (native)
- aarch64-unknown-linux-gnu (cross-compilation)
- aarch64-unknown-linux-musl (cross-compilation)
- x86_64-apple-darwin (Intel Mac)
- aarch64-apple-darwin (Apple Silicon)
- universal-apple-darwin (Universal Binary)
- x86_64-pc-windows-msvc (Windows x64)
- aarch64-pc-windows-msvc (Windows ARM64)
- armv7-unknown-linux-gnueabihf (ARM v7)
- riscv64gc-unknown-linux-gnu (RISC-V 64-bit)
```

## Security Features

### SLSA Level 3 Attestation

All release artifacts include cryptographic attestation proving:
- Build provenance (what, when, where, how)
- Non-tamperable build process
- Isolated build environments
- Signed with ephemeral keys via GitHub OIDC

### Artifact Verification

To verify a release artifact:

```bash
# Download artifact and its signature
curl -LO https://github.com/nutthead/samoyed/releases/download/v0.3.0/samoyed-0.3.0-linux-x86_64.tar.gz
curl -LO https://github.com/nutthead/samoyed/releases/download/v0.3.0/samoyed-0.3.0-linux-x86_64.tar.gz.sig
curl -LO https://github.com/nutthead/samoyed/releases/download/v0.3.0/samoyed-0.3.0-linux-x86_64.tar.gz.crt

# Verify with cosign
cosign verify-blob \
  --certificate samoyed-0.3.0-linux-x86_64.tar.gz.crt \
  --signature samoyed-0.3.0-linux-x86_64.tar.gz.sig \
  --certificate-identity-regexp "https://github.com/nutthead/samoyed" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  samoyed-0.3.0-linux-x86_64.tar.gz
```

### SBOM (Software Bill of Materials)

Each release includes SBOM files in multiple formats:
- `sbom.json`: CycloneDX JSON format
- `sbom.xml`: CycloneDX XML format

These can be used with vulnerability scanners to check for known vulnerabilities in dependencies.

## Release Process

### Automated Release Flow (Three-Workflow Pipeline)

The release process is fully automated through three coordinated workflows:

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. RELEASE PR (release-pr.yml)                                  │
│    ↓ Trigger: Push to master with conventional commits          │
│    ↓ Action: Creates/updates release PR                         │
└─────────────────────────────────────────────────────────────────┘
                            ↓
                    Maintainer reviews & merges PR
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. RELEASE TAG (release-plz.yml)                                │
│    ↓ Trigger: Version change in Cargo.toml on master            │
│    ↓ Action: Creates git tag (v0.2.1)                           │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. RELEASE PIPELINE (release.yml)                               │
│    ↓ Trigger: Version tag pushed                                │
│    ↓ Action: Build, sign, publish to GitHub + crates.io         │
└─────────────────────────────────────────────────────────────────┘
```

**Step-by-Step Process:**

1. **Development** (`main` branch)
   - Developers merge PRs using conventional commits (`feat:`, `fix:`, etc.)
   - Each commit automatically triggers CI validation

2. **Release PR Creation** (`release-pr.yml`)
   - release-plz analyzes commits since last release
   - Determines version bump (major/minor/patch) from commit types
   - Generates changelog from conventional commits
   - Opens/updates PR with version bump and changelog

3. **Maintainer Review**
   - Review generated changelog and version bump
   - Approve and merge the release PR

4. **Automatic Tagging** (`release-plz.yml`)
   - Detects `Cargo.toml` version change on master
   - Creates annotated git tag (e.g., `v0.2.1`)
   - Tag push triggers the release pipeline

5. **Build & Publish** (`release.yml`)
   - Builds binaries for all platforms
   - Generates SBOM and security attestations
   - Signs artifacts with Sigstore/cosign
   - Creates GitHub Release with assets
   - Publishes to crates.io

**Key Benefits:**
- ✅ **Zero Manual Work**: From commit to release fully automated
- ✅ **Conventional Commits**: Version determined automatically from commit messages
- ✅ **Safety**: Maintainer approval required before release
- ✅ **Transparency**: Full changelog generated from git history
- ✅ **Security**: All artifacts signed and attested

### Manual Release

For urgent releases or version overrides:

```bash
# Trigger manual release with specific version
gh workflow run release.yml \
  -f version="1.2.3" \
  -f skip_publish=false
```

## Configuration Files

### `.release-plz.toml`

Configures automated release management:
- Version bumping strategy
- Changelog generation rules
- PR templates
- git-cliff integration

### `clippy.toml`

Linting configuration with cognitive complexity limits.

### `.tarpaulin.toml`

Code coverage settings for multiple output formats.

## Required Secrets

Configure these secrets in repository settings:

- `CARGO_REGISTRY_TOKEN`: crates.io API token for publishing
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

## Environment Protection

Two protected environments are configured:

1. **release**: Required for GitHub release creation
   - Approval required from maintainers
   - Only runs on master branch

2. **crates-io**: Required for crates.io publishing
   - Approval required from maintainers
   - Deployment review timeout: 30 minutes

## Performance Optimizations

### Caching Strategy

1. **Cargo Registry Cache**: Dependencies cached per platform
2. **Standard Cargo Cache**: Using Swatinem/rust-cache for intelligent caching
3. **cargo-binstall**: Pre-built binaries for tools
4. **Target-specific caches**: Separate caches for each compilation target

### Build Optimizations

```toml
# Release profile optimizations
[profile.release]
opt-level = 3     # Optimize for speed
lto = "fat"       # Link Time Optimization (better optimization)
codegen-units = 1 # Single codegen unit (better optimization)
strip = true      # Remove debug symbols (size)
```

### Parallel Execution

- Quick checks run first to fail fast
- Platform builds run in parallel
- Security scans run independently
- Coverage analysis runs after tests

## Monitoring & Metrics

### Build Status

Monitor workflow runs at: <https://github.com/nutthead/samoyed/actions>

### Performance Metrics

Each PR includes:
- Code coverage reports with detailed per-file breakdown
- Security audit results (cargo-audit, cargo-deny)
- Installer script validation results
- Cross-platform test results (Linux, macOS, Windows)

### Release Metrics

Each release includes:
- SLSA compliance level
- Signature verification status
- SBOM availability
- Platform support matrix

## Troubleshooting

### Common Issues

1. **Test failures on Windows**:
   - All tests use single-threaded execution (--test-threads=1)
   - Integration tests use POSIX shell for cross-platform compatibility

2. **Integration test failures**:
   - Tests require single-threaded execution
   - Integration tests build release binary first

3. **Coverage report missing**:
   - Ensure tarpaulin generates all configured formats
   - Check .tarpaulin.toml configuration

4. **Security audit failures**:
   - Review cargo-audit and cargo-deny output
   - Update dependencies or add exceptions as needed

5. **Installer script failures**:
   - Check shellcheck warnings and errors
   - Verify platform detection logic for new distributions
   - Test container-based tests locally with Docker/Podman
   - Ensure scripts are executable (`chmod +x`)

### Debug Mode

Enable debug logging:

```yaml
env:
  ACTIONS_RUNNER_DEBUG: true
  ACTIONS_STEP_DEBUG: true
```

### Testing Installer Scripts Locally

Test the installer scripts before committing:

```bash
# Validate with shellcheck
shellcheck install.sh uninstall.sh

# Test on Ubuntu using Docker
docker run --rm -v "$PWD:/workspace" ubuntu:24.04 bash -c \
  "apt-get update -qq && apt-get install -y curl && cd /workspace && bash install.sh"

# Test on Fedora using Docker
docker run --rm -v "$PWD:/workspace" fedora:42 bash -c \
  "dnf install -y curl tar && cd /workspace && bash install.sh"

# Test uninstall
bash uninstall.sh
```

## Best Practices

1. **Conventional Commits**: Use standard prefixes (feat, fix, docs, etc.)
   - `feat:` - New features (bumps minor version)
   - `fix:` - Bug fixes (bumps patch version)
   - `feat!:` or `BREAKING CHANGE:` - Breaking changes (bumps major version)
   - `docs:`, `chore:`, `test:`, etc. - No version bump

2. **Security First**: All releases are signed and attested
   - SLSA Level 3 compliance
   - Sigstore/cosign for keyless signing
   - SBOM generation for dependency tracking

3. **Cross-Platform**: Test on all supported platforms before release
   - CI validates on Linux, macOS, and Windows
   - Installer scripts tested on Ubuntu, Fedora, and macOS

4. **Incremental Updates**: Let release-plz manage version bumps
   - Never manually edit version in Cargo.toml
   - Trust the automated version calculation
   - Review the generated release PR carefully

5. **Dependency Updates**: Review and test automated dependency PRs
   - Check for breaking changes in dependencies
   - Verify all tests pass with updated dependencies

6. **Installer Maintenance**: Keep installation scripts up-to-date
   - Test `install.sh` on new Linux distributions
   - Validate `uninstall.sh` cleans up completely
   - Update shellcheck suppressions sparingly

## Future Improvements

Planned enhancements:
- [ ] AI-powered changelog enrichment
- [ ] Container image builds with attestation
- [ ] Integration with package managers (Homebrew, AUR, etc.)
- [ ] Fuzz testing integration
- [ ] WASM target support

## Resources

- [release-plz Documentation](https://release-plz.ieni.dev/)
- [Sigstore Documentation](https://www.sigstore.dev/)
- [SLSA Framework](https://slsa.dev/)
- [GitHub Actions Security Hardening](https://docs.github.com/en/actions/security-guides)
- [Rust Release Best Practices](https://rust-lang.github.io/rfcs/3424-cargo-release.html)
