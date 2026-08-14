# PROEXEL legacy behavior map

Status: confirmed discovery from `PROEXEL-main.zip`, 2026-08-13.

## Source inventory

| Source | Path | Status |
|---|---|---|
| Master rebuild prompt | `PROEXEL_REBUILD_PROMPTS/00_MASTER_PROMPT.md` | Read |
| Discovery prompt | `PROEXEL_REBUILD_PROMPTS/01_DISCOVERY_AND_BEHAVIOR_MAP.md` | Read |
| Legacy quick reference | `PROEXEL_REBUILD_PROMPTS/REFERENCE_LEGACY_LOGIC.md` | Read |
| PROEXEL old code | `PROEXEL-main.zip!/PROEXEL-main` | Read from extracted temp copy at `/tmp/proexel-legacy-src/PROEXEL-main` |
| Admin dashboard template | `admin-dashboard-templ-main.zip` and `/Users/dan/Downloads/admin-dashboard-templ-main` | Found |
| AppCore Runtime | `core/AppCore-Runtime` | Present as submodule, commit `12cfc5d3e78f4c7918b335907aab27c2205e710f` |

The legacy application is React/Vite/Supabase. Functional orchestration is concentrated
in `src/App.jsx`; Supabase access, snake/camel normalization and localStorage fallback
are concentrated in `src/lib/useSupabase.js`.

## Routes, views and modals

| Legacy view/modal | Behavior | Evidence | Target decision |
|---|---|---|---|
| Login gate | Blocks app until `getSession()` returns a non-expired local session; login calls RPC `app_login`. | `src/App.jsx:37`, `src/App.jsx:176`; `src/components/Login.jsx:25`; `src/lib/useSupabase.js:35` | PRESERVAR behavior, SUBSTITUIR auth architecture |
| Dashboard `dash` | Totals valves, OK/warn/critical, plant health %, recent maintenance in 7 days, with-kit count, top manufacturers and zone status. | `src/components/Dashboard.jsx:5` | PRESERVAR as overview queries |
| Valves `valves` | Zone selector, search by TAG/zone/kit/manufacturer, status/type filters, valve cards keyed by TAG, add valve for admin/chefe. | `src/App.jsx:112`; `src/components/Valves.jsx:6`; `src/components/Valves.jsx:46` | PRESERVAR flow; NORMALIZAR internal IDs |
| Add valve modal | Requires TAG and zone; captures manufacturer, DN, type, kit, serial, actuator, manufacturing year. | `src/components/AddValveModal.jsx:5`, `src/components/AddValveModal.jsx:18` | PRESERVAR fields and validation |
| Valve detail | Shows photo, status, kit, technical fields, last maintenance/kit; admin can edit technical fields and photo. | `src/components/ValveDetail.jsx:4`, `src/components/ValveDetail.jsx:7`, `src/components/ValveDetail.jsx:22` | PRESERVAR flow; enforce permissions outside UI |
| Maintenance guide | Five-step checklist: isolation/safety, actuator disassembly, seal removal/cleaning, new seal install, reassembly/test. | `src/components/MaintGuide.jsx:8` | PRESERVAR as guided workflow content |
| Maintenance finish | Requires technician name, type preventive/corrective, service, kit changed, notes, signature. Defaults type to preventive and kitChanged true. | `src/components/MaintFinish.jsx:7`, `src/components/MaintFinish.jsx:18` | PRESERVAR with domain command |
| Maintenance history modal | Filters records by `tag`, sorts by date descending, displays technician, service, kit change, notes, signature. | `src/components/MaintHistory.jsx:5` | PRESERVAR but key by `valve_id` |
| Agenda orders tab | Lists service orders, maps status `aberta` to open/pending, allows admin/chefe create/delete/status change. | `src/components/Agenda.jsx:13`, `src/components/Agenda.jsx:61` | PRESERVAR states, NORMALIZAR enum |
| Agenda calendar tab | Computes zone maintenance priority from critical valve ratio: all critical = critical, >50% critical = high, else low. | `src/components/Agenda.jsx:199` | PRESERVAR as planning query |
| Technician restock suggestion | Technician submits name, reference/TAG, description; creates pending restock request. | `src/components/Agenda.jsx:108` | PRESERVAR |
| Restock review | Admin/chefe approve, reject and delete pending requests in agenda; compras can view/manage requests but approval buttons are admin/chefe only. | `src/components/Agenda.jsx:151`, `src/components/Compras.jsx:380` | PRESERVAR with explicit permissions |
| Compras | Purchase cart by manufacturer, stock management, technician requests, suppliers, grouped kit demand and copyable email/order templates. | `src/components/Compras.jsx:6` | PRESERVAR workflows; replace local cart persistence as appropriate |
| Historico | Search maintenance by technician, zone, date range and type; expands record details and signature. | `src/components/HistoricoGeral.jsx:3` | PRESERVAR |
| Notifications | Lists non-OK valves, sorted critical first, with critical/warning filters and days since maintenance. | `src/components/Notifications.jsx:4` | PRESERVAR |
| Report PDF | Uses `jspdf`/`jspdf-autotable`; report types `geral`, `zona`, `valvula`; exports summary, zone status, critical valves and recent maintenance. | `src/components/ReportPDF.jsx:3` | PRESERVAR report data; presentation may change |

