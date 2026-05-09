CREATE TABLE IF NOT EXISTS usage_events (
  event_id      TEXT PRIMARY KEY,
  license_key   TEXT NOT NULL,
  provider      TEXT NOT NULL,
  seconds       REAL NOT NULL,
  ts            INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_usage_license_ts ON usage_events(license_key, ts);

CREATE TABLE IF NOT EXISTS verify_log (
  license_key   TEXT NOT NULL,
  device_id     TEXT NOT NULL,
  ts            INTEGER NOT NULL,
  PRIMARY KEY (license_key, device_id, ts)
);

CREATE TABLE IF NOT EXISTS lemonsqueezy_events (
  id            TEXT PRIMARY KEY,
  license_key   TEXT,
  event_type    TEXT NOT NULL,
  amount_cents  INTEGER,
  ts            INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lemon_license_ts ON lemonsqueezy_events(license_key, ts);
