#!/bin/bash

# Universal Build Script for Explr
# Builds for both Apple Silicon (M1/M2/M3) and Intel Macs
# Creates distribution-ready DMG files with frontend fixes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_header() {
    echo -e "${PURPLE}=================================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}=================================================${NC}"
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script must be run on macOS to build for Mac targets"
    exit 1
fi

print_header "🚀 Universal Explr Build"

# Configuration
APP_NAME="Explr"
VERSION="0.2.3"
TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")
TARGET_NAMES=("Apple Silicon (M1/M2/M3)" "Intel Mac")

# Check if targets are installed
print_step "Checking Rust targets..."
for target in "${TARGETS[@]}"; do
    if rustup target list --installed | grep -q "$target"; then
        print_success "Target $target is installed"
    else
        print_warning "Installing target $target..."
        rustup target add "$target"
    fi
done

# Step 1: Clean previous builds
print_step "Cleaning previous builds..."
rm -rf target/*/release/bundle/
rm -rf dist/
print_success "Previous builds cleaned"

# Step 2: Install dependencies
print_step "Installing Node.js dependencies..."
npm install
print_success "Node.js dependencies installed"

print_step "Installing Rust dependencies..."
cd src-tauri
cargo fetch
cd ..
print_success "Rust dependencies installed"

# Step 3: Build frontend
print_step "Building React frontend..."
npm run build
print_success "React frontend built"

# Step 4: Build for each target
declare -A BUILD_PATHS
declare -A DMG_PATHS

for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    target_name="${TARGET_NAMES[$i]}"
    
    print_header "Building for $target_name ($target)"
    
    print_step "Compiling Rust application for $target..."
    cd src-tauri
    cargo tauri build --target "$target"
    cd ..
    
    # Store paths for later use
    BUILD_PATHS["$target"]="target/$target/release/bundle"
    DMG_PATHS["$target"]="target/$target/release/bundle/dmg/${APP_NAME}_${VERSION}_$(echo $target | cut -d- -f1).dmg"
    
    print_success "Build completed for $target_name"
done

# Step 5: Apply macOS fixes for each target
print_step "Applying macOS frontend fixes for all targets..."

cd src-tauri/scripts

for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    target_name="${TARGET_NAMES[$i]}"
    
    print_status "Applying fixes for $target_name..."
    
    # Set environment variables for the post-build script
    export CARGO_CFG_TARGET_ARCH=$(echo $target | cut -d- -f1)
    export CARGO_CFG_TARGET_OS="macos"
    
    # Modify post-build script to work with specific target
    TARGET_ARCH=$(echo $target | cut -d- -f1)
    
    # Create target-specific temp script
    sed "s/aarch64-apple-darwin/$target/g" post-build.sh > "post-build-$TARGET_ARCH.sh"
    chmod +x "post-build-$TARGET_ARCH.sh"
    
    # Run the fix
    "./post-build-$TARGET_ARCH.sh"
    
    # Clean up temp script
    rm "post-build-$TARGET_ARCH.sh"
    
    print_success "Fixes applied for $target_name"
done

cd ../..

# Step 6: Create distribution directory and copy files
print_step "Creating distribution package..."

DIST_DIR="dist/file-explorer-$VERSION-macos"
mkdir -p "$DIST_DIR"

# Copy fixed DMGs
for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    target_name="${TARGET_NAMES[$i]}"
    arch=$(echo $target | cut -d- -f1)
    
    fixed_dmg="target/$target/release/bundle/dmg/${APP_NAME}_fixed_${VERSION}_${arch}.dmg"
    
    if [ -f "$fixed_dmg" ]; then
        cp "$fixed_dmg" "$DIST_DIR/${APP_NAME}-${VERSION}-${arch}.dmg"
        print_success "Copied DMG for $target_name: ${APP_NAME}-${VERSION}-${arch}.dmg"
    else
        print_warning "Fixed DMG not found for $target_name at $fixed_dmg"
    fi
done

# Step 7: Create installation guide
print_step "Creating installation guide..."

cat > "$DIST_DIR/README.md" << EOF
# File Explorer v$VERSION - macOS Distribution

## Choose the Right Version

### For Apple Silicon Macs (M1, M2, M3, M4)
**Use:** \`${APP_NAME}-${VERSION}-aarch64.dmg\`
- MacBook Air (2020 and later)
- MacBook Pro (2020 and later) 
- iMac (2021 and later)
- Mac Studio (all models)
- Mac Pro (2023 and later)
- Mac mini (2020 and later)

### For Intel Macs
**Use:** \`${APP_NAME}-${VERSION}-x86_64.dmg\`
- MacBook Air (2019 and earlier)
- MacBook Pro (2019 and earlier)
- iMac (2020 and earlier)
- iMac Pro (all models)
- Mac Pro (2019 and earlier)
- Mac mini (2018 and earlier)

## How to Check Your Mac's Architecture

1. Click the Apple menu → About This Mac
2. Look for "Processor" or "Chip":
   - If it says "Apple M1", "Apple M2", "Apple M3", etc. → Use **aarch64** version
   - If it says "Intel" → Use **x86_64** version

Or use Terminal:
\`\`\`bash
uname -m
# arm64 = Apple Silicon (use aarch64 DMG)
# x86_64 = Intel (use x86_64 DMG)
\`\`\`

## Installation Instructions

1. Download the correct DMG file for your Mac
2. Double-click the DMG file to mount it
3. Drag "file-explorer.app" to your Applications folder
4. Launch from Applications or Spotlight search

## Troubleshooting

If the app doesn't open:
1. Right-click the app → Open (to bypass Gatekeeper warnings)
2. Go to System Preferences → Security & Privacy → Allow apps downloaded from App Store and identified developers

## Support

For issues or questions, please check the project documentation.

Built on $(date)
EOF

# Step 8: Create checksums
print_step "Creating checksums..."
cd "$DIST_DIR"
shasum -a 256 *.dmg > checksums.sha256
print_success "Checksums created"
cd ../..

# Step 9: Display results
print_header "🎉 Build Complete!"

echo ""
print_success "Universal build completed successfully!"
echo ""
print_status "Distribution files created in: $DIST_DIR"
echo ""

print_status "📦 Available files:"
ls -la "$DIST_DIR"

echo ""
print_status "🏗️  Build Summary:"
for i in "${!TARGETS[@]}"; do
    target="${TARGETS[$i]}"
    target_name="${TARGET_NAMES[$i]}"
    arch=$(echo $target | cut -d- -f1)
    
    dmg_file="$DIST_DIR/${APP_NAME}-${VERSION}-${arch}.dmg"
    if [ -f "$dmg_file" ]; then
        size=$(du -h "$dmg_file" | cut -f1)
        print_success "✅ $target_name: ${APP_NAME}-${VERSION}-${arch}.dmg ($size)"
    else
        print_error "❌ $target_name: Build failed or DMG not found"
    fi
done

echo ""
print_status "📋 Next Steps:"
echo "   1. Test both DMG files on appropriate Mac architectures"
echo "   2. Distribute the correct DMG file to users based on their Mac type"
echo "   3. Include the README.md file for user guidance"
echo ""

print_status "🚀 Ready for distribution!"
echo ""
print_warning "⚠️  Important: Users must download the correct DMG for their Mac architecture"
print_warning "    Apple Silicon users: ${APP_NAME}-${VERSION}-aarch64.dmg"
print_warning "    Intel Mac users: ${APP_NAME}-${VERSION}-x86_64.dmg"

echo ""
print_header "Build process completed successfully! 🎉"