-- phpMyAdmin SQL Dump
-- version 4.8.4
-- https://www.phpmyadmin.net/
--
-- Server version: 8.0.15-5
-- PHP Version: 7.2.34

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
SET AUTOCOMMIT = 0;
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

-- --------------------------------------------------------

--
-- Table structure for table `bot_admins`
--

CREATE TABLE `bot_admins` (
  `perm_id` int(10) UNSIGNED NOT NULL,
  `server_id` bigint(20) UNSIGNED NOT NULL,
  `user_id` bigint(20) UNSIGNED NOT NULL,
  `floppadmin` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `bug_reports`
--

CREATE TABLE `bug_reports` (
  `bug_id` int(11) NOT NULL,
  `channel_id` bigint(20) NOT NULL,
  `message_id` bigint(20) NOT NULL,
  `title` tinytext COLLATE utf8mb4_bin NOT NULL,
  `status` enum('resolved','low','medium','high','critical','closed','forgevanilla') CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL DEFAULT 'medium',
  `timestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `legacy` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `bug_reports__links`
--

CREATE TABLE `bug_reports__links` (
  `link_id` int(11) NOT NULL,
  `bug_id` int(11) NOT NULL,
  `link_url` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  `link_title` tinytext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin ROW_FORMAT=COMPACT;

-- --------------------------------------------------------

--
-- Table structure for table `channel_blacklist`
--

CREATE TABLE `channel_blacklist` (
  `id` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `channel_id` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `custom_commands`
--

CREATE TABLE `custom_commands` (
  `command_id` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `name` tinytext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  `command_json` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  `documentation` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `floppa_images`
--

CREATE TABLE `floppa_images` (
  `id` int(11) NOT NULL,
  `image_url` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `list_guilds`
--

CREATE TABLE `list_guilds` (
  `guild_id` bigint(20) NOT NULL,
  `guild_name` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `lotr_mod_bot_prefix`
--

CREATE TABLE `lotr_mod_bot_prefix` (
  `server_id` bigint(20) NOT NULL,
  `prefix` tinytext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `mc_server_ip`
--

CREATE TABLE `mc_server_ip` (
  `server_id` bigint(20) NOT NULL,
  `mc_ip` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  `port` smallint(6) NOT NULL DEFAULT '25565'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `roles`
--

CREATE TABLE `roles` (
  `server_id` bigint(20) NOT NULL,
  `role_id` bigint(20) NOT NULL,
  `role_name` tinytext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  `role_properties` text CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  `role_colour` int(10) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `roles__aliases`
--

CREATE TABLE `roles__aliases` (
  `alias_uid` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `alias_name` tinytext COLLATE utf8mb4_bin NOT NULL,
  `role_id` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

-- --------------------------------------------------------

--
-- Table structure for table `user_blacklist`
--

CREATE TABLE `user_blacklist` (
  `id` int(11) NOT NULL,
  `server_id` bigint(20) NOT NULL,
  `user_id` bigint(20) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `bot_admins`
--
ALTER TABLE `bot_admins`
  ADD PRIMARY KEY (`perm_id`);

--
-- Indexes for table `bug_reports`
--
ALTER TABLE `bug_reports`
  ADD PRIMARY KEY (`bug_id`);

--
-- Indexes for table `bug_reports__links`
--
ALTER TABLE `bug_reports__links`
  ADD PRIMARY KEY (`link_id`);

--
-- Indexes for table `channel_blacklist`
--
ALTER TABLE `channel_blacklist`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `custom_commands`
--
ALTER TABLE `custom_commands`
  ADD PRIMARY KEY (`command_id`);

--
-- Indexes for table `floppa_images`
--
ALTER TABLE `floppa_images`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `list_guilds`
--
ALTER TABLE `list_guilds`
  ADD PRIMARY KEY (`guild_id`);

--
-- Indexes for table `lotr_mod_bot_prefix`
--
ALTER TABLE `lotr_mod_bot_prefix`
  ADD PRIMARY KEY (`server_id`);

--
-- Indexes for table `mc_server_ip`
--
ALTER TABLE `mc_server_ip`
  ADD PRIMARY KEY (`server_id`);

--
-- Indexes for table `roles`
--
ALTER TABLE `roles`
  ADD PRIMARY KEY (`role_id`);

--
-- Indexes for table `roles__aliases`
--
ALTER TABLE `roles__aliases`
  ADD PRIMARY KEY (`alias_uid`);

--
-- Indexes for table `user_blacklist`
--
ALTER TABLE `user_blacklist`
  ADD PRIMARY KEY (`id`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `bot_admins`
--
ALTER TABLE `bot_admins`
  MODIFY `perm_id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `bug_reports`
--
ALTER TABLE `bug_reports`
  MODIFY `bug_id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `bug_reports__links`
--
ALTER TABLE `bug_reports__links`
  MODIFY `link_id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `channel_blacklist`
--
ALTER TABLE `channel_blacklist`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `custom_commands`
--
ALTER TABLE `custom_commands`
  MODIFY `command_id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `floppa_images`
--
ALTER TABLE `floppa_images`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `roles__aliases`
--
ALTER TABLE `roles__aliases`
  MODIFY `alias_uid` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `user_blacklist`
--
ALTER TABLE `user_blacklist`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
