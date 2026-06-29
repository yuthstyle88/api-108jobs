-- Thai Baht — default currency for 108Jobs platform.
-- Required by Currency::get_default() at runtime and in tests.
INSERT INTO public.currency (code, name, symbol, numeric_code, coin_to_currency_rate, decimal_places,
    thousands_separator, decimal_separator, symbol_position, is_active, is_default)
VALUES ('THB', 'Thai Baht', '฿', 764, 100, 2, ',', '.', 'prefix', true, true);
