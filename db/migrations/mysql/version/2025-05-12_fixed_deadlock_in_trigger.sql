DROP TRIGGER IF EXISTS `group_user_delete_key_rotation_keys`;
CREATE TRIGGER `group_user_delete_key_rotation_keys`
	BEFORE DELETE
	ON `sentc_group_user`
	FOR EACH ROW DELETE
				 FROM sentc_group_user_key_rotation
				 WHERE user_id = OLD.user_id
				   AND group_id = OLD.group_id