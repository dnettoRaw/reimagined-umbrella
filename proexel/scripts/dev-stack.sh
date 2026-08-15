#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEPLOYMENT="${PROEXEL_DEPLOYMENT_MANIFEST:-$ROOT_DIR/proexel/apps/service/deployment.local.toml}"
SECRET="${PROEXEL_RUNTIME_SECRET_FILE:-$(dirname "$DEPLOYMENT")/target/runtime/runtime-security.secret}"
APPCORE_MANIFEST="$ROOT_DIR/core/AppCore-Runtime/Cargo.toml"
APPCORE_BIN="$ROOT_DIR/core/AppCore-Runtime/target/debug/appcore-bin"
SERVICE_BIN="$ROOT_DIR/proexel/target/debug/proexel-service"
DEV_TTL_MS="${PROEXEL_DEV_TTL_MS:-10800000}"
WEB_PORT="${PROEXEL_WEB_PORT:-3000}"
SERVICE_URL="${PROEXEL_SERVICE_URL:-http://127.0.0.1:39400}"

cargo build --quiet --manifest-path "$APPCORE_MANIFEST" -p appcore-bin --bin appcore-bin
cargo build --quiet --manifest-path "$ROOT_DIR/proexel/Cargo.toml" -p proexel-service
mkdir -p "$(dirname "$SECRET")"
"$APPCORE_BIN" security secret rotate --deployment "$DEPLOYMENT" --out "$SECRET" >/dev/null
chmod 600 "$SECRET"

commands=(
  proexel.valves.create proexel.valves.update
  proexel.valves.add_photo proexel.valves.delete_photo
  proexel.maintenance.register
  proexel.orders.create proexel.orders.change_status proexel.orders.delete
  proexel.purchasing.create_restock_request proexel.purchasing.review_restock_request
  proexel.purchasing.delete_restock_request
  proexel.stock.adjust proexel.stock.upsert_item proexel.stock.delete_item
  proexel.suppliers.create proexel.suppliers.update proexel.suppliers.delete
)
queries=(
  proexel.overview.get proexel.valves.list proexel.maintenance.list
  proexel.orders.list proexel.purchasing.list_restock_requests
  proexel.stock.list proexel.suppliers.list proexel.audit.list
  proexel.reports.get
)

tokens="{"
separator=""
for capability in "${commands[@]}"; do
  token="$("$APPCORE_BIN" token command --deployment "$DEPLOYMENT" --command "$capability" --ttl-ms "$DEV_TTL_MS")"
  tokens+="$separator\"$capability\":\"$token\""
  separator=","
done
for capability in "${queries[@]}"; do
  token="$("$APPCORE_BIN" token query --deployment "$DEPLOYMENT" --query "$capability" --ttl-ms "$DEV_TTL_MS")"
  tokens+="$separator\"$capability\":\"$token\""
  separator=","
done
tokens+="}"

export PROEXEL_SESSION_SECRET="${PROEXEL_SESSION_SECRET:-$(openssl rand -hex 32)}"
if [[ -z "${PROEXEL_AUTH_USERS:-}" ]]; then
  local_email="admin@proexel.local"
  local_password="$(openssl rand -hex 12)"
  local_salt="$(openssl rand -hex 16)"
  local_hash="$(node -e 'const {scryptSync}=require("node:crypto"); process.stdout.write(scryptSync(process.argv[1], process.argv[2], 32).toString("hex"))' "$local_password" "$local_salt")"
  export PROEXEL_AUTH_USERS="[{\"id\":\"local-admin\",\"email\":\"$local_email\",\"name\":\"Administrador local\",\"role\":\"admin\",\"password_hash\":\"scrypt\$$local_salt\$$local_hash\"}]"
  printf 'PROEXEL local login: %s / %s\n' "$local_email" "$local_password"
fi

cleanup() {
  if [[ -n "${web_pid:-}" ]]; then kill "$web_pid" 2>/dev/null || true; fi
  if [[ -n "${service_pid:-}" ]]; then kill "$service_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT INT TERM

APPCORE_APPLICATION_MANIFEST="$ROOT_DIR/proexel/apps/service/application.toml" \
APPCORE_DEPLOYMENT_MANIFEST="$DEPLOYMENT" \
"$SERVICE_BIN" &
service_pid=$!

export PROEXEL_SERVICE_URL="$SERVICE_URL"
export PROEXEL_SERVICE_TOKENS="$tokens"

cd "$ROOT_DIR/proexel/apps/web"
./node_modules/.bin/next dev --hostname 127.0.0.1 --port "$WEB_PORT" &
web_pid=$!
wait "$web_pid"
