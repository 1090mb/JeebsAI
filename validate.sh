#!/usr/bin/env bash
# ============================================================================
# validate.sh — JeebsAI Pre-Deploy Validation Suite
# ============================================================================
# Runs static checks + optional live smoke tests to catch breaks before deploy.
#
# Usage:
#   ./validate.sh          # Static checks only (no server needed)
#   ./validate.sh --live   # Static checks + live smoke tests (starts server)
#
# Exit code 0 = all checks pass, non-zero = failures found
# ============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

PASS=0
FAIL=0
WARN=0

pass()  { PASS=$((PASS + 1)); echo -e "  ${GREEN}✓${NC} $1"; }
fail()  { FAIL=$((FAIL + 1)); echo -e "  ${RED}✗${NC} $1"; }
warn()  { WARN=$((WARN + 1)); echo -e "  ${YELLOW}⚠${NC} $1"; }
header(){ echo -e "\n${CYAN}${BOLD}── $1 ──${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

LIVE_MODE=false
SKIP_BUILD=false
for arg in "$@"; do
    case "$arg" in
        --live) LIVE_MODE=true ;;
        --skip-build) SKIP_BUILD=true ;;
    esac
done

echo -e "${BOLD}JeebsAI Validation Suite${NC}"
echo "Project: $SCRIPT_DIR"
echo "Mode:    $([ "$LIVE_MODE" = true ] && echo 'Static + Live' || echo 'Static only')$([ "$SKIP_BUILD" = true ] && echo ' (skip build)' || echo '')"

# ============================================================================
# 1. RUST BUILD CHECK
# ============================================================================
header "Rust Compilation"

if [ "$SKIP_BUILD" = true ]; then
    pass "cargo check skipped (--skip-build)"
elif command -v cargo &>/dev/null; then
    cargo_output=$(cargo check 2>&1) || true
    if echo "$cargo_output" | grep -q '^error'; then
        fail "cargo check has errors"
        echo "$cargo_output" | grep '^error' | head -5
    else
        pass "cargo check passes"
    fi
else
    warn "cargo not installed — skipping build check"
fi

# ============================================================================
# 2. WEBUI FILE INTEGRITY
# ============================================================================
header "WebUI File Integrity"

REQUIRED_FILES=(
    "webui/auth.js"
    "webui/index.html"
    "webui/admin_dashboard.html"
    "webui/admin_blacklist.html"
    "webui/admin_whitelist.html"
    "webui/trainer_panel.html"
    "webui/evolution.html"
    "webui/logs.html"
    "webui/profile.html"
    "webui/search.html"
    "webui/status.html"
    "webui/visualize.html"
    "webui/logic_visualize.html"
)

for f in "${REQUIRED_FILES[@]}"; do
    if [ -f "$f" ]; then
        if [ -s "$f" ]; then
            pass "$f exists ($(wc -l < "$f" | tr -d ' ') lines)"
        else
            fail "$f is EMPTY"
        fi
    else
        fail "$f is MISSING"
    fi
done

# ============================================================================
# 3. HTML STRUCTURE VALIDATION
# ============================================================================
header "HTML Structure"

