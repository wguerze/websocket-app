# CI/CD Pipeline Documentation

This document describes the CI/CD pipeline setup for the WebSocket Test Server.

## Overview

The pipeline is implemented using GitHub Actions and consists of two main jobs:
1. **Test**: Runs tests and code quality checks
2. **Build Docker**: Builds and publishes Docker images

## Pipeline Stages

### 1. Test Job

Runs on every push and pull request to `main` and `develop` branches.

**Steps:**
- ✅ Checkout code
- ✅ Install Rust toolchain (stable)
- ✅ Cache Cargo dependencies (registry, index, and build artifacts)
- ✅ Run `cargo fmt` to check code formatting
- ✅ Run `cargo clippy` for linting
- ✅ Build the server binary
- ✅ Run all tests with `cargo test`

**Caching Strategy:**
- **Cargo Registry** (`~/.cargo/registry`): Cached based on `Cargo.lock` hash
- **Cargo Index** (`~/.cargo/git`): Cached based on `Cargo.lock` hash
- **Build Artifacts** (`target/`): Cached based on `Cargo.lock` hash

This dramatically reduces build times for subsequent runs.

### 2. Build Docker Job

Runs only after the test job succeeds.

**Steps:**
- ✅ Checkout code
- ✅ Set up Docker Buildx for advanced build features
- ✅ Log in to GitHub Container Registry (ghcr.io)
- ✅ Extract metadata for Docker tags
- ✅ Build multi-platform Docker image (linux/amd64)
- ✅ Push image to ghcr.io (only on push, not on PR)

**Docker Image Tags:**
- `latest` - Latest build from main branch
- `main-<sha>` - Build from main branch with commit SHA
- `develop-<sha>` - Build from develop branch with commit SHA
- Version tags (if using semantic versioning)

**Docker Build Cache:**
Uses GitHub Actions cache for Docker layers, significantly speeding up builds.

## Dockerfile

The Dockerfile uses a **multi-stage build** pattern:

### Stage 1: Builder
- Base: `rust:1.83-slim`
- Installs build dependencies (pkg-config, libssl-dev)
- Copies source code
- Builds release binary

### Stage 2: Runtime
- Base: `debian:bookworm-slim`
- Installs only runtime dependencies (ca-certificates)
- Creates non-root user (`appuser`)
- Copies binary from builder stage
- Runs as non-root for security

**Image Size Optimization:**
- Multi-stage build keeps final image small
- Only runtime dependencies included
- No source code or build tools in final image

## Running Locally

### Test the pipeline locally:

```bash
# Run formatting check
cargo fmt --all -- --check

# Run linting
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build --bin server --verbose

# Run tests
cargo test --bin server --verbose
```

### Build Docker image locally:

```bash
docker build -t websocket-server:test .
docker run -p 8080:8080 websocket-server:test
```

## Tests Included

The server includes several test types:

1. **Unit Tests**:
   - `test_server_config_default()` - Tests default configuration
   - `test_constants()` - Validates constant values
   - `test_server_config_custom()` - Tests custom configuration

2. **Integration Tests**:
   - `test_server_starts_and_accepts_connection()` - Tests server startup and WebSocket connection
   - `test_active_connection_counter()` - Tests connection counting logic
   - `test_connection_limit_with_semaphore()` - Tests concurrent connection limiting

## Secrets and Configuration

### Required Secrets:
- `GITHUB_TOKEN` - Automatically provided by GitHub Actions (no setup needed)

### Environment Variables:
- `RUST_LOG` - Log level (default: `info`)
- `CARGO_TERM_COLOR` - Colored output for Cargo

## Triggering the Pipeline

The pipeline runs automatically on:
- Push to `main` or `develop` branches
- Pull requests targeting `main` or `develop` branches

## Monitoring

### View pipeline status:
1. Go to the **Actions** tab in your GitHub repository
2. Click on the latest workflow run
3. View logs for each job

### Check test results:
Tests are run during the "Run tests" step of the Test job.

### View Docker images:
1. Go to your GitHub repository
2. Click on **Packages** in the right sidebar
3. View published Docker images

## Troubleshooting

### Tests Failing:
1. Check the test logs in GitHub Actions
2. Run tests locally: `cargo test --bin server`
3. Check for formatting issues: `cargo fmt --all -- --check`
4. Check for linting issues: `cargo clippy`

### Docker Build Failing:
1. Check the Docker build logs
2. Build locally: `docker build -t test .`
3. Verify Dockerfile syntax
4. Check .dockerignore doesn't exclude necessary files

### Cache Issues:
If builds are unexpectedly slow:
1. Check cache hit/miss in workflow logs
2. Consider clearing cache from GitHub Actions settings
3. Verify `Cargo.lock` is committed to repository

## Security

- Docker images run as non-root user
- Only necessary dependencies included in runtime image
- GitHub tokens are automatically managed
- Docker images scanned for vulnerabilities (can add)

## Future Enhancements

Consider adding:
- [ ] Security scanning (Trivy, Snyk)
- [ ] Code coverage reporting (cargo-tarpaulin)
- [ ] Performance benchmarking
- [ ] Deployment to staging/production
- [ ] Release automation
- [ ] Multi-architecture builds (ARM64)
