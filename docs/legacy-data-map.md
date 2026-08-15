# PROEXEL legacy data map - CONCLUÍDO 100%

Status: confirmed from `PROEXEL-main.zip`, 2026-08-13.

## Supabase and local cache sources

| Area | Supabase source | Local cache key | Evidence |
|---|---|---|---|
| Login | RPC `app_login(p_username, p_password_hash, p_password_plain)` over table `users` | `mp_session`, `mp_login_attempts` | `src/lib/useSupabase.js:35`; `src/lib/auth.js:14` |
| Valves | table `valves` | `mp_valves` | `src/lib/useSupabase.js:55` |
| Maintenance | table `maintenance_records` | `mp_history` | `src/lib/useSupabase.js:173` |
| Orders | table `orders` | `mp_orders` | `src/lib/useSupabase.js:234` |
| Restock | table `restock_requests` | `mp_restock_requests` | `src/lib/useSupabase.js:304` |
| Stock | table `stock` | `mp_stock` | `src/lib/useSupabase.js:382` |
| Suppliers | table `suppliers` | `mp_suppliers_db` | `src/lib/useSupabase.js:487` |
| Audit | table `audit_log` | none | `src/lib/useSupabase.js:572` |
| Valve photos | table `valve_photos`, storage bucket `valve-photos` | `mp_photos_db`, `mp_photos` | `src/lib/useSupabase.js:617`; `src/App.jsx:66` |
| Purchase cart | none | `mp_purchase_cart` | `src/components/Compras.jsx:32` |
| Language | none | `mp_lang` | `src/i18n/LangContext.jsx:5` |

## Entity maps

### `valves`

Legacy insert fields: `zona`, `tag`, `marca`, `serie`, `kit`, `assento`, `dn`,
`tipo`, `atuador`, `fabricacao`, `ult_kit`, `ult_man`.

Evidence: `src/lib/useSupabase.js:73`.

Target:

- `id`: stable internal identity.
- `tag`, `tag_normalized`.
- `zone`.
- `manufacturer`.
- `serial`, `kit_reference`, `seat`, `dn`, `valve_type`, `actuator`.
- `manufactured_at`, `last_kit_changed_at`, `last_maintenance_at`.
- Preserve raw import fields for traceability during migration.

### `maintenance_records`

Legacy insert fields: `tag`, `date`, `technician`, `type`, `service`,
`kit_changed`, `notes`, `signature`.

Evidence: `src/lib/useSupabase.js:191`.

Target:

- `id`, `valve_id`, `valve_tag_snapshot`.
- `performed_at`, `technician`.
- `maintenance_type`: `preventive` or `corrective`.
- `service`, `notes`, `signature_ref`, `kit_changed`, `kit_reference_snapshot`.
- `stock_debit_id`, `idempotency_key`, `audit_event_id`.

### `orders`

Legacy insert fields: `zone`, `valve_tag`, `description`, `priority`, `status`,
`created_by`, `tecnico`, `data_programada`.

Evidence: `src/lib/useSupabase.js:248`.

Compatibility aliases are created by `normalizeOrder`: `zona/zone`,
`valveTag/valve_tag`, `observacoes/description`, `createdBy/created_by`.

Target:

- `id`, `zone`, optional `valve_id`, `valve_tag_snapshot`.
- `description`, `priority`, `status`, `created_by`, `technician`, `scheduled_for`.
- Status aliases: `aberta` and `pendente` -> `pending`; `andamento` -> `in_progress`; `concluida` -> `completed`.

### `restock_requests`

Legacy insert fields: `kit`, `ref`, `reason`, `description`, `created_by`,
`suggested_by`, `status: pendente`.

Evidence: `src/lib/useSupabase.js:322`.

Target:

- `id`, `reference`, `reason`, `requested_by`, `status`, `created_at`, `reviewed_by`, `reviewed_at`.
- Status aliases: `pendente`, `aprovada`, `rejeitada`.

### `stock`

Legacy insert/update fields: `kit`, `quantity`, `min_quantity`, `location`.
Client normalizer exposes `ref`, `kit`, `minQuantity`, `min_quantity`, `brand`,
`location`, with `brand` and `location` falling back to each other.

Evidence: `src/lib/useSupabase.js:26`, `src/lib/useSupabase.js:410`.

Target:

- `id`, `reference`, `reference_normalized`.
- `quantity >= 0`, `minimum_quantity >= 0`.
- `manufacturer`, `location`.
- `created_at`, `updated_at`.

### `suppliers`

Legacy insert fields: `name`, `contact`, `email`, `website`, `notes`, `created_by`.
UI currently only edits `name` and `contact` in the visible form.

Evidence: `src/lib/useSupabase.js:503`; `src/components/Compras.jsx:51`.

Target:

- `id`, `name`, `contact`, `email`, `website`, `notes`, `created_by`, `created_at`, `updated_at`.

### `audit_log`

Legacy fields: `user_name`, `user_role`, `action`, `table_name`, `record_id`,
`old_values`, `new_values`, `description`, Supabase `created_at`.

Evidence: `src/lib/useSupabase.js:594`.

Target:

- `id`, `actor`, `role`, `operation`, `aggregate`, `aggregate_id`, `before`, `after`,
  `description`, `timestamp`, `trace_id`, `correlation_id`, `idempotency_key`.

### `valve_photos`

Legacy fields: `tag`, `photo_url`, `storage_path`, `updated_at`; storage path is
`${tag.replace(/\./g, '_')}.${ext}` in bucket `valve-photos`.

Evidence: `src/lib/useSupabase.js:640`, `src/lib/useSupabase.js:663`.

Target:

- `id`, `valve_id`, `legacy_tag`, `blob_ref`, `mime_type`, `created_by`, `created_at`, `updated_at`.

## Seed/static data

`src/data/plantData.js` contains static `PLANT_DATA` and `VALVE_PHOTOS`, used as fallback when Supabase returns no valves and for bundled public valve images. Preserve this as an import/seed source, not as runtime truth.

## Migration risks

- `valves.tag` is uppercased only on insert; existing legacy data may have inconsistent casing.
- `valve_photos` and `VALVE_PHOTOS` use TAG keys, so TAG edits can orphan photos.
- Maintenance history uses TAG only, so duplicate or edited TAGs will need import-time resolution.
- Stock manufacturer/location ambiguity must be reviewed during import.
- Local-only records with ids such as `local_*`, `os_*`, `rr_*`, `stk_*`, `sup_*` may exist in browser storage but not in Supabase exports.
