-- Add up migration script here
CREATE TABLE IF NOT EXISTS presentation (
	id INTEGER NOT NULL PRIMARY KEY UNIQUE,
	name TEXT NOT NULL,
	user_id INTEGER NOT NULL,
	content TEXT NOT NULL,
	FOREIGN KEY(user_id) REFERENCES users(id),
	CHECK(
		length("code") <= 32
	)
);