for f in webui/*.html; do
    fname=$(basename "$f")

    # Check for <!doctype> or <!DOCTYPE>
    if ! head -1 "$f" | grep -iq '<!doctype'; then
        fail "$fname — missing <!doctype html>"
    fi

    # Check <html> open/close
    if ! grep -q '<html' "$f"; then
        fail "$fname — missing <html> tag"
    fi
    if ! grep -q '</html>' "$f"; then
        fail "$fname — missing </html> closing tag"
    fi

    # Check <head>/<body> structure
    if ! grep -q '<head' "$f"; then
        warn "$fname — missing <head> tag"
    fi
    if ! grep -q '<body' "$f"; then
        fail "$fname — missing <body> tag"
    fi
    if ! grep -q '</body>' "$f"; then
        fail "$fname — missing </body> closing tag"
    fi

    # Check matching script open/close (proper count)
    open_script=$(grep -o '<script' "$f" | wc -l | tr -d ' ')
    close_script=$(grep -o '</script>' "$f" | wc -l | tr -d ' ')
    if [ "$open_script" != "$close_script" ]; then
        fail "$fname — mismatched script tags: $open_script open vs $close_script close"
    else
        pass "$fname — $open_script script block(s) valid"
    fi
done

# ============================================================================
# 4. AUTH.JS INTEGRATION
# ============================================================================
header "auth.js Integration"

# Verify auth.js exports required functions
for fn in safeFetch authHeaders requireAuth getAuthState jeebsGetToken jeebsSetToken jeebsClearToken logout startAuthGuard startSessionPing; do
    if grep -q "function $fn" webui/auth.js; then
        pass "auth.js exports $fn()"
    else
        fail "auth.js missing function: $fn()"
    fi
done

# Verify all HTML pages include auth.js
for f in webui/*.html; do
    fname=$(basename "$f")
    if grep -q 'src="/webui/auth.js"' "$f" || grep -q "src='/webui/auth.js'" "$f"; then
        pass "$fname includes auth.js"
    else
        fail "$fname does NOT include auth.js"
    fi
done

# Check for pages that still use raw fetch() instead of safeFetch()
for f in webui/*.html; do
    fname=$(basename "$f")
    raw_count=$(grep -c 'await fetch(' "$f" 2>/dev/null || true)
    raw_count=$(echo "$raw_count" | tr -d '[:space:]')
    if [ -n "$raw_count" ] && [ "$raw_count" -gt 0 ] 2>/dev/null; then
        warn "$fname has $raw_count raw fetch() call(s) — should use safeFetch()"
    fi
done

# Check for inline TOKEN_KEY / ROOT_ADMIN constants (should use auth.js shared constants)
for f in webui/*.html; do
    fname=$(basename "$f")
    if [ "$fname" = "index.html" ]; then continue; fi  # index.html is special (login page)
    if grep -q 'const TOKEN_KEY\b' "$f" 2>/dev/null; then
        fail "$fname has inline TOKEN_KEY — should use JEEBS_TOKEN_KEY from auth.js"
    fi
done

# ============================================================================
# 5. ADMIN PAGE GUARDS
# ============================================================================
header "Admin Page Auth Guards"

ADMIN_PAGES=("admin_dashboard.html" "admin_blacklist.html" "admin_whitelist.html" "evolution.html" "logs.html")

for page in "${ADMIN_PAGES[@]}"; do
    f="webui/$page"
    if [ ! -f "$f" ]; then continue; fi
    if grep -q 'requireAuth("admin")' "$f"; then
        pass "$page has requireAuth(\"admin\") guard"
    else
        fail "$page MISSING requireAuth(\"admin\") guard"
    fi
done

if grep -q 'requireAuth("trainer")' webui/trainer_panel.html 2>/dev/null; then
    pass "trainer_panel.html has requireAuth(\"trainer\") guard"
else
    fail "trainer_panel.html MISSING requireAuth(\"trainer\") guard"
fi

# ============================================================================
# 6. RATE LIMITER CHECK
# ============================================================================
header "Rate Limiter Config"

if [ -f "src/main.rs" ]; then
    per_sec=$(grep -oE 'per_second\([0-9]+' src/main.rs | grep -oE '[0-9]+' || echo "?")
    burst=$(grep -oE 'burst_size\([0-9]+' src/main.rs | grep -oE '[0-9]+' || echo "?")

    if [ -n "$per_sec" ] && [ "$per_sec" != "?" ]; then
        if [ "$per_sec" -ge 5 ]; then
            pass "Rate limiter: per_second($per_sec) — adequate for admin"
        else
            fail "Rate limiter: per_second($per_sec) — TOO LOW, admin will get 429s"
        fi
    fi

    if [ -n "$burst" ] && [ "$burst" != "?" ]; then
        if [ "$burst" -ge 15 ]; then
            pass "Rate limiter: burst_size($burst) — adequate for admin"
        else
            fail "Rate limiter: burst_size($burst) — TOO LOW, admin will get 429s"
        fi
    fi
fi

# ============================================================================
# 7. DATABASE / CONFIG
# ============================================================================
header "Database & Config"

if [ -f "Cargo.toml" ]; then
    pass "Cargo.toml exists"
else
    fail "Cargo.toml MISSING"
fi

# Check SQL migrations exist
migration_count=$(ls -1 *.sql 2>/dev/null | wc -l | tr -d ' ')
if [ "$migration_count" -gt 0 ]; then
    pass "$migration_count SQL migration(s) found"
else
    warn "No SQL migrations found"
fi

# ============================================================================
# 8. LIVE SMOKE TESTS (optional)
# ============================================================================
if [ "$LIVE_MODE" = true ]; then
    header "Live Smoke Tests"

    PORT=9999
    PID=""

    cleanup() {
        if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
            kill "$PID" 2>/dev/null
            wait "$PID" 2>/dev/null || true
        fi
    }
    trap cleanup EXIT

    echo "  Starting test server on port $PORT..."
    DATABASE_URL="sqlite:./jeebs.db" PORT=$PORT cargo run --release &>/dev/null &
    PID=$!

    # Wait for server to start (max 30s)
    for i in $(seq 1 30); do
        if curl -sf "http://127.0.0.1:$PORT/health" &>/dev/null; then
            break
        fi
        if ! kill -0 "$PID" 2>/dev/null; then
            fail "Server process died during startup"
            PID=""
            break
        fi
        sleep 1
    done

    if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
        # Health endpoint
        status=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$PORT/health")
        if [ "$status" = "200" ]; then
            pass "GET /health → $status"
        else
            fail "GET /health → $status (expected 200)"
        fi

        # Static files
        STATIC_PAGES=(
            "/webui/index.html"
            "/webui/auth.js"
            "/webui/admin_dashboard.html"
            "/webui/admin_blacklist.html"
            "/webui/admin_whitelist.html"
            "/webui/trainer_panel.html"
            "/webui/evolution.html"
            "/webui/logs.html"
            "/webui/profile.html"
            "/webui/search.html"
            "/webui/status.html"
            "/webui/visualize.html"
            "/webui/logic_visualize.html"
        )

        for page in "${STATIC_PAGES[@]}"; do
            status=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$PORT$page")
            if [ "$status" = "200" ]; then
                pass "GET $page → $status"
            else
                fail "GET $page → $status (expected 200)"
            fi
        done

        # API endpoints (expect 401/403 without auth, NOT 404/500)
        API_ENDPOINTS=(
            "/api/auth/status"
            "/api/admin/status"
            "/api/admin/logs?page=1&limit=1"
        )

        for endpoint in "${API_ENDPOINTS[@]}"; do
            status=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$PORT$endpoint")
            if [ "$status" = "404" ] || [ "$status" = "500" ] || [ "$status" = "502" ]; then
                fail "GET $endpoint → $status (route missing or broken)"
            else
                pass "GET $endpoint → $status (route exists)"
            fi
        done

        # Rate limit test: fire 20 requests rapidly, count 429s
        echo ""
        echo "  Rate limit burst test (20 rapid requests)..."
        rate_limited=0
        for i in $(seq 1 20); do
            s=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$PORT/health")
            [ "$s" = "429" ] && rate_limited=$((rate_limited + 1))
        done
        if [ "$rate_limited" -gt 10 ]; then
            fail "Rate limit too aggressive: $rate_limited/20 requests got 429"
        elif [ "$rate_limited" -gt 0 ]; then
            warn "Rate limit: $rate_limited/20 requests got 429"
        else
            pass "Rate limit: 0/20 requests rate-limited (good)"
        fi
    else
        fail "Server failed to start — skipping HTTP tests"
    fi
fi

# ============================================================================
# SUMMARY
# ============================================================================
echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✓ $PASS passed${NC}"
[ "$WARN" -gt 0 ] && echo -e "${YELLOW}  ⚠ $WARN warnings${NC}"
[ "$FAIL" -gt 0 ] && echo -e "${RED}  ✗ $FAIL failures${NC}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$FAIL" -gt 0 ]; then
    echo -e "\n${RED}${BOLD}VALIDATION FAILED${NC} — fix the issues above before deploying."
    exit 1
else
    echo -e "\n${GREEN}${BOLD}ALL CHECKS PASSED${NC}"
    exit 0
fi
