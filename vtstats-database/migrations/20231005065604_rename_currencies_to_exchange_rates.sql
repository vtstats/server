ALTER TABLE
    currencies RENAME TO exchange_rates;

ALTER TYPE job_kind RENAME VALUE 'update_currency_exchange_rate' TO 'update_exchange_rates';