ALTER TABLE `videos` ADD (
  `series` bigint(20) DEFAULT NULL,
  `episode_number` int(11) DEFAULT NULL
);