----
-- phpLiteAdmin database dump (https://www.phpliteadmin.org/)
-- phpLiteAdmin version: 1.9.8.2
-- Exported: 4:20pm on December 31, 2022 (UTC)
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
-- Data dump for sentc_app_jwt_keys, a total of 1 rows
----
INSERT INTO "sentc_app_jwt_keys" ("id","app_id","sign_key","verify_key","alg","time") VALUES ('174b531f-8814-42a2-94ab-3c17036183a5','1665eb92-4513-469f-81d8-b72a62e0134c','MIG2AgEAMBAGByqGSM49AgEGBSuBBAAiBIGeMIGbAgEBBDAhH0kMPR68V4jaSECXKgz6hEV+7iHqyOFAAv0Y6EXf7Db3T3rwuwuIfHyD41Rgy0ihZANiAARUyndUd/523UjG1Q5cChBHuntfYiQ5wRUIbONlT78ZrU6eUbncTdaWN72pLYTVIyjmpqgCtszZYKQNMw5I1V4c0mEddOe8bMSmic0egcVxmCCjgQVau8xU4bccdyrllFI=','BFTKd1R3/nbdSMbVDlwKEEe6e19iJDnBFQhs42VPvxmtTp5RudxN1pY3vakthNUjKOamqAK2zNlgpA0zDkjVXhzSYR1057xsxKaJzR6BxXGYIKOBBVq7zFThtxx3KuWUUg==','ES384','1659606752935');

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
-- Data dump for sentc_app, a total of 1 rows
----
INSERT INTO "sentc_app" ("id","customer_id","identifier","hashed_secret_token","hashed_public_token","hash_alg","time") VALUES ('1665eb92-4513-469f-81d8-b72a62e0134c','sentc_int',NULL,'cmzOt+BnyErJKsF2qNaiJ/YqsXJymnGQSdvJi5FpeOo=','b/t88y7h0zwqOXAtR/UqE4qsPL11PLFvo1e+8PNP8LU=','SHA256','1659606752935');

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
, 'type' INTEGER, 'invite' INTEGER);

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
, 'previous_group_key_id' TEXT, 'ephemeral_alg' TEXT, 'app_id' TEXT, 'encrypted_sign_key' TEXT, 'verify_key' TEXT, 'keypair_sign_alg' TEXT);

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
	time     text, 'key_upload_session_id' TEXT, 'user_type' INTEGER,
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
-- Table structure for sentc_customer
----
CREATE TABLE 'sentc_customer' ('id' TEXT PRIMARY KEY NOT NULL, 'email' TEXT, 'email_validate_sent' TEXT, 'email_validate' BOOLEAN, 'email_status' INTEGER, 'email_error_msg' TEXT, 'email_token' TEXT, 'name' TEXT, 'first_name' TEXT, 'company' TEXT);

----
-- Data dump for sentc_customer, a total of 0 rows
----

----
-- Table structure for sentc_app_options
----
CREATE TABLE "sentc_app_options" ('app_id' TEXT PRIMARY KEY NOT NULL, 'group_create' INTEGER, 'group_get' INTEGER, 'group_invite' INTEGER, 'group_reject_invite' INTEGER, 'group_accept_invite' INTEGER, 'group_join_req' INTEGER, 'group_accept_join_req' INTEGER, 'group_reject_join_req' INTEGER, 'group_key_rotation' INTEGER, 'group_user_delete' INTEGER, 'group_change_rank' INTEGER, 'group_delete' INTEGER, 'group_leave' INTEGER, 'user_exists' INTEGER, 'user_register' INTEGER, 'user_delete' INTEGER, 'user_update' INTEGER, 'user_change_password' INTEGER, 'user_reset_password' INTEGER, 'user_prepare_login' INTEGER, 'user_done_login' INTEGER, 'user_public_data' INTEGER, 'user_refresh' INTEGER, 'key_register' INTEGER, 'key_get' INTEGER, 'group_user_keys' INTEGER, 'group_user_update_check' INTEGER, 'group_auto_invite' INTEGER, 'group_list' INTEGER, 'file_register' INTEGER, 'file_part_upload' INTEGER, 'file_get' INTEGER, 'file_part_download' INTEGER, 'user_device_register' INTEGER, 'user_device_delete' INTEGER, 'user_device_list' INTEGER, 'group_invite_stop' INTEGER, 'user_key_update' INTEGER);

----
-- Data dump for sentc_app_options, a total of 1 rows
----
INSERT INTO "sentc_app_options" ("app_id","group_create","group_get","group_invite","group_reject_invite","group_accept_invite","group_join_req","group_accept_join_req","group_reject_join_req","group_key_rotation","group_user_delete","group_change_rank","group_delete","group_leave","user_exists","user_register","user_delete","user_update","user_change_password","user_reset_password","user_prepare_login","user_done_login","user_public_data","user_refresh","key_register","key_get","group_user_keys","group_user_update_check","group_auto_invite","group_list","file_register","file_part_upload","file_get","file_part_download","user_device_register","user_device_delete","user_device_list","group_invite_stop","user_key_update") VALUES ('1665eb92-4513-469f-81d8-b72a62e0134c','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0');

