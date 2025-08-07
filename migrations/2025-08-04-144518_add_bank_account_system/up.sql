-- Create banks table with Thailand and Vietnam banks
CREATE TABLE banks (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    country_id VARCHAR(2) NOT NULL,
    bank_code VARCHAR(20),
    swift_code VARCHAR(20),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- Create user_bank_accounts table
CREATE TABLE user_bank_accounts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES local_user(id) ON DELETE CASCADE,
    bank_id INTEGER NOT NULL REFERENCES banks(id),
    account_number VARCHAR(50) NOT NULL,
    account_name VARCHAR(255) NOT NULL,
    is_default BOOLEAN DEFAULT FALSE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    verification_image_path VARCHAR(255)
);

-- Create indexes
CREATE INDEX idx_user_bank_accounts_user_id ON user_bank_accounts(user_id);
CREATE INDEX idx_user_bank_accounts_bank_id ON user_bank_accounts(bank_id);
CREATE INDEX idx_banks_country ON banks(country_id);

-- Insert popular Thai banks
INSERT INTO banks (name, country_id, bank_code, swift_code) VALUES
('Bangkok Bank', 'TH', 'BBL', 'BKKBTHBK'),
('Kasikornbank', 'TH', 'KBANK', 'KASITHBK'),
('Krung Thai Bank', 'TH', 'KTB', 'KRTHTHBK'),
('Siam Commercial Bank', 'TH', 'SCB', 'SICOTHBK'),
('TMBThanachart Bank', 'TH', 'TTB', 'TMBKTHBK'),
('Bank of Ayudhya', 'TH', 'BAY', 'AYUDTHBK'),
('Government Savings Bank', 'TH', 'GSB', 'GSBATHBK'),
('Krungthai Card', 'TH', 'KTC', 'KTCBTHBK');

-- Insert popular Vietnamese banks
INSERT INTO banks (name, country_id, bank_code, swift_code) VALUES
('Vietcombank', 'VI', 'VCB', 'BFTVVNVX'),
('VietinBank', 'VI', 'CTG', 'ICBVVNVX'),
('BIDV', 'VI', 'BIDV', 'BIDVVNVX'),
('Agribank', 'VI', 'AGRI', 'VBAAVNVX'),
('Techcombank', 'VI', 'TCB', 'VTCBVNVX'),
('MB Bank', 'VI', 'MB', 'MSCBVNVX'),
('ACB', 'VI', 'ACB', 'ASCBVNVX'),
('VPBank', 'VI', 'VPB', 'VPBKVNVX'),
('SHB', 'VI', 'SHB', 'SHBAVNVX'),
('TPBank', 'VI', 'TPB', 'TPBVVNVX');