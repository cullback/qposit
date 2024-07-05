PRAGMA foreign_keys = ON;

INSERT INTO user(id, username, created_at, balance, available, password_hash) VALUES
    (1, 'admin', 1710007894419934, 10000 * 10000, 10000 * 10000, '$argon2id$v=19$m=19456,t=2,p=1$xiP9HtGCRNu4qIOQVhj/og$cV9Wjt9ytLYtWqOUOJembuR9hQdp2mihcYBQN/I+oC4'),
    (2, 'account2', 1710007894419934, 10000 * 10000, 10000 * 10000, '$argon2id$v=19$m=19456,t=2,p=1$oY7oDHdkawz7pDgD91BJqw$qdQnWbgzexhJBC23YLJ8M8TJhHi22zf+BMHJAqAL9Rw'),
    (3, 'account3', 1710007894419934, 10000 * 10000, 10000 * 10000, '$argon2id$v=19$m=19456,t=2,p=1$oY7oDHdkawz7pDgD91BJqw$qdQnWbgzexhJBC23YLJ8M8TJhHi22zf+BMHJAqAL9Rw');

INSERT INTO invite(code, created_by, created_at) VALUES
    ('MK6H5JI3HM', 1, 1710007894419934),
    ('EWDELYEEAM', 1, 1710007894419934),
    ('P5TG1W3AFQ', 1, 1710007894419934),
    ('I09U265NKN', 1, 1710007894419934),
    ('Q30J6YNY7W', 1, 1710007894419934),
    ('23TU0SXMBK', 1, 1710007894419934),
    ('0NMWQ9M7CY', 1, 1710007894419934),
    ('42MAIG5EVP', 1, 1710007894419934),
    ('LPNTFXWZAA', 1, 1710007894419934),
    ('2WC3FKEV31', 1, 1710007894419934);

INSERT INTO event(id, slug, title, description, status, created_at, expires_at) VALUES
(1, 'demo-event', 'Demo Event',
    '
## Resolution criteria

This event will be resolved however I please.',
    'active',
    1710007894419934,
    1710008894419934
);


INSERT INTO market(id, event_id, title) VALUES
    (1, 1, 'Option A'),
    (2, 1, 'Option B');

-- INSERT INTO 'order' (id, created_at, book_id, user_id, quantity, remaining, price, is_buy, status) VALUES
--     (1, 1710007894419934, 1, 2, 20, 20, 5600, 0, 'open'),
--     (2, 1710007894419935, 1, 2, 20, 19, 5800, 0, 'open'),
--     (3, 1710007894419934, 1, 2, 25, 25, 2300, 1, 'open'),
--     (4, 1710007894419935, 1, 2, 20, 20, 2200, 1, 'open'),
--     (5, 1710007894419935, 1, 2, 30, 21, 2200, 1, 'open'),
--     (6, 1710007894419935, 1, 1, 20, 0, 1000, 1, 'filled'),
--     (7, 1710007894419935, 1, 2, 20, 0, 1000, 0, 'filled'),
--     (8, 1710007894419935, 2, 1, 25, 0, 1500, 1, 'filled'),
--     (9, 1710007894419935, 2, 2, 25, 0, 1500, 0, 'filled');

-- INSERT INTO trade(id, created_at, tick, book_id, taker_id, maker_id, taker_oid, maker_oid, quantity, price, is_buy)
-- VALUES
--     (123, 1710007894419934, 456, 1, 2, 1, 0, 0, 20, 1000, 1),
--     (124, 1710007894419934, 234, 2, 2, 1, 0, 0, 25, 1500, 1);


