CREATE TABLE `request_network_payments` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `uuid` varchar(255) NOT NULL,
    `user_id` bigint(20) unsigned NOT NULL,
    `serie_id` bigint(20) unsigned NOT NULL,
    `amount_in_eth` varchar(255) NOT NULL,
    `to_address` varchar(255) NOT NULL,
    `reason` varchar(255) NOT NULL,
    `used` tinyint(1) NOT NULL DEFAULT '0',
    `created` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`) 
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;