----
-- Table structure for sentc_sym_key_management
----
CREATE TABLE 'sentc_sym_key_management' ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'master_key_id' TEXT, 'creator_id' TEXT, 'encrypted_key' TEXT, 'master_key_alg' TEXT,'time' TEXT);

----
-- Data dump for sentc_sym_key_management, a total of 0 rows
----

----
-- Table structure for sentc_user_action_log
----
CREATE TABLE "sentc_user_action_log"
(
	user_id   TEXT,
	time      TEXT,
	action_id INTEGER,
	app_id    TEXT, 'amount' INTEGER,
	constraint sentc_user_action_log_pk
		primary key (user_id, app_id, time)
);

----
-- Data dump for sentc_user_action_log, a total of 0 rows
----

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
-- Data dump for sentc_group_user, a total of 0 rows
----

----
-- Table structure for sentc_file_session
----
CREATE TABLE 'sentc_file_session' ('id' TEXT PRIMARY KEY NOT NULL, 'file_id' TEXT, 'app_id' TEXT, 'created_at' TEXT, 'expected_size' INTEGER, 'max_chunk_size' TEXT);

----
-- Data dump for sentc_file_session, a total of 0 rows
----

----
-- Table structure for sentc_file_part
----
CREATE TABLE 'sentc_file_part' ('id' TEXT PRIMARY KEY NOT NULL, 'file_id' TEXT, 'app_id' TEXT, 'size' TEXT, 'sequence' INTEGER, 'extern' BOOLEAN);

----
-- Data dump for sentc_file_part, a total of 0 rows
----

----
-- Table structure for sentc_file_options
----
CREATE TABLE 'sentc_file_options' ('app_id' TEXT PRIMARY KEY NOT NULL, 'file_storage' INTEGER, 'storage_url' TEXT, 'auth_token' TEXT);

----
-- Data dump for sentc_file_options, a total of 1 rows
----
INSERT INTO "sentc_file_options" ("app_id","file_storage","storage_url","auth_token") VALUES ('1665eb92-4513-469f-81d8-b72a62e0134c','0',NULL,NULL);

----
-- Table structure for sentc_file
----
CREATE TABLE 'sentc_file' ('id' TEXT PRIMARY KEY NOT NULL, 'owner' TEXT, 'belongs_to' TEXT, 'belongs_to_type' INTEGER, 'app_id' TEXT, 'key_id' TEXT, 'time' TEXT, 'status' INTEGER, 'delete_at' TEXT, 'encrypted_file_name' TEXT DEFAULT NULL, 'master_key_id' TEXT);

----
-- Data dump for sentc_file, a total of 0 rows
----

----
-- Table structure for sentc_user
----
CREATE TABLE 'sentc_user' ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'time' TEXT, 'user_group_id' TEXT);

----
-- Data dump for sentc_user, a total of 0 rows
----

----
-- Table structure for sentc_user_device
----
CREATE TABLE 'sentc_user_device' ('id' TEXT PRIMARY KEY NOT NULL, 'user_id' TEXT, 'app_id' TEXT, 'device_identifier' TEXT, 'client_random_value' TEXT, 'public_key' TEXT, 'encrypted_private_key' TEXT, 'keypair_encrypt_alg' TEXT, 'encrypted_sign_key' TEXT, 'verify_key' TEXT, 'keypair_sign_alg' TEXT, 'derived_alg' TEXT, 'encrypted_master_key' TEXT, 'master_key_alg' TEXT, 'encrypted_master_key_alg' TEXT, 'hashed_auth_key' TEXT, 'time' TEXT, 'token' TEXT);

----
-- Data dump for sentc_user_device, a total of 0 rows
----

----
-- Table structure for sentc_user_token
----
CREATE TABLE sentc_user_token
(
	device_id TEXT,
	token     TEXT,
	app_id    TEXT,
	time      TEXT,
	constraint sentc_user_token_pk
		primary key (device_id, app_id, token)
);

----
-- Data dump for sentc_user_token, a total of 0 rows
----

----
-- Table structure for sentc_captcha
----
CREATE TABLE 'sentc_captcha' ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'solution' TEXT, 'time' TEXT);

----
-- Data dump for sentc_captcha, a total of 0 rows
----

----
-- Table structure for sentc_content
----
CREATE TABLE 'sentc_content' ('id' TEXT PRIMARY KEY NOT NULL, 'app_id' TEXT, 'item' TEXT, 'time' TEXT, 'belongs_to_group' TEXT, 'belongs_to_user' TEXT, 'creator' TEXT);

