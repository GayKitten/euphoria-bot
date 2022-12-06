-- Add up migration script here
CREATE TABLE users (
	id VARCHAR(20) PRIMARY KEY,
	username VARCHAR(32) NOT NULL,
	avatar VARCHAR(255) NOT NULL,
	access_token VARCHAR(255) NULL,
	expires_at TIMESTAMP NULL,
	refresh_token VARCHAR(255) NULL,
	regex JSON
);
