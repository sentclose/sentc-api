ALTER TABLE `sentc_group_keys`
	ADD `group_key_sig` TEXT NULL DEFAULT NULL AFTER `signed_by_user_sign_key_alg`;

ALTER TABLE `sentc_group_keys`
	DROP `signed_by_user_sign_key_alg`;