CREATE TABLE tweets (
    id BIGINT PRIMARY KEY,
    hit_id INTEGER NOT NULL,
    text TEXT NOT NULL,
    status INTEGER NOT NULL,
    posted_time TIMESTAMP,
    user_id TEXT,
    user_name TEXT,
    user_image TEXT,
    user_verified BOOLEAN,
    user_followers INTEGER
);

CREATE TABLE hits (
  id SERIAL PRIMARY KEY,
  status INTEGER NOT NULL,
  hitdate TIMESTAMP NOT NULL,
  hithash BYTEA NOT NULL,
  hitlen INTEGER NOT NULL
);

CREATE INDEX status_idx ON hits (status);
CREATE INDEX hash_idx ON hits (hithash)
