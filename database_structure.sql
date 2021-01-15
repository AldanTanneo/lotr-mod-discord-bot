SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
SET AUTOCOMMIT = 0;
START TRANSACTION;
SET time_zone = "+00:00";

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;


CREATE TABLE `bot_admins` (
  `perm_id` int(10) UNSIGNED NOT NULL,
  `server_id` bigint(20) UNSIGNED NOT NULL,
  `user_id` bigint(20) UNSIGNED NOT NULL,
  `floppadmin` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;

CREATE TABLE `channel_blacklist` (
  `id` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `channel_id` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;

CREATE TABLE `custom_commands` (
  `command_id` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `command_json` longtext CHARACTER SET utf8 COLLATE utf8_bin NOT NULL,
  `documentation` longtext CHARACTER SET utf8 COLLATE utf8_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;

CREATE TABLE `floppa_images` (
  `id` int(11) NOT NULL,
  `image_url` text CHARACTER SET utf8 COLLATE utf8_bin
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;

CREATE TABLE `lotr_mod_bot_prefix` (
  `server_id` bigint(20) NOT NULL,
  `prefix` text CHARACTER SET utf8 COLLATE utf8_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;

CREATE TABLE `mc_server_ip` (
  `server_id` bigint(20) NOT NULL,
  `mc_ip` text CHARACTER SET utf8 COLLATE utf8_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;

CREATE TABLE `user_blacklist` (
  `id` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `user_id` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_bin;


ALTER TABLE `bot_admins`
  ADD PRIMARY KEY (`perm_id`);

ALTER TABLE `channel_blacklist`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `custom_commands`
  ADD PRIMARY KEY (`command_id`);

ALTER TABLE `floppa_images`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `lotr_mod_bot_prefix`
  ADD PRIMARY KEY (`server_id`);

ALTER TABLE `mc_server_ip`
  ADD PRIMARY KEY (`server_id`);

ALTER TABLE `user_blacklist`
  ADD PRIMARY KEY (`id`);


ALTER TABLE `bot_admins`
  MODIFY `perm_id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `channel_blacklist`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

ALTER TABLE `custom_commands`
  MODIFY `command_id` int(11) NOT NULL AUTO_INCREMENT;

ALTER TABLE `floppa_images`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

ALTER TABLE `user_blacklist`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
