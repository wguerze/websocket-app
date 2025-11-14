# Fixing CI Issues

## Issue 1: Check Formatting Failure ❌

**Problem:** The CI pipeline fails at the "Check formatting" stage.

**Solution:** Run the following command to automatically format all Rust code:

```bash
cargo fmt --all
```

Then commit the changes:
```bash
git add .
git commit -m "Fix code formatting"
git push
```

**Alternative:** Use the provided script:
```bash
./format-code.sh
```

**Why this happens:**
Rust has strict formatting rules enforced by `rustfmt`. The CI checks that all code follows these rules using `cargo fmt --all -- --check`.

---

## Issue 2: `set-output` Deprecation Warning ⚠️

**Problem:** Warning message:
```
The `set-output` command is deprecated and will be disabled soon.
Please upgrade to using Environment Files
```

**Status:** ✅ **FIXED!**

**What was changed:**
Updated `.github/workflows/ci-cd.yml` to use the modern `dtolnay/rust-toolchain` action instead of the deprecated `actions-rs/toolchain@v1`.

**Before:**
```yaml
- name: Install Rust toolchain
  uses: actions-rs/toolchain@v1
  with:
    profile: minimal
    toolchain: stable
    override: true
    components: rustfmt, clippy
```

**After:**
```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable
  with:
    components: rustfmt, clippy
```

---

## Quick Fix Summary

Run these commands to fix all issues:

```bash
# Fix formatting
cargo fmt --all

# Verify formatting
cargo fmt --all -- --check

# Commit changes
git add .
git commit -m "Fix code formatting and update CI workflow"
git push
```

---

## Preventing Future Formatting Issues

### Option 1: Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Auto-format before commit
cargo fmt --all
git add -u
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

### Option 2: IDE Integration

**VS Code:**
1. Install the "rust-analyzer" extension
2. Add to `.vscode/settings.json`:
```json
{
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

**IntelliJ/CLion:**
1. Go to: Settings → Languages & Frameworks → Rust → Rustfmt
2. Check "Run rustfmt on save"

### Option 3: Manual Check Before Push

Always run before pushing:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --bin server
```

Or use the provided script:
```bash
./format-code.sh
```

---

## Expected CI/CD Pipeline Flow

After fixes, the pipeline should:

1. ✅ **Checkout code** - Download repository
2. ✅ **Install Rust toolchain** - No deprecation warnings
3. ✅ **Cache dependencies** - Speed up builds
4. ✅ **Check formatting** - Pass with properly formatted code
5. ✅ **Run clippy** - Linting checks
6. ✅ **Build server** - Compile binary
7. ✅ **Run tests** - All 6 tests pass
8. ✅ **Build Docker image** - Create container
9. ✅ **Push to registry** - Publish to ghcr.io

---

## Troubleshooting

### "cargo: command not found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Formatting check still fails after running cargo fmt

Make sure you committed the formatted files:
```bash
git status
git add .
git commit -m "Apply rustfmt formatting"
```

### Clippy warnings

Fix warnings or allow specific ones:
```rust
#[allow(clippy::warning_name)]
```

---

## Verification Checklist

Before pushing to GitHub:

- [ ] Run `cargo fmt --all`
- [ ] Run `cargo fmt --all -- --check` (should pass)
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` (should pass)
- [ ] Run `cargo test --bin server` (all tests pass)
- [ ] Run `cargo build --bin server` (builds successfully)
- [ ] Commit and push changes
- [ ] Check GitHub Actions tab for green checkmarks

---

## Need Help?

- Check GitHub Actions logs for specific errors
- Review `.github/workflows/ci-cd.yml` for workflow configuration
- Run commands locally to reproduce issues
- Check `README.md` for general usage information
