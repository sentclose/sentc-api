CREATE TABLE `sentc_group_sortable_keys` (
	`id` varchar(36) NOT NULL,
	`group_id` varchar(36) NOT NULL,
	`app_id` varchar(36) NOT NULL,
	`encrypted_sortable_key` text NOT NULL,
	`encrypted_sortable_alg` text NOT NULL,
	`encrypted_sortable_encryption_key_id` varchar(36) NOT NULL COMMENT 'the key id which encrypted this key',
	`time` bigint(20) NOT NULL,
	PRIMARY KEY (`id`),
	KEY `group_id` (`group_id`, `app_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TRIGGER `group_delete_sortable_keys` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_sortable_keys WHERE group_id = OLD.id;