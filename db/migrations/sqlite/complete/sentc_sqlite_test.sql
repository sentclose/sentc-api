----
-- phpLiteAdmin database dump (https://www.phpliteadmin.org/)
-- phpLiteAdmin version: 1.9.8.2
-- Exported: 3:38pm on June 4, 2023 (UTC)
-- database file: D:\Programming\sentclose\sentc\backend\sentc-api\db\sqlite\db.sqlite3
----
BEGIN TRANSACTION;

----
-- Table structure for sentc_file
----
CREATE TABLE 'sentc_file' ('id' TEXT PRIMARY KEY NOT NULL, 'owner' TEXT, 'belongs_to' TEXT, 'belongs_to_type' INTEGER, 'app_id' TEXT,'encrypted_key' TEXT, 'time' TEXT, 'status' INTEGER, 'delete_at' TEXT, 'encrypted_file_name' TEXT DEFAULT NULL, 'master_key_id' TEXT, 'encrypted_key_alg' TEXT);

----
-- Data dump for sentc_file, a total of 0 rows
----

----
-- structure for index sqlite_autoindex_sentc_file_1 on table sentc_file
----
;

----
-- structure for trigger file_delete_parts on table sentc_file
----
CREATE TRIGGER 'file_delete_parts' AFTER DELETE ON "sentc_file" FOR EACH ROW BEGIN DELETE FROM sentc_file_part WHERE file_id = OLD.id; END;

----
-- structure for trigger file_session_delete on table sentc_file
----
CREATE TRIGGER 'file_session_delete' AFTER DELETE ON "sentc_file" FOR EACH ROW BEGIN DELETE FROM sentc_file_session WHERE file_id = OLD.id; END;
COMMIT;
