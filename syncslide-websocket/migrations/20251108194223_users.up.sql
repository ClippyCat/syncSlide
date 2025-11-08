-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
	id INT NOT NULL PRIMARY KEY,
	name TEXT NOT NULL UNIQUE,
	email TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL
);
-- Add admin with password admin
INSERT INTO users (id, name, email, password) VALUES (
	1, 'admin', 'admin@example.com', '$argon2id$v=19$m=16,t=2,p=1$YmxhaGJsYWQ$SfWugE3WI6xvYWcPQnXhFQ'
);
