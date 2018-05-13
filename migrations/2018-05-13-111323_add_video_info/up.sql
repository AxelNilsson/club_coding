ALTER TABLE `videos` ADD (
  `archived` tinyint(1) NOT NULL DEFAULT '0',
  `vimeo_id` varchar(255) NOT NULL
);