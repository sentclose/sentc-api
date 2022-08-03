----
-- phpLiteAdmin database dump (https://www.phpliteadmin.org/)
-- phpLiteAdmin version: 1.9.8.2
-- Exported: 11:07pm on August 2, 2022 (UTC)
-- database file: D:\Programming\sentclose\sentc\backend\sentc-api\db\sqlite\db.sqlite3
----
BEGIN TRANSACTION;

----
-- Table structure for test
----
CREATE TABLE "test"
(
	id   TEXT
		constraint "PRIMARY"
			primary key,
	name TEXT,
	time TEXT
);

----
-- Data dump for test, a total of 0 rows
----

----
-- Table structure for user
----
CREATE TABLE 'user' ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'identifier' TEXT, 'time' TEXT);

----
-- Data dump for user, a total of 0 rows
----

----
-- Table structure for user_keys
----
CREATE TABLE 'user_keys' ('id' TEXT PRIMARY KEY NOT NULL, 'user_id' TEXT, 'client_random_value' TEXT, 'public_key' TEXT, 'encrypted_private_key' TEXT, 'keypair_encrypt_alg' TEXT, 'encrypted_sign_key' TEXT, 'verify_key' TEXT, 'keypair_sign_alg' TEXT, 'derived_alg' TEXT, 'encrypted_master_key' TEXT, 'master_key_alg' TEXT, 'hashed_auth_key' TEXT, 'time' TEXT, encrypted_master_key_alg text);

----
-- Data dump for user_keys, a total of 0 rows
----

----
-- Table structure for app_jwt_keys
----
CREATE TABLE app_jwt_keys
(
	id         text
		constraint app_jwt_keys_pk
			primary key,
	app_id     text,
	sign_key   text,
	verify_key text,
	alg        text,
	time       text
);

----
-- Data dump for app_jwt_keys, a total of 0 rows
----

----
-- Table structure for app
----
CREATE TABLE 'app' (
	id                  TEXT
		constraint app_pk
			primary key,
	customer_id         text,'identifier' text,
	hashed_secret_token text,
	hashed_public_token text,
	hash_alg            text,
	time                text
);

----
-- Data dump for app, a total of 0 rows
----

----
-- Table structure for sentc_group
----
CREATE TABLE sentc_group
(
	id         text
		constraint sentc_group_pk
			primary key,
	app_id     text,
	parent     text,
	identifier text,
	time       text
);

----
-- Data dump for sentc_group, a total of 0 rows
----

----
-- Table structure for sentc_group_keys
----
CREATE TABLE sentc_group_keys
(
	id                             text
		constraint sentc_group_keys_pk
			primary key,
	group_id                       text,
	private_key_pair_alg           text,
	encrypted_private_key          text,
	public_key                     text,
	group_key_alg                  text,
	encrypted_ephemeral_key        text,
	encrypted_group_key_by_eph_key text,
	time                           text
, 'previous_group_key_id' TEXT, 'ephemeral_alg' TEXT, 'app_id' TEXT);

----
-- Data dump for sentc_group_keys, a total of 0 rows
----

----
-- Table structure for sentc_group_user_invites_and_join_req
----
CREATE TABLE sentc_group_user_invites_and_join_req
(
	user_id  text,
	group_id text,
	type     text,
	time     text, 'key_upload_session_id' TEXT,
	constraint sentc_group_user_invites_and_join_req_pk
		primary key (user_id, group_id)
);

----
-- Data dump for sentc_group_user_invites_and_join_req, a total of 0 rows
----

----
-- Table structure for sentc_group_user_keys
----
CREATE TABLE sentc_group_user_keys
(
	k_id                       text,
	user_id                    text,
	encrypted_group_key        text,
	encrypted_alg              text,
	encrypted_group_key_key_id text,
	time                       text, 'group_id' TEXT,
	constraint sentc_group_user_keys_pk
		primary key (k_id, user_id)
);

----
-- Data dump for sentc_group_user_keys, a total of 0 rows
----

----
-- Table structure for sentc_group_user_key_rotation
----
CREATE TABLE sentc_group_user_key_rotation
(
	key_id                   text,
	user_id                  text,
	encrypted_ephemeral_key  text,
	encrypted_eph_key_key_id text, 'group_id' TEXT,
	constraint sentc_group_user_key_rotation_pk
		primary key (key_id, user_id)
);

----
-- Data dump for sentc_group_user_key_rotation, a total of 0 rows
----

----
-- Table structure for sentc_group_user
----
CREATE TABLE 'sentc_group_user' (
	user_id  text,
	group_id text,
	time     text,'rank' INTEGER, 'key_upload_session_id' TEXT, 'type' TEXT DEFAULT NULL,
	constraint sentc_group_user_pk
		primary key (user_id, group_id)
);

