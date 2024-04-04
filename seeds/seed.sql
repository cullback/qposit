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
VALUES (
    1,
    '2024-us-presidential-election',
    'US presidential election',
    '
## Resolution criteria

This **question** will ~resolve~ as Yes for the person who wins the 2024 US presidential election,
and No for all other options. This will be the person who wins the majority of votes
in the Electoral College, or selected by Congress following the contingency procedure in the Twelfth
Amendment. This question is not limited to the individuals currently listed below; the question may
resolve as No for all listed options, and options may be added later.',
    'active',
    1710007894419934,
    1710008894419934
);

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

INSERT INTO
    'order' (id, created_at, book_id, user_id, quantity, remaining, price, is_buy, status)
VALUES
    (1, 1710007894419934, 1, 0, 20, 20, 56, 0, 'open'),
    (5, 1710007894419935, 1, 0, 20, 19, 58, 0, 'open'),
    (2, 1710007894419934, 1, 0, 25, 25, 23, 1, 'open'),
    (3, 1710007894419935, 1, 0, 20, 20, 22, 1, 'open'),
    (4, 1710007894419935, 1, 0, 30, 21, 22, 1, 'open');
