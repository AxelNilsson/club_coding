CREATE TABLE `request_network_hashes` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `payment_id` bigint(20) unsigned NOT NULL,
    `hash` varchar(255) NOT NULL,
    `created` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
