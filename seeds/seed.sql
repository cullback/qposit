PRAGMA foreign_keys = ON;

INSERT INTO
    user(id, username, password_hash, created_at, balance)
VALUES (
    1,
    'testaccount',
    '$argon2id$v=19$m=19456,t=2,p=1$oY7oDHdkawz7pDgD91BJqw$qdQnWbgzexhJBC23YLJ8M8TJhHi22zf+BMHJAqAL9Rw', -- password123
    1710007894419934,
    10000 * 100
),
(
    2,
    'strategy',
    '$argon2id$v=19$m=19456,t=2,p=1$oY7oDHdkawz7pDgD91BJqw$qdQnWbgzexhJBC23YLJ8M8TJhHi22zf+BMHJAqAL9Rw', -- password123
    1710007894419934,
    10000 * 100
)
;


INSERT INTO
    market(id, slug, title, description, status, created_at, expires_at)
VALUES (
    1,
    '2024-us-presidential-election',
    'US Presidential Election',
    '
## Resolution criteria

This question will resolve as Yes for the person who wins the 2024 US presidential election,
and No for all other options. This will be the person who wins the majority of votes
in the Electoral College, or selected by Congress following the contingency procedure in the Twelfth
Amendment. This question is not limited to the individuals currently listed below; the question may
resolve as No for all listed options, and options may be added later.',
    'active',
    1710007894419934,
    1710008894419934
),
(
    2,
    '2024-us-republican-nominee',
    'US Republican VP Nominee',
    '
## Resolution criteria

This question will resolve as Yes for the person who is selected as the Republican nominee for Vice President.
',
    'active',
    1710007894419934,
    1710008894419934
);

INSERT INTO
    book(id, market_id, title)
VALUES
    (1, 1, 'Donald Trump'),
    (2, 1, 'Joe Biden'),
    (3, 2, 'Tim Scott'),
    (4, 2, 'Other man'),
    (5, 2, 'Other Woman'),
    (6, 2, 'Kristi Noem');

INSERT INTO
    'order' (id, created_at, book_id, user_id, quantity, remaining, price, is_buy, status)
VALUES
    (1, 1710007894419934, 1, 2, 20, 20, 5600, 0, 'open'),
    (2, 1710007894419935, 1, 2, 20, 19, 5800, 0, 'open'),
    (3, 1710007894419934, 1, 2, 25, 25, 2300, 1, 'open'),
    (4, 1710007894419935, 1, 2, 20, 20, 2200, 1, 'open'),
    (5, 1710007894419935, 1, 2, 30, 21, 2200, 1, 'open'),
    (6, 1710007894419935, 1, 1, 20, 0, 1000, 1, 'filled'),
    (7, 1710007894419935, 1, 2, 20, 0, 1000, 0, 'filled'),
    (8, 1710007894419935, 2, 1, 25, 0, 1500, 1, 'filled'),
    (9, 1710007894419935, 2, 2, 25, 0, 1500, 0, 'filled');

INSERT INTO
    trade(id, created_at, tick, book_id, taker_id, maker_id, taker_oid, maker_oid, quantity, price, is_buy)
VALUES
    (123, 1710007894419934, 456, 1, 2, 1, 0, 0, 20, 1000, 1),
    (124, 1710007894419934, 234, 2, 2, 1, 0, 0, 25, 1500, 1);


