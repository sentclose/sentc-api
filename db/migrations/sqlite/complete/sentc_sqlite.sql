----
-- phpLiteAdmin database dump (https://www.phpliteadmin.org/)
-- phpLiteAdmin version: 1.9.8.2
-- Exported: 8:59am on August 25, 2022 (UTC)
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
-- Table structure for sentc_user
----
CREATE TABLE "sentc_user" ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'identifier' TEXT, 'time' TEXT);

----
-- Table structure for sentc_user_keys
----
CREATE TABLE "sentc_user_keys" ('id' TEXT PRIMARY KEY NOT NULL, 'user_id' TEXT, 'client_random_value' TEXT, 'public_key' TEXT, 'encrypted_private_key' TEXT, 'keypair_encrypt_alg' TEXT, 'encrypted_sign_key' TEXT, 'verify_key' TEXT, 'keypair_sign_alg' TEXT, 'derived_alg' TEXT, 'encrypted_master_key' TEXT, 'master_key_alg' TEXT, 'hashed_auth_key' TEXT, 'time' TEXT, encrypted_master_key_alg text);

----
-- Table structure for sentc_app_jwt_keys
----
CREATE TABLE "sentc_app_jwt_keys"
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
-- Table structure for sentc_app
----
CREATE TABLE "sentc_app" (
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
-- Table structure for sentc_customer
----
CREATE TABLE 'sentc_customer' ('id' TEXT PRIMARY KEY NOT NULL, 'email' TEXT, 'email_validate_sent' TEXT, 'email_validate' BOOLEAN, 'email_status' INTEGER, 'email_error_msg' TEXT, 'email_token' TEXT);

----
-- Table structure for sentc_app_options
----
CREATE TABLE "sentc_app_options" ('app_id' TEXT PRIMARY KEY NOT NULL, 'group_create' INTEGER, 'group_get' INTEGER, 'group_invite' INTEGER, 'group_reject_invite' INTEGER, 'group_accept_invite' INTEGER, 'group_join_req' INTEGER, 'group_accept_join_req' INTEGER, 'group_reject_join_req' INTEGER, 'group_key_rotation' INTEGER, 'group_user_delete' INTEGER, 'group_change_rank' INTEGER, 'group_delete' INTEGER, 'group_leave' INTEGER, 'user_exists' INTEGER, 'user_register' INTEGER, 'user_delete' INTEGER, 'user_update' INTEGER, 'user_change_password' INTEGER, 'user_reset_password' INTEGER, 'user_prepare_login' INTEGER, 'user_done_login' INTEGER, 'user_public_data' INTEGER, 'user_refresh' INTEGER, 'key_register' INTEGER, 'key_get' INTEGER, 'group_user_keys' INTEGER, 'group_user_update_check' INTEGER, 'group_auto_invite' INTEGER, 'group_list' INTEGER);

----
-- Table structure for sentc_user_token
----
CREATE TABLE "sentc_user_token"
(
	user_id TEXT,
	token   TEXT,
	app_id  TEXT,
	time    TEXT,
	constraint sentc_user_token_pk
		primary key (user_id, app_id, token)
);

----
-- Table structure for sentc_sym_key_management
----
CREATE TABLE 'sentc_sym_key_management' ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'master_key_id' TEXT, 'creator_id' TEXT, 'encrypted_key' TEXT, 'master_key_alg' TEXT,'time' TEXT);

----
-- Table structure for sentc_user_action_log
----
CREATE TABLE "sentc_user_action_log"
(
	user_id   TEXT,
	time      TEXT,
	action_id INTEGER,
	app_id    TEXT,
	constraint sentc_user_action_log_pk
		primary key (user_id, app_id, time)
);

----
-- Table structure for sentc_group_user
----
CREATE TABLE 'sentc_group_user' (
	user_id  text,
	group_id text,
	time     text,'rank' INTEGER, 'key_upload_session_id' TEXT,'type' INTEGER DEFAULT NULL,
	constraint sentc_group_user_pk
		primary key (user_id, group_id)
);

----
-- structure for index sqlite_autoindex_test_1 on table test
----
;

----
-- structure for index sqlite_autoindex_sentc_user_1 on table sentc_user
----
;

----
-- structure for index sqlite_autoindex_sentc_user_keys_1 on table sentc_user_keys
----
;

