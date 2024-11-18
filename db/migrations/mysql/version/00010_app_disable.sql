ALTER TABLE `sentc_app`
	ADD `disabled`    INT    NULL DEFAULT NULL AFTER `time`,
	ADD `disabled_ts` BIGINT NULL DEFAULT NULL AFTER `disabled`;