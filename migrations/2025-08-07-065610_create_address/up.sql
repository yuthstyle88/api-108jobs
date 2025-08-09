CREATE TABLE address
(
    id            serial PRIMARY KEY,
    address_line1 text        NOT NULL,
    address_line2 text,
    subdistrict   text,
    district      text        NOT NULL,
    province      text        NOT NULL,
    postal_code   text        NOT NULL,
    country_id    varchar(2)           DEFAULT 'TH',
    is_default    boolean              DEFAULT FALSE,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz
);