----
-- Data dump for sentc_group_user, a total of 0 rows
----

----
-- structure for index sqlite_autoindex_test_1 on table test
----
;

----
-- structure for index sqlite_autoindex_user_1 on table user
----
;

----
-- structure for index sqlite_autoindex_user_keys_1 on table user_keys
----
;

----
-- structure for index sqlite_autoindex_app_jwt_keys_1 on table app_jwt_keys
----
;

----
-- structure for index sqlite_autoindex_app_1 on table app
----
;

----
-- structure for index sqlite_autoindex_sentc_group_1 on table sentc_group
----
;

----
-- structure for index sqlite_autoindex_sentc_group_keys_1 on table sentc_group_keys
----
;

----
-- structure for index sqlite_autoindex_sentc_group_user_invites_and_join_req_1 on table sentc_group_user_invites_and_join_req
----
;

----
-- structure for index sqlite_autoindex_sentc_group_user_keys_1 on table sentc_group_user_keys
----
;

----
-- structure for index sqlite_autoindex_sentc_group_user_key_rotation_1 on table sentc_group_user_key_rotation
----
;

----
-- structure for index sqlite_autoindex_sentc_group_user_1 on table sentc_group_user
----
;

----
-- structure for index user_id on table user_keys
----
CREATE INDEX 'user_id' ON "user_keys" ("user_id");

----
-- structure for index app_id on table user
----
CREATE INDEX 'app_id' ON "user" ("app_id");

----
-- structure for index app_jwt_keys_app_id_index on table app_jwt_keys
----
CREATE INDEX app_jwt_keys_app_id_index
	on app_jwt_keys (app_id);

----
-- structure for index app_hashed_public_token_index on table app
----
CREATE INDEX app_hashed_public_token_index
	on app (hashed_public_token);

----
-- structure for index app_hashed_secret_token_index on table app
----
CREATE INDEX app_hashed_secret_token_index
	on app (hashed_secret_token);

----
-- structure for index sentc_group_app_id_index on table sentc_group
----
CREATE INDEX sentc_group_app_id_index
	on sentc_group (app_id);

----
-- structure for index sentc_group_parent_index on table sentc_group
----
CREATE INDEX sentc_group_parent_index
	on sentc_group (parent);

----
-- structure for index get_group on table sentc_group_keys
----
CREATE INDEX 'get_group' ON "sentc_group_keys" ("group_id" ASC, "app_id" ASC);

----
-- structure for trigger user_delete_user_keys on table user
----
CREATE TRIGGER 'user_delete_user_keys' AFTER DELETE ON "user" FOR EACH ROW BEGIN DELETE FROM user_keys WHERE user_id = OLD.id; END;

----
-- structure for trigger  delete_app_jwt on table app
----
CREATE TRIGGER ' delete_app_jwt' AFTER DELETE ON "app" FOR EACH ROW BEGIN DELETE FROM app_jwt_keys WHERE app_id = OLD.id; END;

----
-- structure for trigger delete_user on table app
----
CREATE TRIGGER 'delete_user' AFTER DELETE ON "app" FOR EACH ROW BEGIN DELETE FROM user WHERE app_id = OLD.id; END;

----
-- structure for trigger group_delete_invites on table sentc_group
----
CREATE TRIGGER 'group_delete_invites' AFTER DELETE ON "sentc_group" FOR EACH ROW BEGIN DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = OLD.id; END;

----
-- structure for trigger group_delete_keys on table sentc_group
----
CREATE TRIGGER 'group_delete_keys' AFTER DELETE ON "sentc_group" FOR EACH ROW BEGIN DELETE FROM sentc_group_keys WHERE group_id = OLD.id; END;

----
-- structure for trigger group_delete_user on table sentc_group
----
CREATE TRIGGER 'group_delete_user' AFTER DELETE ON "sentc_group" FOR EACH ROW BEGIN DELETE FROM sentc_group_user WHERE group_id = OLD.id; END;

----
-- structure for trigger  group_user_delete_key_rotation_keys on table sentc_group_user
----
CREATE TRIGGER ' group_user_delete_key_rotation_keys' AFTER DELETE ON "sentc_group_user" FOR EACH ROW BEGIN DELETE FROM sentc_group_user_key_rotation WHERE user_id = OLD.user_id AND group_id = OLD.group_id; END;

----
-- structure for trigger group_user_delete_user_keys on table sentc_group_user
----
CREATE TRIGGER 'group_user_delete_user_keys' AFTER DELETE ON "sentc_group_user" FOR EACH ROW BEGIN DELETE FROM sentc_group_user_keys WHERE user_id = OLD.user_id AND group_id = OLD.group_id; END;
COMMIT;
