ALTER TABLE local_site
    ADD COLUMN enable_self_promotion boolean NOT NULL DEFAULT TRUE;

UPDATE
    local_site
SET
    enable_self_promotion = CASE WHEN site.content_warning IS NULL THEN
        FALSE
    ELSE
        TRUE
    END
FROM
    site
WHERE
    -- only local site has private key
    site.private_key IS NOT NULL;

