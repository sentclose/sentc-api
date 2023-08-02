CREATE TABLE `sentc_user_otp_recovery` (
	`id` varchar(36) NOT NULL,
	`user_id` varchar(36) NOT NULL,
	`token` text NOT NULL,
	`time` bigint(20) NOT NULL,
	`token_hash` varchar(100) NOT NULL COMMENT 'to search the token',
	PRIMARY KEY (`id`),
	KEY `user_id` (`user_id`,`token_hash`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `user_delete_otp` AFTER DELETE ON `sentc_user` FOR EACH ROW DELETE FROM sentc_user_otp_recovery WHERE user_id = OLD.id ;

ALTER TABLE `sentc_user` ADD `otp_secret` TEXT NULL AFTER `time`, ADD `otp_alg` TEXT NULL AFTER `otp_secret`;