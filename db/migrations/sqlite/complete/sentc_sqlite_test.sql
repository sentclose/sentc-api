----
-- phpLiteAdmin database dump (https://www.phpliteadmin.org/)
-- phpLiteAdmin version: 1.9.8.2
-- Exported: 8:48pm on July 28, 2023 (UTC)
-- database file: D:\Programming\sentclose\sentc\backend\sentc-api\db\sqlite\db.sqlite3
----
BEGIN TRANSACTION;

----
-- Table structure for sentc_app_options
----
CREATE TABLE 'sentc_app_options' ('app_id' TEXT PRIMARY KEY NOT NULL, 'group_create' INTEGER, 'group_get' INTEGER, 'group_invite' INTEGER, 'group_reject_invite' INTEGER, 'group_accept_invite' INTEGER, 'group_join_req' INTEGER, 'group_accept_join_req' INTEGER, 'group_reject_join_req' INTEGER, 'group_key_rotation' INTEGER, 'group_user_delete' INTEGER, 'group_change_rank' INTEGER, 'group_delete' INTEGER, 'group_leave' INTEGER, 'user_exists' INTEGER, 'user_register' INTEGER, 'user_delete' INTEGER, 'user_update' INTEGER, 'user_change_password' INTEGER, 'user_reset_password' INTEGER, 'user_prepare_login' INTEGER, 'user_done_login' INTEGER, 'user_public_data' INTEGER, 'user_refresh' INTEGER, 'key_register' INTEGER, 'key_get' INTEGER, 'group_user_keys' INTEGER, 'group_user_update_check' INTEGER, 'group_auto_invite' INTEGER, 'group_list' INTEGER, 'file_register' INTEGER, 'file_part_upload' INTEGER, 'file_get' INTEGER, 'file_part_download' INTEGER, 'user_device_register' INTEGER, 'user_device_delete' INTEGER, 'user_device_list' INTEGER, 'group_invite_stop' INTEGER, 'user_key_update' INTEGER, 'file_delete' INTEGER, 'content' INTEGER, 'content_small' INTEGER, 'content_med' INTEGER, 'content_large' INTEGER, 'content_x_large' INTEGER);

----
-- Data dump for sentc_app_options, a total of 1 rows
----
INSERT INTO "sentc_app_options" ("app_id","group_create","group_get","group_invite","group_reject_invite","group_accept_invite","group_join_req","group_accept_join_req","group_reject_join_req","group_key_rotation","group_user_delete","group_change_rank","group_delete","group_leave","user_exists","user_register","user_delete","user_update","user_change_password","user_reset_password","user_prepare_login","user_done_login","user_public_data","user_refresh","key_register","key_get","group_user_keys","group_user_update_check","group_auto_invite","group_list","file_register","file_part_upload","file_get","file_part_download","user_device_register","user_device_delete","user_device_list","group_invite_stop","user_key_update","file_delete","content","content_small","content_med","content_large","content_x_large") VALUES ('sentc_int','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0','0');

----
-- structure for index sqlite_autoindex_sentc_app_options_1 on table sentc_app_options
----
;
COMMIT;
