ALTER TABLE `sentc_group_keys` ADD `public_key_sig` TEXT NULL DEFAULT NULL AFTER `signed_by_user_sign_key_alg`;

ALTER TABLE `sentc_group_keys` ADD `public_key_sig_key_id` VARCHAR(36) NULL COMMENT 'the key id which was used to create the sig' AFTER `public_key_sig`;