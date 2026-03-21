-- Rollback multi-currency support

DROP TRIGGER IF EXISTS update_ride_session_updated_at ON ride_session;
DROP TRIGGER IF EXISTS update_pricing_config_updated_at ON pricing_config;
DROP TRIGGER IF EXISTS update_currency_updated_at ON currency;

DROP TABLE IF EXISTS ride_meter_snapshot;
DROP TABLE IF EXISTS ride_session;
DROP TABLE IF EXISTS pricing_config;
DROP TABLE IF EXISTS currency_rate_history;
DROP TABLE IF EXISTS currency;
