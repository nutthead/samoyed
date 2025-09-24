# GitHub Actions Workflows Documentation

## Overview

This repository uses a modern, security-focused CI/CD pipeline designed for Rust projects with comprehensive cross-platform support and automated release management.

## Workflows

### 1. CI (`ci.yml`)

**Purpose**: High-performance continuous integration with intelligent caching and parallel execution.

**Triggers**:
- Pull requests (opened, synchronized, reopened)
- Pushes to master/main
- Merge queue requests
- Manual dispatch
- Weekly security audits (Monday 00:00 UTC)

**Features**:
- **Quick Checks**: Fast-fail on formatting and linting issues
- **Security Scanning**: Automated vulnerability detection with cargo-audit and cargo-deny
- **Cross-Platform Testing**: Linux (glibc/musl), macOS (Intel/ARM), Windows
- **Code Coverage**: Automated coverage reports with Codecov integration
- **Standard Caching**: Cargo registry and target caching
- **Merge Queue Support**: Automated status checks for GitHub merge queues

**Key Optimizations**:
- Concurrent job execution with dependency management
- Platform-specific caching strategies
- cargo-binstall for faster tool installation
- Single-threaded test execution (project requirement)
- Standardized Rust 1.90.0 toolchain

### 2. Release Pipeline (`release.yml`)

**Purpose**: Fully automated release pipeline with SLSA Level 3 attestation.

**Triggers**:
- Push to master (automated release PR creation)
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

### Automated Release Flow

1. **Development**: Developers merge PRs to master using conventional commits
2. **Release PR**: release-plz automatically creates a PR with:
   - Version bumps based on commit types
   - Generated changelog using git-cliff
   - Updated dependencies
3. **Review**: Maintainers review and merge the release PR
4. **Build & Sign**: GitHub Actions builds all platform binaries and signs them
5. **Attestation**: SLSA provenance is generated and attached
6. **Publish**: Artifacts are published to GitHub Releases and crates.io

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

Monitor workflow runs at: https://github.com/nutthead/samoyed/actions

### Performance Metrics

Each PR includes:
- Code coverage reports
- Security audit results

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

### Debug Mode

Enable debug logging:

```yaml
env:
  ACTIONS_RUNNER_DEBUG: true
  ACTIONS_STEP_DEBUG: true
```

## Best Practices

1. **Conventional Commits**: Use standard prefixes (feat, fix, docs, etc.)
2. **Security First**: All releases are signed and attested
3. **Cross-Platform**: Test on all supported platforms before release
4. **Incremental Updates**: Let release-plz manage version bumps
5. **Dependency Updates**: Review and test automated dependency PRs

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