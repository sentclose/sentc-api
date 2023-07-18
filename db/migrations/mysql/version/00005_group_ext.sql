CREATE TABLE `sentc_group_ext` (
	`id` varchar(36) NOT NULL,
	`app_id` varchar(36) NOT NULL,
	`group_id` varchar(36) NOT NULL,
	`ext_name` text NOT NULL,
	`ext_data` text NOT NULL COMMENT 'the json string of each ext',
	`encrypted_key_id` varchar(36) NOT NULL COMMENT 'the key id which was used to encrypt the first password. it is from the manager group',
	`time` bigint(20) NOT NULL,
	PRIMARY KEY (`id`),
	KEY `group_id` (`group_id`,`app_id`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `group_delete_ext_keys` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_ext WHERE app_id = OLD.app_id AND group_id = OLD.id;