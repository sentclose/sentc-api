ALTER TABLE `sentc_group_user_keys`
	ADD INDEX (`group_id`);

ALTER TABLE `sentc_group_user`
	ADD INDEX (`group_id`);