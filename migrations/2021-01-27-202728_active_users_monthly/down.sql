ALTER TABLE site_aggregates
    DROP COLUMN users_active_day,
    DROP COLUMN users_active_week,
    DROP COLUMN users_active_month,
    DROP COLUMN users_active_half_year;

ALTER TABLE category_aggregates
    DROP COLUMN users_active_day,
    DROP COLUMN users_active_week,
    DROP COLUMN users_active_month,
    DROP COLUMN users_active_half_year;

DROP FUNCTION site_aggregates_activity (i text);

DROP FUNCTION category_aggregates_activity (i text);

