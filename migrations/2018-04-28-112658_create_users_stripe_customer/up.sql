CREATE TABLE `users_stripe_customer` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) NOT NULL,
  `uuid` varchar(255) NOT NULL,
  `account_balance` bigint(20) NOT NULL,
  `business_vat_id` varchar(255) DEFAULT NULL,
  `created_at_stripe` bigint(20) unsigned NOT NULL,
  `default_source` varchar(255) DEFAULT NULL,
  `delinquent` tinyint(1) NOT NULL,
  `desc` text DEFAULT NULL,
  `email` varchar(255) DEFAULT NULL,
  `livemode` tinyint(1) NOT NULL,
  `created` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;