----
-- structure for index sqlite_autoindex_sentc_app_jwt_keys_1 on table sentc_app_jwt_keys
----
;

----
-- structure for index sqlite_autoindex_sentc_app_1 on table sentc_app
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
-- structure for index user_id on table sentc_user_keys
----
CREATE INDEX 'user_id' ON "sentc_user_keys" ("user_id");

----
-- structure for index app_id on table sentc_user
----
CREATE INDEX 'app_id' ON "sentc_user" ("app_id");

----
-- structure for index app_jwt_keys_app_id_index on table sentc_app_jwt_keys
----
CREATE INDEX app_jwt_keys_app_id_index
	on "sentc_app_jwt_keys" (app_id);

----
-- structure for index app_hashed_public_token_index on table sentc_app
----
CREATE INDEX app_hashed_public_token_index
	on "sentc_app" (hashed_public_token);

----
-- structure for index app_hashed_secret_token_index on table sentc_app
----
CREATE INDEX app_hashed_secret_token_index
	on "sentc_app" (hashed_secret_token);

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
-- structure for index sqlite_autoindex_sentc_customer_1 on table sentc_customer
----
;

----
-- structure for index sqlite_autoindex_sentc_app_options_1 on table sentc_app_options
----
;

----
-- structure for index sqlite_autoindex_sentc_user_token_1 on table sentc_user_token
----
;

----
-- structure for index sqlite_autoindex_sentc_sym_key_management_1 on table sentc_sym_key_management
----
;

----
-- structure for index by_user on table sentc_sym_key_management
----
CREATE INDEX 'by_user' ON "sentc_sym_key_management" ("creator_id" ASC, "app_id" ASC);

----
-- structure for index sqlite_autoindex_sentc_user_action_log_1 on table sentc_user_action_log
----
;

----
-- structure for index sqlite_autoindex_sentc_group_user_1 on table sentc_group_user
----
;

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
-- structure for trigger delete_app on table sentc_customer
----
CREATE TRIGGER 'delete_app' AFTER DELETE ON "sentc_customer" FOR EACH ROW BEGIN DELETE FROM "sentc_app" WHERE customer_id = OLD.id; END;

----
-- structure for trigger delete_app_jwt on table sentc_app
----
CREATE TRIGGER 'delete_app_jwt' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_app_jwt_keys WHERE app_id = OLD.id; END;

----
-- structure for trigger delete_group on table sentc_app
----
CREATE TRIGGER 'delete_group' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_group WHERE app_id = OLD.id; END;

----
-- structure for trigger delete_options on table sentc_app
----
CREATE TRIGGER 'delete_options' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_app_options WHERE app_id = OLD.id; END;

----
-- structure for trigger delete_user on table sentc_app
----
CREATE TRIGGER 'delete_user' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_user WHERE app_id = OLD.id; END;

----
-- structure for trigger user_delete_jwt_refresh on table sentc_user
----
CREATE TRIGGER 'user_delete_jwt_refresh' AFTER DELETE ON "sentc_user" FOR EACH ROW BEGIN DELETE FROM sentc_user_token WHERE user_id = OLD.id; END;

----
-- structure for trigger user_delete_user_keys on table sentc_user
----
CREATE TRIGGER 'user_delete_user_keys' AFTER DELETE ON "sentc_user" FOR EACH ROW BEGIN DELETE FROM sentc_user_keys WHERE user_id = OLD.id; END;

----
-- structure for trigger delete_keys on table sentc_app
----
CREATE TRIGGER 'delete_keys' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_sym_key_management WHERE app_id = OLD.id; END;

----
-- structure for trigger  group_user_delete_key_rotation_keys on table sentc_group_user
----
CREATE TRIGGER ' group_user_delete_key_rotation_keys' AFTER DELETE ON "sentc_group_user" FOR EACH ROW BEGIN DELETE FROM sentc_group_user_key_rotation WHERE user_id = OLD.user_id AND group_id = OLD.group_id; END;

----
-- structure for trigger group_user_delete_user_keys on table sentc_group_user
----
CREATE TRIGGER 'group_user_delete_user_keys' AFTER DELETE ON "sentc_group_user" FOR EACH ROW BEGIN DELETE FROM sentc_group_user_keys WHERE user_id = OLD.user_id AND group_id = OLD.group_id; END;
COMMIT;