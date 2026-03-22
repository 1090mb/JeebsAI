#!/usr/bin/env bash
set -euo pipefail

# Safe production deploy for JeebsAI.
# Builds before restarting the service to avoid downtime from failed pulls/builds.
#
# Usage:
#   sudo ./scripts/safe_production_deploy.sh
#   sudo ./scripts/safe_production_deploy.sh main
#   sudo ./scripts/safe_production_deploy.sh <commit-or-tag> /root/JeebsAI jeebs

REF="${1:-main}"
APP_DIR="${2:-/root/JeebsAI}"
SERVICE_NAME="${3:-jeebs}"
DB_PATH="${DB_PATH:-/var/lib/jeebs/jeebs.db}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/jeebs}"
HEALTH_URL="${HEALTH_URL:-http://127.0.0.1:8080/api/health}"
WEB_URL="${WEB_URL:-http://127.0.0.1:8080/webui/index.html}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

if [[ "${EUID}" -ne 0 ]]; then
  error "Run this script with sudo or as root."
  exit 1
fi

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    error "Required command not found: $1"
    exit 1
  fi
}

need_cmd git
need_cmd cargo
need_cmd curl

if [[ ! -d "${APP_DIR}/.git" ]]; then
  error "Expected a git repo at ${APP_DIR}"
  exit 1
fi

mkdir -p "${BACKUP_DIR}"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
DB_BACKUP=""

if [[ -f "${DB_PATH}" ]]; then
  DB_BACKUP="${BACKUP_DIR}/jeebs_${TIMESTAMP}.db"
  info "Backing up database to ${DB_BACKUP}"
  cp "${DB_PATH}" "${DB_BACKUP}"
  success "Database backup created"
else
  warn "Database not found at ${DB_PATH}; skipping backup"
fi

cd "${APP_DIR}"

CURRENT_REF="$(git rev-parse --short HEAD)"
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
info "Current ref: ${CURRENT_REF} (${CURRENT_BRANCH})"

info "Fetching latest refs from origin"
git fetch origin --prune

TARGET_DESC="${REF}"
if git rev-parse --verify --quiet "origin/${REF}" >/dev/null 2>&1; then
  info "Checking out origin/${REF}"
  git checkout -B "${REF}" "origin/${REF}"
  git reset --hard "origin/${REF}"
  TARGET_DESC="origin/${REF}"
elif git rev-parse --verify --quiet "${REF}" >/dev/null 2>&1; then
  info "Checking out ${REF}"
  git checkout --detach "${REF}"
else
  error "Could not resolve deploy target '${REF}' as a branch, tag, or commit"
  exit 1
fi

NEW_REF="$(git rev-parse --short HEAD)"
info "Deploy target resolved to ${NEW_REF} (${TARGET_DESC})"

info "Removing macOS metadata files that break sqlx migrations"
find "${APP_DIR}" -type f -name '._*' -delete

info "Building release binary before touching the running service"
cargo build --release

BINARY_PATH="${APP_DIR}/target/release/jeebs"
if [[ ! -x "${BINARY_PATH}" ]]; then
  error "Build completed but binary is missing: ${BINARY_PATH}"
  exit 1
fi
success "Release build succeeded"

info "Restarting systemd service ${SERVICE_NAME}"
systemctl daemon-reload
systemctl restart "${SERVICE_NAME}"
sleep 3

if ! systemctl is-active --quiet "${SERVICE_NAME}"; then
  error "Service failed to start"
  echo
  echo "Rollback:"
  echo "  cd ${APP_DIR}"
  echo "  git checkout ${CURRENT_REF}"
  echo "  cargo build --release"
  echo "  systemctl restart ${SERVICE_NAME}"
  if [[ -n "${DB_BACKUP}" ]]; then
    echo "  cp ${DB_BACKUP} ${DB_PATH}"
  fi
  echo
  systemctl status "${SERVICE_NAME}" --no-pager -l || true
  journalctl -u "${SERVICE_NAME}" -n 100 --no-pager || true
  exit 1
fi
success "Service is running"

info "Running health checks"
if ! curl --fail --silent --show-error "${HEALTH_URL}" >/dev/null; then
  error "Health check failed: ${HEALTH_URL}"
  exit 1
fi

if ! curl --fail --silent --show-error "${WEB_URL}" >/dev/null; then
  error "Web UI check failed: ${WEB_URL}"
  exit 1
fi

success "Health checks passed"
echo
echo "Deployment complete"
echo "  Previous ref: ${CURRENT_REF}"
echo "  Current ref:  ${NEW_REF}"
if [[ -n "${DB_BACKUP}" ]]; then
  echo "  DB backup:    ${DB_BACKUP}"
fi
