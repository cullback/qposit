INSERT INTO
    user(username, password_hash, create_time)
VALUES
    -- password: password
    ('bob', '$argon2id$v=19$m=19456,t=2,p=1$t25Co/ZvcPYNUCeMxP9FqQ$KMfPerJtIvi0wBSoRwgUeJQ9Dnms7oMv4ynoSf65uuE', 1710007894419934);


INSERT INTO
    market(id, title, description, status, create_time, expire_time)
VALUES
    (1, '2024 US Presidential Election', 'Who will win?', 'active', 1710007894419934, 1710008894419934);

INSERT INTO
    book(id, market_id, name, status, value)
VALUES
    (1, 1, 'Joe Biden', 'active', null),
    (2, 1, 'Donald Trump', 'active', null);


INSERT INTO
    trade(id, create_time, tick, book_id, taker_id, maker_id, quantity, price, is_buy)
VALUES
    (123, 1710007894419934, 456, 1, 0, 0, 20, 10, 1),
    (124, 1710007894419934, 234, 2, 0, 0, 25, 15, 1);
