-- Add up migration script here
CREATE TABLE IF NOT EXISTS groups (
	id INTEGER NOT NULL PRIMARY KEY,
	name TEXT NOT NULL UNIQUE
);
CREATE UNIQUE INDEX name_lookup
ON groups(name);
CREATE TABLE IF NOT EXISTS group_users (
	id INTEGER NOT NULL PRIMARY KEY,
	user_id INTEGER NOT NULL,
	group_id INTEGER NOT NULL,
	FOREIGN KEY(user_id) REFERENCES users(id),
	FOREIGN KEY(group_id) REFERENCES groups(id),
	-- combination of useer_id, group_id must be unique
	CONSTRAINT unq UNIQUE (user_id, group_id)
);
-- add admin group
INSERT INTO groups (id, name) VALUES (1, 'admin');
-- add admin user to admin group
INSERT INTO group_users (user_id, group_id) VALUES (1, 1);
