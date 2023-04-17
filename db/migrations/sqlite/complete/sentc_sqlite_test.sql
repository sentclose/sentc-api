----
-- phpLiteAdmin database dump (https://www.phpliteadmin.org/)
-- phpLiteAdmin version: 1.9.8.2
-- Exported: 9:48am on April 17, 2023 (UTC)
-- database file: D:\Programming\sentclose\sentc\backend\sentc-api\db\sqlite\db.sqlite3
----
BEGIN TRANSACTION;

----
-- Table structure for sentc_group_user_key_rotation
----
CREATE TABLE sentc_group_user_key_rotation
(
	key_id                   text,
	user_id                  text,
	encrypted_ephemeral_key  text,
	encrypted_eph_key_key_id text, 'group_id' TEXT, 'error' TEXT,
	constraint sentc_group_user_key_rotation_pk
		primary key (key_id, user_id)
);

----
-- Data dump for sentc_group_user_key_rotation, a total of 0 rows
----

----
-- structure for index sqlite_autoindex_sentc_group_user_key_rotation_1 on table sentc_group_user_key_rotation
----
;
COMMIT;
