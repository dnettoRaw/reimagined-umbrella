#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUN_DIR="$(mktemp -d /tmp/proexel-e2e.XXXXXX)"
LOG_FILE="$RUN_DIR/stack.log"
PASSWORD="$(openssl rand -hex 12)"
SALT="$(openssl rand -hex 16)"
HASH="$(node -e 'const {scryptSync}=require("node:crypto"); process.stdout.write(scryptSync(process.argv[1], process.argv[2], 32).toString("hex"))' "$PASSWORD" "$SALT")"

export PROEXEL_SESSION_SECRET="$(openssl rand -hex 32)"
export PROEXEL_DATA_FILE="$RUN_DIR/proexel-state-v1.json"
export PROEXEL_ATTACHMENTS_DIR="$RUN_DIR/attachments"
export PROEXEL_E2E_PASSWORD="$PASSWORD"
export PROEXEL_WEB_PORT=3010
export PROEXEL_SERVICE_URL="http://127.0.0.1:39410"
export PROEXEL_E2E_BASE_URL="http://127.0.0.1:3010"
export PROEXEL_NEXT_DIST_DIR=".next-e2e"
export PROEXEL_DEPLOYMENT_MANIFEST="$RUN_DIR/deployment.e2e.toml"
export PROEXEL_RUNTIME_SECRET_FILE="$RUN_DIR/target/runtime/runtime-security.secret"
export PROEXEL_AUTH_USERS="[{\"id\":\"e2e-admin\",\"email\":\"admin-e2e@proexel.local\",\"name\":\"Admin E2E\",\"role\":\"admin\",\"password_hash\":\"scrypt\$$SALT\$$HASH\"},{\"id\":\"e2e-chefe\",\"email\":\"chefe-e2e@proexel.local\",\"name\":\"Chefe E2E\",\"role\":\"chefe\",\"password_hash\":\"scrypt\$$SALT\$$HASH\"},{\"id\":\"e2e-compras\",\"email\":\"compras-e2e@proexel.local\",\"name\":\"Compras E2E\",\"role\":\"compras\",\"password_hash\":\"scrypt\$$SALT\$$HASH\"},{\"id\":\"e2e-tecnico\",\"email\":\"tecnico-e2e@proexel.local\",\"name\":\"Técnico E2E\",\"role\":\"tecnico\",\"password_hash\":\"scrypt\$$SALT\$$HASH\"}]"

sed 's/127.0.0.1:39400/127.0.0.1:39410/' "$ROOT_DIR/proexel/apps/service/deployment.local.toml" >"$PROEXEL_DEPLOYMENT_MANIFEST"
cp "$ROOT_DIR/proexel/apps/service/application.toml" "$RUN_DIR/application.toml"

cleanup() {
  status=$?
  if [[ -n "${stack_pid:-}" ]]; then kill "$stack_pid" 2>/dev/null || true; fi
  wait "${stack_pid:-}" 2>/dev/null || true
  if [[ $status -ne 0 ]]; then tail -120 "$LOG_FILE" >&2 || true; fi
  rm -rf "$RUN_DIR"
  exit "$status"
}
trap cleanup EXIT INT TERM

"$ROOT_DIR/proexel/scripts/dev-stack.sh" >"$LOG_FILE" 2>&1 &
stack_pid=$!

for _ in $(seq 1 120); do
  if curl --fail --silent http://127.0.0.1:3010/auth/login >/dev/null; then break; fi
  if ! kill -0 "$stack_pid" 2>/dev/null; then exit 1; fi
  sleep 1
done
curl --fail --silent http://127.0.0.1:3010/auth/login >/dev/null

cd "$ROOT_DIR/proexel/apps/web"
npx playwright test
