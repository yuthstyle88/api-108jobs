-- Create banks table with Thailand and Vietnam banks
CREATE TABLE banks (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    country VARCHAR(100) NOT NULL CHECK (country IN ('Thailand', 'Vietnam')),
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
CREATE INDEX idx_banks_country ON banks(country);

-- Insert popular Thai banks
INSERT INTO banks (name, country, bank_code, swift_code) VALUES
('Bangkok Bank', 'Thailand', 'BBL', 'BKKBTHBK'),
('Kasikornbank', 'Thailand', 'KBANK', 'KASITHBK'),
('Krung Thai Bank', 'Thailand', 'KTB', 'KRTHTHBK'),
('Siam Commercial Bank', 'Thailand', 'SCB', 'SICOTHBK'),
('TMBThanachart Bank', 'Thailand', 'TTB', 'TMBKTHBK'),
('Bank of Ayudhya', 'Thailand', 'BAY', 'AYUDTHBK'),
('Government Savings Bank', 'Thailand', 'GSB', 'GSBATHBK'),
('Krungthai Card', 'Thailand', 'KTC', 'KTCBTHBK');

-- Insert popular Vietnamese banks
INSERT INTO banks (name, country, bank_code, swift_code) VALUES
('Vietcombank', 'Vietnam', 'VCB', 'BFTVVNVX'),
('VietinBank', 'Vietnam', 'CTG', 'ICBVVNVX'),
('BIDV', 'Vietnam', 'BIDV', 'BIDVVNVX'),
('Agribank', 'Vietnam', 'AGRI', 'VBAAVNVX'),
('Techcombank', 'Vietnam', 'TCB', 'VTCBVNVX'),
('MB Bank', 'Vietnam', 'MB', 'MSCBVNVX'),
('ACB', 'Vietnam', 'ACB', 'ASCBVNVX'),
('VPBank', 'Vietnam', 'VPB', 'VPBKVNVX'),
('SHB', 'Vietnam', 'SHB', 'SHBAVNVX'),
('TPBank', 'Vietnam', 'TPB', 'TPBVVNVX');