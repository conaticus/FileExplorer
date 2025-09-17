# Explr - macOS Distribution Guide

## Building for Distribution

### Quick Start - Universal Build (Recommended)
```bash
# Build for both Apple Silicon and Intel Macs
./build-universal.sh

# Or using npm
npm run dist
```

This creates distribution-ready DMG files for both architectures with all macOS frontend fixes applied.

### Individual Architecture Builds
```bash
# Apple Silicon (M1/M2/M3/M4) only
npm run tauri:build:macos

# Intel Macs only  
npm run tauri:build:intel
```

## Distribution Files

After running the universal build, you'll find distribution files in:
```
dist/file-explorer-0.2.3-macos/
├── Explr-0.2.3-aarch64.dmg    # Apple Silicon (M-series)
├── Explr-0.2.3-x86_64.dmg     # Intel Macs
├── checksums.sha256           # File verification
└── README.md                  # User installation guide
```

## Architecture Guide for Users

### Apple Silicon Macs (Use aarch64 DMG)
- **MacBook Air**: 2020 and later
- **MacBook Pro**: 2020 and later (13", 14", 16")
- **iMac**: 2021 and later (24")
- **Mac Studio**: All models (2022+)
- **Mac Pro**: 2023 and later
- **Mac mini**: 2020 and later

### Intel Macs (Use x86_64 DMG)
- **MacBook Air**: 2019 and earlier
- **MacBook Pro**: 2019 and earlier (13", 15", 16")
- **iMac**: 2020 and earlier (21.5", 27")
- **iMac Pro**: All models (2017-2021)
- **Mac Pro**: 2013-2019 models
- **Mac mini**: 2018 and earlier

### How Users Can Check Their Mac Type
```bash
# Terminal command
uname -m
# arm64 = Apple Silicon → Use aarch64 DMG
# x86_64 = Intel → Use x86_64 DMG
```

Or: Apple Menu → About This Mac → Look for "Chip" (M1/M2/M3) or "Processor" (Intel)

## File Sizes and Performance

### Typical Build Sizes
- **Apple Silicon DMG**: ~15-25 MB (optimized for M-series processors)
- **Intel DMG**: ~18-30 MB (compatible with older Intel processors)

### Performance Characteristics
- **Apple Silicon**: Better performance, lower power consumption
- **Intel**: Broader compatibility with older macOS versions

## Distribution Checklist

### Before Distribution
- [ ] Run universal build: `./build-universal.sh`
- [ ] Test Apple Silicon DMG on M-series Mac
- [ ] Test Intel DMG on Intel Mac (if available)
- [ ] Verify app launches and shows frontend on both
- [ ] Check file integrity with checksums
- [ ] Include README.md for users

### Distribution Options

#### 1. GitHub Releases
```bash
# Upload both DMG files to GitHub Releases
# Include checksums.sha256 file
# Add installation instructions in release notes
```

#### 2. Direct Distribution
- Host DMG files on your website
- Provide clear download links for each architecture
- Include the user guide (README.md)

#### 3. Package Managers (Future)
```bash
# Homebrew cask (requires both architectures)
brew install --cask file-explorer
```

## Signing and Notarization (Optional)

For wider distribution without security warnings:

### 1. Code Signing (Requires Apple Developer Account)
```bash
# Sign the app bundle before creating DMG
codesign --force --deep --sign "Developer ID Application: Your Name" path/to/app.app
```

### 2. Notarization (Requires Apple Developer Account)
```bash
# Notarize the app with Apple
xcrun notarytool submit file-explorer.dmg --keychain-profile "AC_PASSWORD" --wait
```

### 3. Update Build Scripts
Modify `src-tauri/scripts/post-build.sh` to use your signing certificate:
```bash
# Replace this line:
codesign --force --deep --sign - "$APP_PATH"

# With your certificate:
codesign --force --deep --sign "Developer ID Application: Your Name" "$APP_PATH"
```

## Troubleshooting Distribution

### Common Issues

1. **"App is damaged" error**
   - Users need to right-click → Open for first launch
   - Or use signed/notarized builds

2. **Wrong architecture downloaded**
   - Provide clear download instructions
   - Consider creating an auto-detection webpage

3. **App doesn't launch**
   - Ensure users downloaded the fixed DMG (with frontend fix)
   - Check macOS version compatibility (10.13+)

### Build Issues

1. **Rust target not found**
   ```bash
   rustup target add aarch64-apple-darwin
   rustup target add x86_64-apple-darwin
   ```

2. **Build fails on cross-compilation**
   - Ensure Xcode Command Line Tools are installed
   - Some dependencies might not support cross-compilation

3. **DMG creation fails**
   - Check disk space
   - Ensure no existing mounts with same name

## Version Management

### Updating Versions
1. Update version in `package.json`
2. Update version in `src-tauri/Cargo.toml`
3. Update version in `src-tauri/tauri.conf.json`
4. Run universal build to generate new DMGs

### Release Naming Convention
- `file-explorer-[VERSION]-aarch64.dmg` - Apple Silicon
- `file-explorer-[VERSION]-x86_64.dmg` - Intel
- Example: `Explr-0.2.3-aarch64.dmg`

## Automated Distribution (CI/CD)

### GitHub Actions Example
```yaml
name: Build and Release
on:
  release:
    types: [published]

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin
      - name: Install dependencies fast
        run: npm ci --no-audit --no-fund --ignore-scripts
      - name: Build universal
        env:
          CI: "true"  # skip Finder AppleScript during DMG creation
        run: npm run dist
      - name: Upload assets
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./dist/file-explorer-0.2.3-macos/
```

## Support and Maintenance

### User Support
- Provide clear architecture detection instructions
- Include troubleshooting steps in distribution
- Consider creating a support webpage

### Maintenance
- Test new macOS versions when released
- Update Rust/Tauri dependencies regularly
- Monitor for architecture-specific issues

---

## Quick Reference

### Build Commands
```bash
./build-universal.sh        # Build for both architectures (recommended)
npm run dist               # Same as above
npm run tauri:build:macos  # Apple Silicon only
npm run tauri:build:intel  # Intel only
```

### File Locations
```bash
dist/Explr-0.2.3-macos/    # Distribution files
target/*/release/bundle/dmg/        # Individual build outputs
checksums.sha256                    # Verification hashes
```

### Architecture Detection
```bash
uname -m                   # Terminal command
system_profiler SPHardwareDataType | grep Chip  # Detailed info
```
