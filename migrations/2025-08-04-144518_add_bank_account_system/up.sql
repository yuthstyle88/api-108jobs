-- Banks
CREATE TABLE banks
(
    id         SERIAL PRIMARY KEY,
    name       VARCHAR(255) NOT NULL,
    country_id CHAR(2)      NOT NULL, -- ISO-3166-1 alpha-2
    bank_code  VARCHAR(20),
    swift_code VARCHAR(20),
    is_active  BOOLEAN               DEFAULT TRUE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE banks
    ADD CONSTRAINT chk_country_id_format CHECK (country_id ~ '^[A-Z]{2}$'),
    ADD CONSTRAINT chk_swift_format CHECK (swift_code IS NULL OR swift_code ~ '^[A-Z0-9]{8}([A-Z0-9]{3})?$');

-- User bank accounts
CREATE TABLE user_bank_accounts
(
    id                      SERIAL PRIMARY KEY,
    local_user_id           INTEGER      NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
    bank_id                 INTEGER      NOT NULL REFERENCES banks (id),
    account_number          VARCHAR(50)  NOT NULL,
    account_name            VARCHAR(255) NOT NULL,
    is_default              BOOLEAN               DEFAULT FALSE,
    is_verified             BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ,
    verification_image_path VARCHAR(255)
);

-- Indexes (fixed + recommended)
CREATE INDEX idx_user_bank_accounts_local_user_id ON user_bank_accounts (local_user_id);
CREATE INDEX idx_user_bank_accounts_bank_id ON user_bank_accounts (bank_id);
CREATE INDEX idx_banks_country ON banks (country_id);
CREATE UNIQUE INDEX uniq_default_bank_account_per_user
    ON user_bank_accounts (local_user_id) WHERE is_default = TRUE;
CREATE UNIQUE INDEX uniq_user_bank_account
    ON user_bank_accounts (local_user_id, bank_id, account_number);
CREATE INDEX idx_user_bank_accounts_default
    ON user_bank_accounts (local_user_id, is_default) WHERE is_default = TRUE;

-- Updated-at triggers
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_user_bank_accounts_updated_at
    BEFORE UPDATE
    ON user_bank_accounts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_banks_updated_at
    BEFORE UPDATE
    ON banks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Seed: Thailand (TH)
INSERT INTO banks (name, country_id, bank_code, swift_code)
VALUES ('Bangkok Bank', 'TH', 'BBL', 'BKKBTHBK'),
       ('Kasikornbank', 'TH', 'KBANK', 'KASITHBK'),
       ('Krung Thai Bank', 'TH', 'KTB', 'KRTHTHBK'),
       ('Siam Commercial Bank', 'TH', 'SCB', 'SICOTHBK'),
       ('TMBThanachart Bank', 'TH', 'TTB', 'TMBKTHBK'),
       ('Bank of Ayudhya', 'TH', 'BAY', 'AYUDTHBK'),
       ('Government Savings Bank', 'TH', 'GSB', 'GSBATHBK');

-- Seed: Vietnam (VN)  <-- changed from VI -> VN
INSERT INTO banks (name, country_id, bank_code, swift_code)
VALUES ('Vietcombank', 'VN', 'VCB', 'BFTVVNVX'),
       ('VietinBank', 'VN', 'CTG', 'ICBVVNVX'),
       ('BIDV', 'VN', 'BIDV', 'BIDVVNVX'),
       ('Agribank', 'VN', 'AGRI', 'VBAAVNVX'),
       ('Techcombank', 'VN', 'TCB', 'VTCBVNVX'),
       ('MB Bank', 'VN', 'MB', 'MSCBVNVX'),
       ('ACB', 'VN', 'ACB', 'ASCBVNVX'),
       ('VPBank', 'VN', 'VPB', 'VPBKVNVX'),
       ('SHB', 'VN', 'SHB', 'SHBAVNVX'),
       ('TPBank', 'VN', 'TPB', 'TPBVVNVX');