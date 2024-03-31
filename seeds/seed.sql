INSERT INTO
    user(username, password_hash, created_at, balance)
VALUES (
    'testaccount',
    '$argon2id$v=19$m=19456,t=2,p=1$oY7oDHdkawz7pDgD91BJqw$qdQnWbgzexhJBC23YLJ8M8TJhHi22zf+BMHJAqAL9Rw', -- password123
    1710007894419934,
    10000 * 100
);


INSERT INTO
    market(id, slug, title, description, status, created_at, expires_at)
VALUES
    (1, '2024-us-presidential-election', 'US presidential election', 'Who will win?', 'active', 1710007894419934, 1710008894419934);

INSERT INTO
    book(id, market_id, title)
VALUES
    (1, 1, 'Joe Biden'),
    (2, 1, 'Donald Trump');


INSERT INTO
    trade(id, created_at, tick, book_id, taker_id, maker_id, quantity, price, is_buy)
VALUES
    (123, 1710007894419934, 456, 1, 0, 0, 20, 10, 1),
    (124, 1710007894419934, 234, 2, 0, 0, 25, 15, 1);
