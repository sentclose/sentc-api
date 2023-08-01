CREATE TABLE `sentc_user_otp_recovery` (
    `id` VARCHAR(36) NOT NULL ,
    `user_id` VARCHAR(36) NOT NULL ,
    `token` VARCHAR(50) NOT NULL ,
    `time` BIGINT NOT NULL ,
    PRIMARY KEY (`id`),
	KEY `token` (`token`,`user_id`)
) ENGINE = InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `user_delete_otp` AFTER DELETE ON `sentc_user` FOR EACH ROW DELETE FROM sentc_user_otp_recovery WHERE user_id = OLD.id ;

ALTER TABLE `sentc_user` ADD `otp_secret` TEXT NULL AFTER `time`, ADD `otp_alg` TEXT NULL AFTER `otp_secret`;