----
-- Data dump for sentc_content, a total of 0 rows
----

----
-- Table structure for sentc_content_category
----
CREATE TABLE 'sentc_content_category' ('id' TEXT PRIMARY KEY NOT NULL, 'name' TEXT, 'time' TEXT, 'app_id' TEXT);

----
-- Data dump for sentc_content_category, a total of 0 rows
----

----
-- Table structure for sentc_content_category_connect
----
CREATE TABLE 'sentc_content_category_connect' ('cat_id' TEXT NOT NULL, 'content_id' TEXT NOT NULL, PRIMARY KEY ('cat_id', 'content_id'));

----
-- Data dump for sentc_content_category_connect, a total of 0 rows
----

----
-- structure for index sqlite_autoindex_test_1 on table test
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
-- structure for index sqlite_autoindex_sentc_customer_1 on table sentc_customer
----
;

----
-- structure for index sqlite_autoindex_sentc_app_options_1 on table sentc_app_options
----
;

----
-- structure for index sqlite_autoindex_sentc_sym_key_management_1 on table sentc_sym_key_management
----
;

----
-- structure for index sqlite_autoindex_sentc_user_action_log_1 on table sentc_user_action_log
----
;

----
-- structure for index sqlite_autoindex_sentc_group_user_1 on table sentc_group_user
----
;

----
-- structure for index sqlite_autoindex_sentc_file_session_1 on table sentc_file_session
----
;

----
-- structure for index sqlite_autoindex_sentc_file_part_1 on table sentc_file_part
----
;

----
-- structure for index sqlite_autoindex_sentc_file_options_1 on table sentc_file_options
----
;

----
-- structure for index sqlite_autoindex_sentc_file_1 on table sentc_file
----
;

----
-- structure for index sqlite_autoindex_sentc_user_1 on table sentc_user
----
;

----
-- structure for index sqlite_autoindex_sentc_user_device_1 on table sentc_user_device
----
;

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
-- structure for index by_user on table sentc_sym_key_management
----
CREATE INDEX 'by_user' ON "sentc_sym_key_management" ("creator_id" ASC, "app_id" ASC);

----
-- structure for index app_id on table sentc_user
----
CREATE INDEX 'app_id' ON "sentc_user" ("app_id");

----
-- structure for index sqlite_autoindex_sentc_user_token_1 on table sentc_user_token
----
;

----
-- structure for index token on table sentc_user_device
----
CREATE INDEX 'token' ON "sentc_user_device" ("token");

----
-- structure for index sqlite_autoindex_sentc_captcha_1 on table sentc_captcha
----
;

----
-- structure for index sqlite_autoindex_sentc_content_1 on table sentc_content
----
;

----
-- structure for index time on table sentc_content
----
CREATE INDEX 'time' ON "sentc_content" ("time" DESC);

----
-- structure for index item on table sentc_content
----
CREATE INDEX 'item' ON "sentc_content" ("item" ASC);

----
-- structure for index sqlite_autoindex_sentc_content_category_1 on table sentc_content_category
----
;

----
-- structure for index sqlite_autoindex_sentc_content_category_connect_1 on table sentc_content_category_connect
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

----
-- structure for trigger delete_file_options on table sentc_app
----
CREATE TRIGGER 'delete_file_options' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_file_options WHERE app_id = OLD.id; END;

----
-- structure for trigger file_delete_parts on table sentc_file
----
CREATE TRIGGER 'file_delete_parts' AFTER DELETE ON "sentc_file" FOR EACH ROW BEGIN DELETE FROM sentc_file_part WHERE file_id = OLD.id; END;

----
-- structure for trigger file_session_delete on table sentc_file
----
CREATE TRIGGER 'file_session_delete' AFTER DELETE ON "sentc_file" FOR EACH ROW BEGIN DELETE FROM sentc_file_session WHERE file_id = OLD.id; END;

----
-- structure for trigger user_delete_user_device on table sentc_user
----
CREATE TRIGGER 'user_delete_user_device' AFTER DELETE ON "sentc_user" FOR EACH ROW BEGIN DELETE FROM sentc_user_device WHERE user_id = OLD.id; END;

----
-- structure for trigger user_delete_jwt_refresh on table sentc_user_device
----
CREATE TRIGGER 'user_delete_jwt_refresh' AFTER DELETE ON "sentc_user_device" FOR EACH ROW BEGIN DELETE FROM sentc_user_token WHERE device_id = OLD.id; END;

----
-- structure for trigger delete_app_content on table sentc_app
----
CREATE TRIGGER 'delete_app_content' AFTER DELETE ON "sentc_app" FOR EACH ROW BEGIN DELETE FROM sentc_content WHERE app_id = OLD.id; END;
COMMIT;
