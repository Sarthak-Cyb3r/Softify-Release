#!/usr/bin/env bash
# Softify Studio Master — 1-Line Standalone Linux Installer
# Installs Softify Standalone Desktop Application & CLI Engine

set -euo pipefail

BOLD='\033[1m'
CYAN='\033[0;36m'
GOLD='\033[0;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}"
cat << 'EOF'
     /\
    /  \      SOFTIFY NEXT-GEN
   / /\ \     320 Kbps Studio Master Audio Engine
  / /  \ \    Clean, Ad-Free, Hi-Res Streaming
 / /    \ \   Version v0.1-beta
 \ \    / /
  \ \  / /
   \ \/ /
    \  /
     \/
EOF
echo -e "${NC}"

INSTALL_DIR="${HOME}/.local/bin"
LIB_DIR="${HOME}/.local/lib"
CONFIG_DIR="${HOME}/.softify"
APPS_DIR="${HOME}/.local/share/applications"
ICONS_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"

mkdir -p "${INSTALL_DIR}" "${LIB_DIR}" "${CONFIG_DIR}" "${APPS_DIR}" "${ICONS_DIR}"

REPO="Sarthak-Cyb3r/Softify-Release"
TAG="v0.1-beta"

echo -e "${BOLD}==> Detecting system architecture...${NC}"
ARCH=$(uname -m)
if [ "${ARCH}" != "x86_64" ]; then
    echo -e "${RED}Error: Currently official release builds support x86_64 Linux. Detected: ${ARCH}${NC}"
    exit 1
fi

echo -e "${BOLD}==> Installing Softify Standalone Engine & Desktop App (${TAG})...${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
if [ -f "${SCRIPT_DIR}/dist/linux/softify-linux-x86_64/softify-cli" ]; then
    echo -e "${CYAN}Found local prebuilt binary. Installing...${NC}"
    cp "${SCRIPT_DIR}/dist/linux/softify-linux-x86_64/softify-cli" "${INSTALL_DIR}/softify"
    cp "${SCRIPT_DIR}/dist/linux/softify-linux-x86_64/libsoftify_core.so" "${LIB_DIR}/libsoftify_core.so"
    if [ -f "${SCRIPT_DIR}/assets/icon.png" ]; then
        cp "${SCRIPT_DIR}/assets/icon.png" "${ICONS_DIR}/softify.png"
    fi
    if [ -f "${SCRIPT_DIR}/assets/softify.desktop" ]; then
        cp "${SCRIPT_DIR}/assets/softify.desktop" "${APPS_DIR}/softify.desktop"
    fi
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/softify-linux-x86_64.tar.gz"
    echo -e "${CYAN}Downloading from ${DOWNLOAD_URL}...${NC}"
    
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "${TMP_DIR}"' EXIT

    if curl -sSL -f "${DOWNLOAD_URL}" -o "${TMP_DIR}/softify.tar.gz" 2>/dev/null; then
        tar -xzf "${TMP_DIR}/softify.tar.gz" -C "${TMP_DIR}"
        cp "${TMP_DIR}/softify-cli" "${INSTALL_DIR}/softify"
        cp "${TMP_DIR}/libsoftify_core.so" "${LIB_DIR}/libsoftify_core.so"
    else
        RAW_CLI="https://raw.githubusercontent.com/${REPO}/main/dist/linux/softify-linux-x86_64/softify-cli"
        RAW_LIB="https://raw.githubusercontent.com/${REPO}/main/dist/linux/softify-linux-x86_64/libsoftify_core.so"
        echo -e "${CYAN}Fetching release binaries from primary repository mirror...${NC}"
        curl -sSL -f "${RAW_CLI}" -o "${INSTALL_DIR}/softify"
        curl -sSL -f "${RAW_LIB}" -o "${LIB_DIR}/libsoftify_core.so"
    fi

    # Fetch app icon and desktop entry
    curl -sSL -f "https://raw.githubusercontent.com/${REPO}/main/assets/icon.png" -o "${ICONS_DIR}/softify.png" 2>/dev/null || true
    curl -sSL -f "https://raw.githubusercontent.com/${REPO}/main/assets/softify.desktop" -o "${APPS_DIR}/softify.desktop" 2>/dev/null || true
fi

chmod +x "${INSTALL_DIR}/softify"

# Register Desktop entry with system app menu
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APPS_DIR}" 2>/dev/null || true
fi

# Check PATH
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo -e "${GOLD}Note: ${INSTALL_DIR} is not in your current PATH.${NC}"
    echo -e "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo -e "    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
fi

echo -e ""
echo -e "${GREEN}${BOLD}✓ Softify standalone app installed successfully!${NC}"
echo -e ""
echo -e "Launch Softify:"
echo -e "  • ${CYAN}From Application Launcher${NC}: Search for 'Softify' in GNOME / KDE / App Menu"
echo -e "  • ${CYAN}From Terminal${NC}: Run 'softify search \"Song\"' or 'softify --gui'"
echo -e ""
