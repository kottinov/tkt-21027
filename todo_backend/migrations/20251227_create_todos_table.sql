CREATE TABLE IF NOT EXISTS todos (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL CHECK (length(content) > 0 AND length(content) <= 140),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
