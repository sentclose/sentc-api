DROP TRIGGER IF EXISTS `delete_app_search`;

DROP TABLE `sentc_content_searchable_item_parts`;

DROP TABLE `sentc_content_searchable_item`;

ALTER TABLE `sentc_app_options` DROP `content_search`;