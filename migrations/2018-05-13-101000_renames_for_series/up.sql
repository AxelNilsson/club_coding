ALTER TABLE `series` CHANGE COLUMN `is_archived` `archived` tinyint(1) NOT NULL DEFAULT '0';
ALTER TABLE `series` CHANGE COLUMN `name` `title` varchar(255) NOT NULL;