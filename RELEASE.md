# Release Process Guide

This document outlines how to create and distribute releases for the QM Rust Agent.

## 🚀 Release Methods

### Method 1: Automated GitHub Release (Recommended)

**For tagged releases:**
1. Create and push a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. GitHub Actions will automatically:
   - Build for all platforms (Windows, macOS, Linux)
   - Create release archives
   - Generate checksums
   - Create GitHub Release with assets

**For manual releases:**
1. Go to GitHub Actions → Release workflow
2. Click "Run workflow"
3. Enter version (e.g., `v1.0.0`)
4. Click "Run workflow"

### Method 2: Manual Release with Script

```bash
# Build and package release locally
./scripts/package-release.sh v1.0.0

# Upload the files in releases/ directory to GitHub Releases manually
```

## 📦 Release Artifacts

Each release includes:

### For End Users
- **`qmc-rust-agent-{version}-installer.tar.gz`** - Unix/Linux/macOS installer
- **`qmc-rust-agent-{version}-installer.zip`** - Windows installer

### Pre-built Binaries
- **`qmc-rust-agent-linux-x86_64.tar.gz`** - Linux binary + docs
- **`qmc-rust-agent-windows-x86_64.zip`** - Windows binary + docs
- **`qmc-rust-agent-macos-x86_64.tar.gz`** - macOS binary + docs

### Source Code
- **`qmc-rust-agent-{version}-source.tar.gz`** - Complete source
- **`qmc-rust-agent-{version}-source.zip`** - Complete source

### Verification
- **`checksums.txt`** - SHA256 checksums for all files

## ✅ Pre-Release Checklist

Before creating a release:

### 1. Code Quality
- [ ] All tests pass: `cargo test`
- [ ] Code builds without warnings: `cargo build --release`
- [ ] Documentation is up to date
- [ ] CHANGELOG.md updated (if exists)

### 2. Version Preparation
- [ ] Update version in `Cargo.toml`
- [ ] Update version references in documentation
- [ ] Test installers work correctly
- [ ] Verify Claude Code integration works

### 3. Testing
- [ ] Test on different platforms (Windows, macOS, Linux)
- [ ] Test installer scripts:
  ```bash
  ./install.sh --local
  ./install.sh --global
  ```
- [ ] Test CLI functionality:
  ```bash
  cargo run -- examples
  cargo run -- minimize -i "f(A,B) = Σ(1,3)"
  ```

### 4. Documentation
- [ ] README.md has correct download links
- [ ] INSTALL.md instructions are current
- [ ] CLAUDE.md reflects current functionality

## 🎯 Release Steps

### 1. Prepare the Release
```bash
# Ensure clean working directory
git status

# Update version in Cargo.toml
sed -i 's/version = "0.1.0"/version = "1.0.0"/' Cargo.toml

# Commit version bump
git add Cargo.toml
git commit -m "Bump version to v1.0.0"

# Test the build
cargo test
cargo build --release
```

### 2. Create the Release
```bash
# Create and push tag
git tag v1.0.0
git push origin main
git push origin v1.0.0
```

### 3. Verify Release
1. Check GitHub Actions completed successfully
2. Verify all artifacts are present in the release
3. Download and test the installer package
4. Test the pre-built binaries

## 📋 Post-Release Tasks

After successful release:

### 1. Update Documentation
- [ ] Update download links in README.md (replace `your-username` with actual username)
- [ ] Update installation instructions if needed
- [ ] Create release announcement if desired

### 2. Testing
- [ ] Download release artifacts and test installation
- [ ] Verify Claude Code integration still works
- [ ] Test on different operating systems

### 3. Communication
- [ ] Announce release (if desired)
- [ ] Update any external documentation
- [ ] Notify users of new features

## 🐛 Troubleshooting

### GitHub Actions Fails
1. Check the workflow logs in GitHub Actions tab
2. Common issues:
   - Missing secrets (GITHUB_TOKEN should be automatic)
   - Build failures due to dependencies
   - Cross-compilation issues

### Manual Release Issues
1. Ensure all dependencies are installed
2. Check that `rsync`, `tar`, and `zip` are available
3. Verify file permissions on scripts

### Installation Problems
1. Test installers on clean systems
2. Verify all required files are included in archives
3. Check that binary permissions are correct

## 🔧 Customizing Releases

### Adding New Platforms
Edit `.github/workflows/release.yml` to add new targets in the build matrix:

```yaml
- os: ubuntu-latest
  target: aarch64-unknown-linux-gnu
  binary_name: qm-agent
  archive_name: qmc-rust-agent-linux-aarch64
```

### Changing Archive Contents
Modify the archive creation steps in both the GitHub workflow and the manual packaging script to include/exclude files as needed.

### Release Automation
The current setup provides full automation through GitHub Actions. For additional automation, consider:
- Auto-incrementing version numbers
- Generating changelogs from commit messages
- Automatic testing of release artifacts