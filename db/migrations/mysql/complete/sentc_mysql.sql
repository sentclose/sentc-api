-- phpMyAdmin SQL Dump
-- version 4.9.5
-- https://www.phpmyadmin.net/
--
-- Host: localhost:3306
-- Erstellungszeit: 26. Jul 2023 um 20:28
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
  `owner_id` varchar(36) NOT NULL COMMENT 'the customer group',
  `owner_type` int(11) NOT NULL,
  `identifier` text NOT NULL,
  `hashed_secret_token` varchar(100) NOT NULL COMMENT 'only one per app, when updating the token -> delete the old',
  `hashed_public_token` varchar(100) NOT NULL,
  `hash_alg` text DEFAULT NULL,
  `time` bigint(20) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_app`
--

CREATE TRIGGER `delete_app_content` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_content WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_app_jwt` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_app_jwt_keys WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_file_options` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_file_options WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_group` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_group WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_group_options` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_app_group_options WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_keys` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_sym_key_management WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_options` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_app_options WHERE app_id = OLD.id

;

CREATE TRIGGER `delete_user` AFTER DELETE ON `sentc_app` FOR EACH ROW DELETE FROM sentc_user WHERE app_id = OLD.id

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_app_group_options`
--

CREATE TABLE `sentc_app_group_options` (
  `app_id` varchar(36) NOT NULL,
  `max_key_rotation_month` int(11) NOT NULL,
  `min_rank_key_rotation` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

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

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_app_options`
--

CREATE TABLE `sentc_app_options` (
  `app_id` varchar(36) NOT NULL,
  `group_create` int(11) NOT NULL COMMENT 'create a group',
  `group_get` int(11) NOT NULL COMMENT 'get the group keys',
  `group_user_keys` int(11) NOT NULL,
  `group_user_update_check` int(11) NOT NULL,
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
  `key_get` int(11) NOT NULL,
  `group_auto_invite` int(11) NOT NULL,
  `group_list` int(11) NOT NULL,
  `file_register` int(11) NOT NULL,
  `file_part_upload` int(11) NOT NULL,
  `file_get` int(11) NOT NULL,
  `file_part_download` int(11) NOT NULL,
  `user_device_register` int(11) NOT NULL,
  `user_device_delete` int(11) NOT NULL,
  `user_device_list` int(11) NOT NULL,
  `group_invite_stop` int(11) NOT NULL,
  `user_key_update` int(11) NOT NULL,
  `content_search` int(11) NOT NULL,
  `file_delete` int(11) NOT NULL,
  `content` int(11) NOT NULL,
  `content_small` int(11) NOT NULL,
  `content_med` int(11) NOT NULL,
  `content_large` int(11) NOT NULL,
  `content_x_large` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='option: 0 = not allowed,  1 = public token, 2 = secret token';

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_captcha`
--

CREATE TABLE `sentc_captcha` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `solution` text NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_content`
--

CREATE TABLE `sentc_content` (
  `id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `item` varchar(50) NOT NULL,
  `time` bigint(20) NOT NULL,
  `belongs_to_group` varchar(36) DEFAULT NULL,
  `belongs_to_user` varchar(36) DEFAULT NULL,
  `creator` varchar(36) NOT NULL,
  `category` varchar(50) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_customer`
--

CREATE TABLE `sentc_customer` (
  `id` varchar(36) NOT NULL COMMENT 'the user_id from user table because customer and user are related',
  `email` text NOT NULL,
  `name` text NOT NULL,
  `first_name` text NOT NULL,
  `company` text DEFAULT NULL,
  `email_validate_sent` bigint(20) NOT NULL,
  `email_validate` tinyint(1) NOT NULL DEFAULT 0,
  `email_status` int(11) NOT NULL DEFAULT 1 COMMENT 'the status of the send email: 1 = normal, other value = error code',
  `email_error_msg` text DEFAULT NULL,
  `email_token` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_customer`
--

CREATE TRIGGER `delete_app` AFTER DELETE ON `sentc_customer` FOR EACH ROW DELETE FROM sentc_app WHERE owner_id = OLD.id AND owner_type = 0

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_customer_group`
--

CREATE TABLE `sentc_customer_group` (
  `sentc_group_id` varchar(36) NOT NULL,
  `name` text DEFAULT NULL,
  `des` text DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_customer_group`
--

CREATE TRIGGER `delete_customer_group_apps` AFTER DELETE ON `sentc_customer_group` FOR EACH ROW DELETE FROM sentc_app WHERE owner_id = OLD.sentc_group_id AND owner_type = 1

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_file`
--

CREATE TABLE `sentc_file` (
  `id` varchar(36) NOT NULL,
  `owner` varchar(36) NOT NULL COMMENT 'user_id',
  `encrypted_file_name` text DEFAULT NULL,
  `belongs_to` varchar(36) DEFAULT NULL,
  `belongs_to_type` int(11) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `encrypted_key` text NOT NULL,
  `master_key_id` varchar(36) NOT NULL,
  `status` int(11) NOT NULL COMMENT '0 = to delete, 1 = avalible, 2 = disabled',
  `delete_at` bigint(20) NOT NULL COMMENT '0 = not deleted, time when the file was deleted',
  `time` bigint(20) NOT NULL,
  `encrypted_key_alg` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_file`
--

CREATE TRIGGER `file_delete_parts` AFTER DELETE ON `sentc_file` FOR EACH ROW DELETE FROM sentc_file_part WHERE file_id = OLD.id

;

CREATE TRIGGER `file_session_delete` AFTER DELETE ON `sentc_file` FOR EACH ROW DELETE FROM sentc_file_session WHERE file_id = OLD.id

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_file_options`
--

CREATE TABLE `sentc_file_options` (
  `app_id` varchar(36) NOT NULL,
  `file_storage` int(11) NOT NULL COMMENT '0 = our backend; 1 = customer backend',
  `storage_url` text DEFAULT NULL COMMENT 'when file_storage != 0',
  `auth_token` varchar(50) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_file_part`
--

CREATE TABLE `sentc_file_part` (
  `id` varchar(36) NOT NULL,
  `file_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `size` bigint(20) NOT NULL COMMENT 'only set when using our backend',
  `sequence` int(11) NOT NULL,
  `extern` tinyint(1) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_file_session`
--

CREATE TABLE `sentc_file_session` (
  `id` varchar(36) NOT NULL,
  `file_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `created_at` bigint(20) NOT NULL,
  `expected_size` int(11) NOT NULL,
  `max_chunk_size` int(11) NOT NULL
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
  `type` int(11) NOT NULL COMMENT '0 0 normal group, 1 = user group',
  `is_connected_group` tinyint(1) NOT NULL,
  `invite` tinyint(1) NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_group`
--

CREATE TRIGGER `group_delete_hmac_keys` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_hmac_keys WHERE group_id = OLD.id

;

CREATE TRIGGER `group_delete_invites` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_user_invites_and_join_req WHERE group_id = OLD.id

;

CREATE TRIGGER `group_delete_keys` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_keys WHERE group_id = OLD.id

;

CREATE TRIGGER `group_delete_sortable_keys` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_sortable_keys WHERE group_id = OLD.id

;

CREATE TRIGGER `group_delete_user` AFTER DELETE ON `sentc_group` FOR EACH ROW DELETE FROM sentc_group_user WHERE group_id = OLD.id

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_hmac_keys`
--

CREATE TABLE `sentc_group_hmac_keys` (
  `id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `encrypted_hmac_key` text NOT NULL,
  `encrypted_hmac_alg` text NOT NULL,
  `encrypted_hmac_encryption_key_id` varchar(36) NOT NULL COMMENT 'the key id which encrypted this key',
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='the hmac keys are the keys for searchable encryption hashes.';

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
  `encrypted_sign_key` text DEFAULT NULL,
  `verify_key` text DEFAULT NULL,
  `keypair_sign_alg` text DEFAULT NULL,
  `group_key_alg` text NOT NULL,
  `encrypted_ephemeral_key` text DEFAULT NULL COMMENT 'after key rotation, encrypt this key with every group member public key',
  `encrypted_group_key_by_eph_key` text DEFAULT NULL COMMENT 'encrypted group master key, encrypted by the eph key. this key needs to distribute to all group member',
  `ephemeral_alg` text DEFAULT NULL COMMENT 'the alg of the eph key',
  `previous_group_key_id` varchar(36) DEFAULT NULL COMMENT 'the key which was used to encrypt the eph key',
  `time` bigint(20) NOT NULL,
  `signed_by_user_id` varchar(36) DEFAULT NULL,
  `signed_by_user_sign_key_id` varchar(36) DEFAULT NULL,
  `signed_by_user_sign_key_alg` text DEFAULT NULL,
  `public_key_sig` text DEFAULT NULL,
  `public_key_sig_key_id` varchar(36) DEFAULT NULL COMMENT 'the key id which was used to create the sig'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_sortable_keys`
--

CREATE TABLE `sentc_group_sortable_keys` (
  `id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `encrypted_sortable_key` text NOT NULL,
  `encrypted_sortable_alg` text NOT NULL,
  `encrypted_sortable_encryption_key_id` varchar(36) NOT NULL COMMENT 'the key id which encrypted this key',
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
  `type` tinyint(4) NOT NULL DEFAULT 0 COMMENT '0 = normal user, 1 = group from parent group, 2 = a group as member'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_group_user`
--

CREATE TRIGGER `group_user_delete_key_rotation_keys` AFTER DELETE ON `sentc_group_user` FOR EACH ROW DELETE FROM sentc_group_user_key_rotation WHERE user_id = OLD.user_id AND group_id = OLD.group_id

;

CREATE TRIGGER `group_user_delete_user_keys` AFTER DELETE ON `sentc_group_user` FOR EACH ROW DELETE FROM sentc_group_user_keys WHERE user_id = OLD.user_id AND group_id = OLD.group_id

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_group_user_invites_and_join_req`
--

CREATE TABLE `sentc_group_user_invites_and_join_req` (
  `user_id` varchar(36) NOT NULL,
  `group_id` varchar(36) NOT NULL,
  `type` int(11) NOT NULL COMMENT '0 = invite (keys needed); 1 = join req (no keys needed)',
  `time` bigint(20) NOT NULL,
  `key_upload_session_id` varchar(36) DEFAULT NULL COMMENT 'if there are too many keys used in this group -> upload the keys via session. this is only used for invite req',
  `user_type` int(11) NOT NULL COMMENT '0 = normal user, 2 = group as member',
  `new_user_rank` int(11) NOT NULL COMMENT 'only for invite req.'
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
  `encrypted_eph_key_key_id` varchar(36) NOT NULL,
  `error` text DEFAULT NULL
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
  `user_group_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL COMMENT 'registered at'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

--
-- Trigger `sentc_user`
--

CREATE TRIGGER `user_delete_user_device` AFTER DELETE ON `sentc_user` FOR EACH ROW DELETE FROM sentc_user_device WHERE user_id = OLD.id

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_action_log`
--

CREATE TABLE `sentc_user_action_log` (
  `user_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL,
  `action_id` int(11) NOT NULL COMMENT '0 = done login; 1 = refresh token or init client',
  `app_id` varchar(36) NOT NULL,
  `amount` int(11) NOT NULL COMMENT 'when saving how many'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_device`
--

CREATE TABLE `sentc_user_device` (
  `id` varchar(36) NOT NULL,
  `user_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `device_identifier` varchar(200) NOT NULL,
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
  `time` bigint(20) NOT NULL COMMENT 'active since',
  `token` varchar(100) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='multiple device per user';

--
-- Trigger `sentc_user_device`
--

CREATE TRIGGER `user_delete_challenge` AFTER DELETE ON `sentc_user_device` FOR EACH ROW DELETE FROM sentc_user_device_challenge WHERE device_id = OLD.id

;

CREATE TRIGGER `user_delete_jwt_refresh` AFTER DELETE ON `sentc_user_device` FOR EACH ROW DELETE FROM sentc_user_token WHERE device_id = OLD.id

;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_device_challenge`
--

CREATE TABLE `sentc_user_device_challenge` (
  `challenge` varchar(100) NOT NULL,
  `device_id` varchar(36) NOT NULL,
  `app_id` varchar(36) NOT NULL,
  `time` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- --------------------------------------------------------

--
-- Tabellenstruktur für Tabelle `sentc_user_token`
--

CREATE TABLE `sentc_user_token` (
  `device_id` varchar(36) NOT NULL,
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
-- Indizes für die Tabelle `sentc_app_group_options`
--
ALTER TABLE `sentc_app_group_options`
  ADD PRIMARY KEY (`app_id`);

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
-- Indizes für die Tabelle `sentc_captcha`
--
ALTER TABLE `sentc_captcha`
  ADD PRIMARY KEY (`id`);

--
-- Indizes für die Tabelle `sentc_content`
--
ALTER TABLE `sentc_content`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`),
  ADD KEY `time` (`time`),
  ADD KEY `item` (`item`) USING BTREE,
  ADD KEY `cat_id` (`category`);

--
-- Indizes für die Tabelle `sentc_customer`
--
ALTER TABLE `sentc_customer`
  ADD PRIMARY KEY (`id`);

--
-- Indizes für die Tabelle `sentc_customer_group`
--
ALTER TABLE `sentc_customer_group`
  ADD PRIMARY KEY (`sentc_group_id`);

--
-- Indizes für die Tabelle `sentc_file`
--
ALTER TABLE `sentc_file`
  ADD PRIMARY KEY (`id`),
  ADD KEY `belongs_to` (`belongs_to`,`belongs_to_type`),
  ADD KEY `owner` (`owner`,`app_id`);

--
-- Indizes für die Tabelle `sentc_file_options`
--
ALTER TABLE `sentc_file_options`
  ADD PRIMARY KEY (`app_id`);

--
-- Indizes für die Tabelle `sentc_file_part`
--
ALTER TABLE `sentc_file_part`
  ADD PRIMARY KEY (`id`),
  ADD KEY `file` (`file_id`,`app_id`);

--
-- Indizes für die Tabelle `sentc_file_session`
--
ALTER TABLE `sentc_file_session`
  ADD PRIMARY KEY (`id`);

--
-- Indizes für die Tabelle `sentc_group`
--
ALTER TABLE `sentc_group`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`),
  ADD KEY `parent` (`parent`);

--
-- Indizes für die Tabelle `sentc_group_hmac_keys`
--
ALTER TABLE `sentc_group_hmac_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `app_id` (`app_id`,`group_id`);

--
-- Indizes für die Tabelle `sentc_group_keys`
--
ALTER TABLE `sentc_group_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `group_id` (`group_id`,`app_id`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_group_sortable_keys`
--
ALTER TABLE `sentc_group_sortable_keys`
  ADD PRIMARY KEY (`id`),
  ADD KEY `group_id` (`group_id`,`app_id`);

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
  ADD KEY `app_id` (`app_id`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_user_action_log`
--
ALTER TABLE `sentc_user_action_log`
  ADD PRIMARY KEY (`user_id`,`time`,`app_id`);

--
-- Indizes für die Tabelle `sentc_user_device`
--
ALTER TABLE `sentc_user_device`
  ADD PRIMARY KEY (`id`),
  ADD KEY `user_id` (`user_id`,`app_id`) USING BTREE,
  ADD KEY `app_id` (`app_id`,`token`),
  ADD KEY `device_identifier` (`device_identifier`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_user_device_challenge`
--
ALTER TABLE `sentc_user_device_challenge`
  ADD PRIMARY KEY (`challenge`,`device_id`,`app_id`,`time`) USING BTREE;

--
-- Indizes für die Tabelle `sentc_user_token`
--
ALTER TABLE `sentc_user_token`
  ADD PRIMARY KEY (`device_id`,`token`,`app_id`) USING BTREE;

--
-- Indizes für die Tabelle `test`
--
ALTER TABLE `test`
  ADD PRIMARY KEY (`id`);
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
