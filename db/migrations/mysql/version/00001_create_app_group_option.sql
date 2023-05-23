CREATE TABLE `sentc_app_group_options` (
  `app_id` varchar(36) NOT NULL,
  `max_key_rotation_month` int(11) NOT NULL,
  `min_rank_key_rotation` int(11) NOT NULL,
  PRIMARY KEY (`app_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- trigger for app delete
CREATE TRIGGER `delete_group_options` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_app_group_options WHERE app_id = OLD.id;

-- insert the customer app
INSERT INTO `sentc_app_group_options` (`app_id`, `max_key_rotation_month`, `min_rank_key_rotation`) VALUES
('1665eb92-4513-469f-81d8-b72a62e0134c', 100, 4);
