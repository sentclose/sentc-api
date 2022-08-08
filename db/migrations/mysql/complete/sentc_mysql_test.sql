-- phpMyAdmin SQL Dump
-- version 4.9.5
-- https://www.phpmyadmin.net/
--
-- Host: localhost:3306
-- Erstellungszeit: 08. Aug 2022 um 18:59
-- Server-Version: 10.2.6-MariaDB-log
-- PHP-Version: 7.4.5

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
SET AUTOCOMMIT = 0;
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Datenbank: `sentc`
--

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_app`
--

CREATE TABLE `sentc_app` (
  `id` varchar(36) NOT NULL,
  `customer_id` varchar(36) NOT NULL,
  `identifier` text NOT NULL,
  `hashed_secret_token` varchar(100) NOT NULL COMMENT 'only one per app, when updating the token -> delete the old',
  `hashed_public_token` varchar(100) NOT NULL,
  `hash_alg` text DEFAULT NULL,
  `time` bigint(20) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Daten für Tabelle `sentc_app`
--

INSERT INTO `sentc_app` (`id`, `customer_id`, `identifier`, `hashed_secret_token`, `hashed_public_token`, `hash_alg`, `time`) VALUES
('1665eb92-4513-469f-81d8-b72a62e0134c', 'sentc_int', '', 'cmzOt+BnyErJKsF2qNaiJ/YqsXJymnGQSdvJi5FpeOo=', 'b/t88y7h0zwqOXAtR/UqE4qsPL11PLFvo1e+8PNP8LU=', 'SHA256', 1659606752935);

--
-- Trigger `sentc_app`
--
DELIMITER $$
CREATE TRIGGER `delete_app_jwt` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_app_jwt_keys WHERE app_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `delete_group` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_group WHERE app_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `delete_keys` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_sym_key_management WHERE app_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `delete_options` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_app_options WHERE app_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `delete_user` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_user WHERE app_id = OLD.id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_app_jwt_keys`
--

CREATE TABLE `sentc_app_jwt_keys` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `sign_key` text NOT NULL,
  `verify_key` text NOT NULL,
  `alg` text NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='multiple per app';

--
-- Daten für Tabelle `sentc_app_jwt_keys`
--

INSERT INTO `sentc_app_jwt_keys` (`id`, `app_id`, `sign_key`, `verify_key`, `alg`, `time`) VALUES
('174b531f-8814-42a2-94ab-3c17036183a5', '1665eb92-4513-469f-81d8-b72a62e0134c', 'MIG2AgEAMBAGByqGSM49AgEGBSuBBAAiBIGeMIGbAgEBBDAhH0kMPR68V4jaSECXKgz6hEV+7iHqyOFAAv0Y6EXf7Db3T3rwuwuIfHyD41Rgy0ihZANiAARUyndUd/523UjG1Q5cChBHuntfYiQ5wRUIbONlT78ZrU6eUbncTdaWN72pLYTVIyjmpqgCtszZYKQNMw5I1V4c0mEddOe8bMSmic0egcVxmCCjgQVau8xU4bccdyrllFI=', 'BFTKd1R3/nbdSMbVDlwKEEe6e19iJDnBFQhs42VPvxmtTp5RudxN1pY3vakthNUjKOamqAK2zNlgpA0zDkjVXhzSYR1057xsxKaJzR6BxXGYIKOBBVq7zFThtxx3KuWUUg==', 'ES384', 1659606752935);

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_app_options`
--

CREATE TABLE `sentc_app_options` (
  `app_id` varchar(36) NOT NULL,
  `group_create` int(11) NOT NULL COMMENT 'create a group',
  `group_get` int(11) NOT NULL COMMENT 'get the group keys',
  `group_invite` int(11) NOT NULL COMMENT 'sending invites',
  `group_reject_invite` int(11) NOT NULL,
  `group_accept_invite` int(11) NOT NULL,
  `group_join_req` int(11) NOT NULL COMMENT 'sending join req',
  `group_accept_join_req` int(11) NOT NULL,
  `group_reject_join_req` int(11) NOT NULL,
  `group_key_rotation` int(11) NOT NULL,
  `group_user_delete` int(11) NOT NULL,
  `group_change_rank` int(11) NOT NULL,
  `group_delete` int(11) NOT NULL,
  `group_leave` int(11) NOT NULL,
  `user_exists` int(11) NOT NULL,
  `user_register` int(11) NOT NULL,
  `user_delete` int(11) NOT NULL,
  `user_update` int(11) NOT NULL COMMENT 'change identifier',
  `user_change_password` int(11) NOT NULL,
  `user_reset_password` int(11) NOT NULL,
  `user_prepare_login` int(11) NOT NULL,
  `user_done_login` int(11) NOT NULL,
  `user_public_data` int(11) NOT NULL,
  `user_refresh` int(11) NOT NULL,
  `key_register` int(11) NOT NULL,
  `key_get` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='option: 0 = not allowed,  1 = public token, 2 = secret token';

--
-- Daten für Tabelle `sentc_app_options`
--

INSERT INTO `sentc_app_options` (`app_id`, `group_create`, `group_get`, `group_invite`, `group_reject_invite`, `group_accept_invite`, `group_join_req`, `group_accept_join_req`, `group_reject_join_req`, `group_key_rotation`, `group_user_delete`, `group_change_rank`, `group_delete`, `group_leave`, `user_exists`, `user_register`, `user_delete`, `user_update`, `user_change_password`, `user_reset_password`, `user_prepare_login`, `user_done_login`, `user_public_data`, `user_refresh`, `key_register`, `key_get`) VALUES
('1665eb92-4513-469f-81d8-b72a62e0134c', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_customer`
--

