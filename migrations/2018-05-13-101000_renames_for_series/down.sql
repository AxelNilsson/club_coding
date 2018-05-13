ALTER TABLE `series` CHANGE COLUMN `archived` `is_archived` tinyint(1) NOT NULL DEFAULT '0';
ALTER TABLE `series` CHANGE COLUMN `title` `name` varchar(255) NOT NULL;