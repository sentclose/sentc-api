CREATE TABLE `sentc_user_device_challenge` (
	`challenge` varchar(100) NOT NULL,
	`device_id` varchar(36) NOT NULL,
	`app_id` varchar(36) NOT NULL,
	`time` bigint(20) NOT NULL,
	PRIMARY KEY (`challenge`,`device_id`,`app_id`,`time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `user_delete_challenge` AFTER DELETE ON `sentc_user_device` FOR EACH ROW DELETE FROM sentc_user_device_challenge WHERE device_id = OLD.id ;