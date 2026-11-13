#!/usr/bin/env bash
# Softify Studio Master — 1-Line Linux Installer
# Installs softify terminal runner and native engine to ~/.local/bin/softify

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

mkdir -p "${INSTALL_DIR}" "${LIB_DIR}" "${CONFIG_DIR}"

REPO="Sarthak-Cyb3r/Softify-Release"
TAG="v0.1-beta"

echo -e "${BOLD}==> Detecting system architecture...${NC}"
ARCH=$(uname -m)
if [ "${ARCH}" != "x86_64" ]; then
    echo -e "${RED}Error: Currently official release builds support x86_64 Linux. Detected: ${ARCH}${NC}"
    exit 1
fi

echo -e "${BOLD}==> Installing Softify Engine & CLI (${TAG})...${NC}"

# If running within clone with prebuilt dist, install locally
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
if [ -f "${SCRIPT_DIR}/dist/linux/softify-linux-x86_64/softify-cli" ]; then
    echo -e "${CYAN}Found local prebuilt binary. Installing...${NC}"
    cp "${SCRIPT_DIR}/dist/linux/softify-linux-x86_64/softify-cli" "${INSTALL_DIR}/softify"
    cp "${SCRIPT_DIR}/dist/linux/softify-linux-x86_64/libsoftify_core.so" "${LIB_DIR}/libsoftify_core.so"
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
        # Fallback to raw GitHub dist assets
        RAW_CLI="https://raw.githubusercontent.com/${REPO}/main/dist/linux/softify-linux-x86_64/softify-cli"
        RAW_LIB="https://raw.githubusercontent.com/${REPO}/main/dist/linux/softify-linux-x86_64/libsoftify_core.so"
        echo -e "${CYAN}Fetching release binaries from primary repository mirror...${NC}"
        curl -sSL -f "${RAW_CLI}" -o "${INSTALL_DIR}/softify"
        curl -sSL -f "${RAW_LIB}" -o "${LIB_DIR}/libsoftify_core.so"
    fi
fi

chmod +x "${INSTALL_DIR}/softify"

# Check PATH
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo -e "${GOLD}Note: ${INSTALL_DIR} is not in your current PATH.${NC}"
    echo -e "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo -e "    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
fi

echo -e ""
echo -e "${GREEN}${BOLD}✓ Softify v0.1-beta installed successfully!${NC}"
echo -e ""
echo -e "Quick commands to get started:"
echo -e "  ${CYAN}softify search \"Starboy\"${NC}          Search studio master catalog"
echo -e "  ${CYAN}softify resolve <track-id>${NC}        Resolve 320 Kbps stream URL"
echo -e "  ${CYAN}softify lyrics \"Tum Hi Ho\" \"Arijit\"${NC} Fetch synchronized & Romanized lyrics"
echo -e "  ${CYAN}softify mixes${NC}                     View your daily algorithmic mixes"
echo -e "  ${CYAN}softify check-update${NC}              Check GitHub releases for updates"
echo -e ""
