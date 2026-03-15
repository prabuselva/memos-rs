# Versioning Scheme

This project follows [Semantic Versioning (SemVer)](https://semver.org/) with a Git-based versioning strategy.

## Version Format

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward-compatible)
- **PATCH**: Bug fixes (backward-compatible)

## Version Tags

Versions are tagged in Git using the format: `vX.Y.Z`

Examples:
- `v1.0.0`
- `v2.3.1`

## Version Sources

The version is determined in this priority order:

1. **Build-time environment variable**: `VERSION` environment variable
2. **Git tag**: Most recent tag matching `v*` pattern
3. **Git describe**: Fallback to `git describe --tags --always --dirty`
4. **Cargo.toml**: Package version as final fallback

## Version in Code

### Rust

The version is generated at build time in `build.rs` and available via:

```rust
use memos_rs::{VERSION, VERSION_SHORT};

println!("Version: {}", VERSION);      // e.g., "v1.0.0"
println!("Short version: {}", VERSION_SHORT);  // e.g., "1.0.0"
```

### Frontend (TypeScript)

The version is injected via Vite's `define` config:

```typescript
import { APP_VERSION, getVersion } from '@/lib/version';

console.log('Version:', APP_VERSION);  // e.g., "0.0.0"
```

## Release Process

### Manual Release

1. Update version in `Cargo.toml` and `frontend/package.json`
2. Create and push a Git tag:

```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

3. The GitHub workflow will automatically:
   - Build binaries for all platforms
   - Create a GitHub Release
   - Build and push Docker images

### CI/CD Release

The `release.yml` workflow triggers on:
- Pushing a tag matching `v*`
- Manual dispatch with version input

## Branching Strategy

- `main`: Always reflects the latest release
- `develop`: Integration branch for next release
- Feature branches: `feature/feature-name`
- Bugfix branches: `fix/issue-description`

## Version Bump Guidelines

- **Patch (0.0.X)**: Bug fixes, documentation updates
- **Minor (0.X.0)**: New features, enhancements
- **Major (X.0.0)**: Breaking changes, major refactors