CREATE TABLE hits (
  id SERIAL PRIMARY KEY,
  status INTEGER NOT NULL,
  hitdate TIMESTAMP NOT NULL,
  one TEXT NOT NULL,
  two TEXT NOT NULL,
  hithash BYTEA NOT NULL,
  hitlen INTEGER NOT NULL
);

CREATE INDEX status_idx ON hits (status);
CREATE INDEX hash_idx ON hits (hithash)
