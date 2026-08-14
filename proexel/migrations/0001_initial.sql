-- PROEXEL canonical schema draft.
-- This migration documents the storage contract before adapter-specific SQL is finalized.

CREATE TABLE valves (
  id TEXT PRIMARY KEY,
  tag TEXT NOT NULL,
  tag_normalized TEXT NOT NULL UNIQUE,
  zone TEXT NOT NULL,
  manufacturer TEXT,
  serial_number TEXT,
  kit_reference TEXT,
  seat TEXT,
  dn TEXT,
  valve_type TEXT,
  actuator_reference TEXT,
  manufacturing_year INTEGER,
  last_kit_change_at TEXT,
  last_maintenance_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE stock_items (
  id TEXT PRIMARY KEY,
  reference TEXT NOT NULL,
  reference_normalized TEXT NOT NULL UNIQUE,
  manufacturer TEXT,
  location TEXT,
  quantity INTEGER NOT NULL CHECK (quantity >= 0),
  min_quantity INTEGER NOT NULL CHECK (min_quantity >= 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE stock_movements (
  id TEXT PRIMARY KEY,
  stock_item_id TEXT NOT NULL REFERENCES stock_items(id),
  kind TEXT NOT NULL CHECK (kind IN ('receipt', 'consumption', 'correction', 'migration')),
  delta INTEGER NOT NULL CHECK (delta <> 0),
  balance_after INTEGER NOT NULL CHECK (balance_after >= 0),
  reason TEXT NOT NULL,
  actor TEXT NOT NULL,
  idempotency_key TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);

CREATE TABLE maintenance_records (
  id TEXT PRIMARY KEY,
  valve_id TEXT NOT NULL REFERENCES valves(id),
  performed_at TEXT NOT NULL,
  technician TEXT NOT NULL,
  type TEXT NOT NULL CHECK (type IN ('preventive', 'corrective')),
  service_description TEXT NOT NULL,
  notes TEXT,
  kit_changed INTEGER NOT NULL CHECK (kit_changed IN (0, 1)),
  kit_reference_snapshot TEXT,
  signature_ref TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE service_orders (
  id TEXT PRIMARY KEY,
  zone TEXT NOT NULL,
  valve_id TEXT REFERENCES valves(id),
  description TEXT NOT NULL,
  priority TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending', 'in_progress', 'completed')),
  created_by TEXT NOT NULL,
  assigned_technician TEXT,
  scheduled_for TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE restock_requests (
  id TEXT PRIMARY KEY,
  stock_item_id TEXT REFERENCES stock_items(id),
  requested_reference TEXT NOT NULL,
  reason TEXT NOT NULL,
  requested_by TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'rejected')),
  reviewed_by TEXT,
  reviewed_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE suppliers (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  contact TEXT NOT NULL,
  email TEXT,
  website TEXT,
  notes TEXT,
  created_by TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE valve_photos (
  id TEXT PRIMARY KEY,
  valve_id TEXT NOT NULL REFERENCES valves(id),
  legacy_tag TEXT,
  storage_ref TEXT NOT NULL,
  content_hash TEXT,
  content_type TEXT,
  size_bytes INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE audit_events (
  id TEXT PRIMARY KEY,
  actor TEXT NOT NULL,
  role TEXT NOT NULL,
  operation TEXT NOT NULL,
  aggregate TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  before_json TEXT,
  after_json TEXT,
  description TEXT,
  trace_id TEXT,
  correlation_id TEXT,
  created_at TEXT NOT NULL
);
