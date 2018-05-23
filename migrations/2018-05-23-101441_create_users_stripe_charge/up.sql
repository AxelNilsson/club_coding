CREATE TABLE `users_stripe_charge` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) unsigned NOT NULL,
  `series_id` bigint(20) unsigned NOT NULL,
  `uuid` varchar(255) NOT NULL DEFAULT '',
  `amount` int(11) unsigned NOT NULL,
  `amount_refunded` int(10) unsigned NOT NULL,
  `balance_transaction` varchar(255) DEFAULT NULL,
  `captured` tinyint(1) NOT NULL,
  `created_at_stripe` bigint(20) NOT NULL,
  `description` text,
  `destination` varchar(255) DEFAULT NULL,
  `dispute` varchar(255) DEFAULT NULL,
  `failure_code` varchar(255) DEFAULT NULL,
  `failure_message` varchar(255) DEFAULT NULL,
  `livemode` tinyint(1) NOT NULL,
  `on_behalf_of` varchar(255) DEFAULT NULL,
  `order` varchar(255) DEFAULT NULL,
  `paid` tinyint(1) NOT NULL,
  `refunded` tinyint(1) NOT NULL,
  `source_id` varchar(255) NOT NULL DEFAULT '',
  `source_transfer` varchar(255) DEFAULT NULL,
  `statement_descriptor` varchar(255) DEFAULT NULL,
  `status` varchar(255) NOT NULL DEFAULT '',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;