CREATE TABLE `sentc_customer` (
  `id` varchar(36) NOT NULL COMMENT 'the user_id from user table because customer and user are related',
  `email` text NOT NULL,
  `email_validate_sent` bigint(20) NOT NULL,
  `email_validate` tinyint(1) NOT NULL DEFAULT 0,
  `email_status` int(11) NOT NULL DEFAULT 1 COMMENT 'the status of the send email: 1 = normal, other value = error code',
  `email_error_msg` text DEFAULT NULL,
  `email_token` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_customer`
--
DELIMITER $$
CREATE TRIGGER `delete_app` AFTER DELETE ON `sentc_customer` FOR EACH ROW DELETE FROM sentc_app WHERE customer_id = OLD.id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group`
--

CREATE TABLE `sentc_group` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `parent` varchar(36) DEFAULT NULL,
  `identifier` text DEFAULT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_group`
--
DELIMITER $$
CREATE TRIGGER `group_delete_invites` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `group_delete_keys` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_keys WHERE group_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `group_delete_user` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_user WHERE group_id = OLD.id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_keys`
--

CREATE TABLE `sentc_group_keys` (
  `id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `private_key_pair_alg` text NOT NULL,
  `encrypted_private_key` text NOT NULL,
  `public_key` text NOT NULL,
  `group_key_alg` text NOT NULL,
  `encrypted_ephemeral_key` text DEFAULT NULL COMMENT 'after key rotation, encrypt this key with every group member public key',
  `encrypted_group_key_by_eph_key` text DEFAULT NULL COMMENT 'encrypted group master key, encrypted by the eph key. this key needs to distribute to all group member',
  `ephemeral_alg` text DEFAULT NULL COMMENT 'the alg of the eph key',
  `previous_group_key_id` varchar(36) DEFAULT NULL COMMENT 'the key which was used to encrypt the eph key',
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_user`
--

CREATE TABLE `sentc_group_user` (
  `user_id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL COMMENT 'joined time',
  `rank` int(11) NOT NULL,
  `key_upload_session_id` varchar(36) DEFAULT NULL COMMENT 'this is used when there are many keys used in this group. then upload the keys via pagination. this is only used for accept join req',
  `type` tinyint(4) NOT NULL DEFAULT 0 COMMENT '0 = normal user, 1 = group from parent group'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_group_user`
--
DELIMITER $$
CREATE TRIGGER `group_user_delete_key_rotation_keys` AFTER DELETE ON `sentc_group_user` FOR EACH ROW DELETE FROM sentc_group_user_key_rotation WHERE user_id = OLD.user_id AND group_id = OLD.group_id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `group_user_delete_user_keys` AFTER DELETE ON `sentc_group_user` FOR EACH ROW DELETE FROM sentc_group_user_keys WHERE user_id = OLD.user_id AND group_id = OLD.group_id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_user_invites_and_join_req`
--

CREATE TABLE `sentc_group_user_invites_and_join_req` (
  `user_id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `type` int(11) NOT NULL COMMENT '0 = invite (keys needed); 1 = join req (no keys needed)',
  `time` bigint(20) NOT NULL,
  `key_upload_session_id` varchar(36) DEFAULT NULL COMMENT 'if there are too many keys used in this group -> upload the keys via session. this is only used for invite req'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='the invite req from the group to an user';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_user_keys`
--

CREATE TABLE `sentc_group_user_keys` (
  `k_id` varchar(36) NOT NULL,
  `user_id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `encrypted_group_key` text NOT NULL,
  `encrypted_alg` text NOT NULL COMMENT 'the alg from the public key',
  `encrypted_group_key_key_id` varchar(36) NOT NULL COMMENT 'the public key id, which encrypted the group key',
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='every key for one user to one group (m:n), multiple keys for user (via key rotation';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_user_key_rotation`
--

CREATE TABLE `sentc_group_user_key_rotation` (
  `key_id` varchar(36) NOT NULL,
  `user_id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `encrypted_ephemeral_key` text NOT NULL COMMENT 'encrypted by users public key on the server',
  `encrypted_eph_key_key_id` varchar(36) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='after a key rotation, before done key rotation';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_internally_db_version`
--

CREATE TABLE `sentc_internally_db_version` (
  `version` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='for migration';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_sym_key_management`
--

CREATE TABLE `sentc_sym_key_management` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `master_key_id` varchar(36) NOT NULL COMMENT 'the key which encrypted this key (e.g. a group key)',
  `creator_id` varchar(36) NOT NULL,
  `encrypted_key` text NOT NULL,
  `master_key_alg` text NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Symmetric key created by the sdk';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user`
--

CREATE TABLE `sentc_user` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `identifier` varchar(200) NOT NULL,
  `time` bigint(20) NOT NULL COMMENT 'registered at'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_user`
--
DELIMITER $$
CREATE TRIGGER `user_delete_jwt_refresh` AFTER DELETE ON `sentc_user` FOR EACH ROW DELETE FROM sentc_user_token WHERE user_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `user_delete_user_keys` AFTER DELETE ON `sentc_user` FOR EACH ROW DELETE FROM sentc_user_keys WHERE user_id = OLD.id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_action_log`
--

CREATE TABLE `sentc_user_action_log` (
  `user_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL,
  `action_id` int(11) NOT NULL COMMENT '0 = done login; 1 = refresh token or init client',
  `app_id` varchar(36) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_keys`
--

CREATE TABLE `sentc_user_keys` (
  `id` varchar(36) NOT NULL,
  `user_id` varchar(36) NOT NULL,
  `client_random_value` text NOT NULL,
  `public_key` text NOT NULL,
  `encrypted_private_key` text NOT NULL,
  `keypair_encrypt_alg` text NOT NULL,
  `encrypted_sign_key` text NOT NULL,
  `verify_key` text NOT NULL,
  `keypair_sign_alg` text NOT NULL,
  `derived_alg` text NOT NULL,
  `encrypted_master_key` text NOT NULL,
  `master_key_alg` text NOT NULL,
  `encrypted_master_key_alg` text NOT NULL,
  `hashed_auth_key` text NOT NULL,
  `time` bigint(20) NOT NULL COMMENT 'active since'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='multiple keys per user';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_token`
--

CREATE TABLE `sentc_user_token` (
  `user_id` varchar(36) NOT NULL,
  `token` varchar(100) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `test`
--

CREATE TABLE `test` (
  `id` varchar(36) NOT NULL,
  `name` text NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Indizes der exportierten Tabellen
--

--
-- Indizes für die Tabelle `sentc_app`
--
ALTER TABLE `sentc_app`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `hashed_secret_token` (`hashed_secret_token`),
  ADD UNIQUE KEY `hashed_public_token` (`hashed_public_token`);

--
-- Indizes für die Tabelle `sentc_app_jwt_keys`
--
ALTER TABLE `sentc_app_jwt_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`);

--
-- Indizes für die Tabelle `sentc_app_options`
--
ALTER TABLE `sentc_app_options`
  ADD PRIMARY KEY (`app_id`);

--
-- Indizes für die Tabelle `sentc_customer`
--
ALTER TABLE `sentc_customer`
  ADD PRIMARY KEY (`id`);

--
-- Indizes für die Tabelle `sentc_group`
--
ALTER TABLE `sentc_group`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`),
  ADD KEY `parent` (`parent`);

--
-- Indizes für die Tabelle `sentc_group_keys`
--
ALTER TABLE `sentc_group_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `group_id` (`group_id`,`app_id`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_group_user`
--
ALTER TABLE `sentc_group_user`
  ADD PRIMARY KEY (`user_id`,`group_id`);

--
-- Indizes für die Tabelle `sentc_group_user_invites_and_join_req`
--
ALTER TABLE `sentc_group_user_invites_and_join_req`
  ADD PRIMARY KEY (`user_id`,`group_id`);

--
-- Indizes für die Tabelle `sentc_group_user_keys`
--
ALTER TABLE `sentc_group_user_keys`
  ADD PRIMARY KEY (`k_id`,`user_id`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_group_user_key_rotation`
--
ALTER TABLE `sentc_group_user_key_rotation`
  ADD PRIMARY KEY (`key_id`,`user_id`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_internally_db_version`
--
ALTER TABLE `sentc_internally_db_version`
  ADD PRIMARY KEY (`version`);

--
-- Indizes für die Tabelle `sentc_sym_key_management`
--
ALTER TABLE `sentc_sym_key_management`
  ADD PRIMARY KEY (`id`),
  ADD KEY `master_key` (`master_key_id`,`app_id`) USING BTREE,
  ADD KEY `by_user` (`app_id`,`creator_id`);

--
-- Indizes für die Tabelle `sentc_user`
--
ALTER TABLE `sentc_user`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `app_id` (`app_id`,`identifier`);

--
-- Indizes für die Tabelle `sentc_user_action_log`
--
ALTER TABLE `sentc_user_action_log`
  ADD PRIMARY KEY (`user_id`,`time`,`app_id`);

--
-- Indizes für die Tabelle `sentc_user_keys`
--
ALTER TABLE `sentc_user_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `user_id` (`user_id`);

--
-- Indizes für die Tabelle `sentc_user_token`
--
ALTER TABLE `sentc_user_token`
  ADD PRIMARY KEY (`user_id`,`token`,`app_id`) USING BTREE;

--
-- Indizes für die Tabelle `test`
--
ALTER TABLE `test`
  ADD PRIMARY KEY (`id`);
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
