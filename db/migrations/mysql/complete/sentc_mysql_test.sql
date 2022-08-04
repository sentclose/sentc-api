-- phpMyAdmin SQL Dump
-- version 4.9.5
-- https://www.phpmyadmin.net/
--
-- Host: localhost:3306
-- Erstellungszeit: 04. Aug 2022 um 16:11
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
-- Tabellenstruktur für Tabelle `app`
--

CREATE TABLE `app` (
  `id` varchar(36) NOT NULL,
  `customer_id` varchar(36) NOT NULL,
  `identifier` text NOT NULL,
  `hashed_secret_token` varchar(100) NOT NULL COMMENT 'only one per app, when updating the token -> delete the old',
  `hashed_public_token` varchar(100) NOT NULL,
  `hash_alg` text DEFAULT NULL,
  `time` bigint(20) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Daten für Tabelle `app`
--

INSERT INTO `app` (`id`, `customer_id`, `identifier`, `hashed_secret_token`, `hashed_public_token`, `hash_alg`, `time`) VALUES
('1665eb92-4513-469f-81d8-b72a62e0134c', 'sentc_int', '', 'cmzOt+BnyErJKsF2qNaiJ/YqsXJymnGQSdvJi5FpeOo=', 'b/t88y7h0zwqOXAtR/UqE4qsPL11PLFvo1e+8PNP8LU=', 'SHA256', 1659606752935);

--
-- Trigger `app`
--
DELIMITER $$
CREATE TRIGGER `delete_app_jwt` AFTER DELETE ON `app` FOR EACH ROW DELETE FROM app_jwt_keys WHERE app_id = OLD.id
$$
DELIMITER ;
DELIMITER $$
CREATE TRIGGER `delete_user` AFTER DELETE ON `app` FOR EACH ROW DELETE FROM user WHERE app_id = OLD.id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `app_active_log`
--

CREATE TABLE `app_active_log` (
  `app_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL,
  `action_id` int(11) NOT NULL COMMENT 'what was done. internal id'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `app_jwt_keys`
--

CREATE TABLE `app_jwt_keys` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `sign_key` text NOT NULL,
  `verify_key` text NOT NULL,
  `alg` text NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='multiple per app';

--
-- Daten für Tabelle `app_jwt_keys`
--

INSERT INTO `app_jwt_keys` (`id`, `app_id`, `sign_key`, `verify_key`, `alg`, `time`) VALUES
('174b531f-8814-42a2-94ab-3c17036183a5', '1665eb92-4513-469f-81d8-b72a62e0134c', 'MIG2AgEAMBAGByqGSM49AgEGBSuBBAAiBIGeMIGbAgEBBDAhH0kMPR68V4jaSECXKgz6hEV+7iHqyOFAAv0Y6EXf7Db3T3rwuwuIfHyD41Rgy0ihZANiAARUyndUd/523UjG1Q5cChBHuntfYiQ5wRUIbONlT78ZrU6eUbncTdaWN72pLYTVIyjmpqgCtszZYKQNMw5I1V4c0mEddOe8bMSmic0egcVxmCCjgQVau8xU4bccdyrllFI=', 'BFTKd1R3/nbdSMbVDlwKEEe6e19iJDnBFQhs42VPvxmtTp5RudxN1pY3vakthNUjKOamqAK2zNlgpA0zDkjVXhzSYR1057xsxKaJzR6BxXGYIKOBBVq7zFThtxx3KuWUUg==', 'ES384', 1659606752935);

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `app_options`
--

CREATE TABLE `app_options` (
  `app_id` varchar(36) NOT NULL,
  `group_create` int(11) NOT NULL COMMENT 'create a group',
  `group_get` int(11) NOT NULL COMMENT 'get the group keys',
  `group_invite` int(11) NOT NULL COMMENT 'sending invites',
  `group_join_req` int(11) NOT NULL COMMENT 'sending join req',
  `group_accept_join_req` int(11) NOT NULL,
  `group_key_rotation` int(11) NOT NULL,
  `group_user_delete` int(11) NOT NULL,
  `group_user_update` int(11) NOT NULL COMMENT 'update rank',
  `user_regsiter` int(11) NOT NULL,
  `user_delete` int(11) NOT NULL,
  `user_update` int(11) NOT NULL COMMENT 'change identifier',
  `user_change_password` int(11) NOT NULL,
  `user_reset_password` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='option: 0 = not allowed,  1 = public token, 2 = secret token';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `internally_db_version`
--

CREATE TABLE `internally_db_version` (
  `version` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='for migration';

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
-- Tabellenstruktur für Tabelle `test`
--

CREATE TABLE `test` (
  `id` varchar(36) NOT NULL,
  `name` text NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `user`
--

CREATE TABLE `user` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `identifier` varchar(200) NOT NULL,
  `time` bigint(20) NOT NULL COMMENT 'registered at'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `user`
--
DELIMITER $$
CREATE TRIGGER `user_delete_user_keys` AFTER DELETE ON `user` FOR EACH ROW DELETE FROM user_keys WHERE user_id = OLD.id
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `user_keys`
--

CREATE TABLE `user_keys` (
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

--
-- Indizes der exportierten Tabellen
--

--
-- Indizes für die Tabelle `app`
--
ALTER TABLE `app`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `hashed_secret_token` (`hashed_secret_token`),
  ADD UNIQUE KEY `hashed_public_token` (`hashed_public_token`);

--
-- Indizes für die Tabelle `app_active_log`
--
ALTER TABLE `app_active_log`
  ADD PRIMARY KEY (`app_id`,`time`);

--
-- Indizes für die Tabelle `app_jwt_keys`
--
ALTER TABLE `app_jwt_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`);

--
-- Indizes für die Tabelle `app_options`
--
ALTER TABLE `app_options`
  ADD PRIMARY KEY (`app_id`);

--
-- Indizes für die Tabelle `internally_db_version`
--
ALTER TABLE `internally_db_version`
  ADD PRIMARY KEY (`version`);

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
-- Indizes für die Tabelle `test`
--
ALTER TABLE `test`
  ADD PRIMARY KEY (`id`);

--
-- Indizes für die Tabelle `user`
--
ALTER TABLE `user`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `app_id` (`app_id`,`identifier`);

--
-- Indizes für die Tabelle `user_keys`
--
ALTER TABLE `user_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `user_id` (`user_id`);
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