## PRESERVAR

- Maintenance health policy: `!ult_man => crit`, `> 180 => crit`, `> 150 => warn`, else `ok`. Evidence: `src/App.jsx:104`.
- Alert count includes every valve whose computed status is not OK. Evidence: `src/App.jsx:108`.
- Search matches `tag`, `zona`, `kit` and `marca`. Evidence: `src/App.jsx:112`.
- Creating maintenance writes a `maintenance_records` row, logs audit and, if `kitChanged` and matching stock exists, decrements exactly one stock unit. Evidence: `src/App.jsx:128`.
- Creating a valve normalizes TAG to uppercase and ensures a stock row with `quantity: 0`, `min_quantity: 1` when the kit is new. Evidence: `src/lib/useSupabase.js:73`.
- Stock quantity is clamped with `Math.max(0, ...)` on add/increment and +/- adjustments. Evidence: `src/lib/useSupabase.js:410`, `src/lib/useSupabase.js:463`.
- Maintenance records are inserted newest-first and include signature data URL. Evidence: `src/lib/useSupabase.js:191`; `src/components/SignaturePad.jsx`.
- Login hashes password client-side with SHA-256 and salt `_proexel_salt_2026`, passes both hash and plain password during migration. Evidence: `src/lib/auth.js:7`; `src/lib/useSupabase.js:35`.
- Session lasts 8 hours and login locks after 5 failed attempts for 15 minutes. Evidence: `src/lib/auth.js:17`, `src/lib/auth.js:52`.
- Offline behavior silently reads/writes localStorage caches when Supabase fails. Evidence: `src/lib/useSupabase.js` hooks and `src/lib/utils.js`.
- i18n supports `pt`, `es`, `en` persisted in `mp_lang`, with fallback to Portuguese. Evidence: `src/i18n/LangContext.jsx:5`, `src/i18n/translations.js`.

## NORMALIZAR

| Legacy inconsistency | Evidence | Target decision |
|---|---|---|
| Photos keyed by mutable TAG. | `src/App.jsx:296`; `src/lib/useSupabase.js:623` | Use immutable `valve_id`; store legacy TAG/path for migration/audit. |
| Orders carry `zona/zone`, `valveTag/valve_tag`, `observacoes/description`, `createdBy/created_by`. | `src/lib/useSupabase.js:6` | Canonical DTO names with import aliases. |
| Restock requests carry `ref/kit`, `description/reason`, `suggestedBy/suggested_by/created_by`, `createdAt/created_at`. | `src/lib/useSupabase.js:16` | Canonical `reference`, `reason`, `requested_by`, `created_at`. |
| Stock carries `ref/kit`, `minQuantity/min_quantity`, and maps `brand` to `location`. | `src/lib/useSupabase.js:26` | Split `manufacturer` and `location`; unique normalized reference. |
| Order status mixes `aberta`, `pendente`, `andamento`, `concluida`. | `src/components/Agenda.jsx:61`; `src/lib/useSupabase.js:254` | Normalize to `pending`, `in_progress`, `completed`. |
| Valve type includes `assento_simples`, `assento_duplo`, `mariposa`, `mariposa_brida`, `outro`; seed data also has `mariposa` text field. | `src/components/AddValveModal.jsx:3`; `src/data/plantData.js` | Preserve enum plus raw legacy details. |

## SUBSTITUIR

- Vite/React/SCSS app shell with Next.js admin dashboard.
- Supabase hooks and localStorage fallback with application commands/queries and versioned local-first storage.
- UI-only role checks with backend/application RBAC.
- Scattered audit calls with command pipeline audit.
- UI-calculated health/report logic with domain policies and query services.
- Client-side password hashing/session storage as primary security with proper auth/session adapter.

## DESCARTAR

- Neon/industrial visual language in `src/styles/main.scss`.
- React/Vite demo assets `src/assets/react.svg`, `src/assets/vite.svg`.
- Legacy architecture centered on `App.jsx` state and hooks.
- Static `PLANT_DATA` as runtime fallback once real import/storage exists; preserve it only as seed/import source.

## Side effects and risks

- Audit currently does not block operations and silently fails on Supabase errors. New local audit must be reliable; only secondary sinks may fail non-blocking.
- Maintenance stock debit is not transactional with maintenance insert. New `RegisterMaintenance` must make history, stock debit and audit idempotent.
- Offline local writes are not synced or conflict-aware. New local-first design needs explicit outbox/conflict policy.
- RLS scripts permit broad anonymous reads/writes on operational tables. New RBAC must be application-enforced and not rely on UI button hiding.

## Functional parity checklist

- [x] Read legacy `src/App.jsx`.
- [x] Read legacy `src/lib/useSupabase.js`.
- [x] Confirm Supabase tables, RPC and storage bucket.
- [x] Confirm legacy route/view/modal inventory.
- [x] Confirm page-level and key action-level RBAC.
- [ ] Implement domain policy tests for maintenance health.
- [ ] Implement idempotent maintenance-with-kit stock debit.
- [ ] Implement stock non-negative invariant without silent clamping.
- [ ] Implement valve photo association by immutable valve ID.
- [ ] Implement central audit event pipeline.
- [ ] Implement report query parity.
- [ ] Implement PT/EN/ES localization for primary flows.
