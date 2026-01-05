-- If these are not urls, it will crash the server
UPDATE
    user_
SET
    avatar = NULL
WHERE
    avatar NOT LIKE 'http%';

UPDATE
    user_
SET
    banner = NULL
WHERE
    banner NOT LIKE 'http%';

UPDATE
    category
SET
    icon = NULL
WHERE
    icon NOT LIKE 'http%';

UPDATE
    category
SET
    banner = NULL
WHERE
    banner NOT LIKE 'http%';

