CREATE TABLE file (
  path TEXT NOT NULL PRIMARY KEY,
  modified INTEGER NOT NULL,
  language TEXT NOT NULL,

  -- this is set to TRUE once a file is fully parsed. if this is false, this
  -- file was being parsed when cymbal exited, so it should be reparsed as
  -- not all symbols may have been read
  is_fully_parsed INTEGER NOT NULL DEFAULT FALSE
);

CREATE TABLE symbol (
  file_path REFERENCES file (path) NOT NULL,
  kind TEXT NOT NULL,
  start INTEGER NOT NULL,
  end INTEGER NOT NULL,
  content TEXT NOT NULL,

  -- leading/trailing text optionally captured by queries
  leading TEXT,
  trailing TEXT
);
