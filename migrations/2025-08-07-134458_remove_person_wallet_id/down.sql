-- Restore wallet_id field to person table (rollback)

ALTER TABLE person ADD COLUMN wallet_id INTEGER REFERENCES wallet(id) ON DELETE SET NULL;
CREATE INDEX idx_person_wallet_id ON person(wallet_id);

-- Populate the wallet_id field based on existing wallet.person_id relationships
UPDATE person 
SET wallet_id = (SELECT id FROM wallet WHERE wallet.person_id = person.id) 
WHERE EXISTS (SELECT 1 FROM wallet WHERE wallet.person_id = person.id);