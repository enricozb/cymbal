CREATE TABLE file (
  id TEXT PRIMARY KEY,
  modified INTEGER NOT NULL,
  language TEXT NOT NULL
);

CREATE TABLE symbol (
  file_id REFERENCES file (id) NOT NULL,
  kind TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  content TEXT NOT NULL,

  -- leading/trailing text optionally captured by queries
  leading TEXT,
  trailing TEXT
);
