CREATE TABLE IF NOT EXISTS user(
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username        TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    create_time     INTEGER NOT NULL,
    balance         INTEGER NOT NULL DEFAULT 0 CHECK(balance >= 0)
);

CREATE TABLE IF NOT EXISTS session(
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     INTEGER NOT NULL,
    -- user agent
    -- create_time
    -- expire_time
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);



CREATE TABLE IF NOT EXISTS market(
    id          INTEGER NOT NULL PRIMARY KEY,
    title       TEXT NOT NULL CHECK (length(title) <= 50),
    description TEXT NOT NULL,
    status      TEXT NOT NULL CHECK(status IN ('active', 'resolved')),
    create_time INTEGER NOT NULL,
    expire_time INTEGER NOT NULL
);


CREATE TABLE IF NOT EXISTS book(
    id          INTEGER NOT NULL PRIMARY KEY autoincrement,
    market_id   TEXT not null,
    name        TEXT not null,
    status      TEXT not null check(status IN ('active', 'resolved')),
    value       INTEGER,
    FOREIGN KEY (market_id) REFERENCES market(id)
);


CREATE TABLE IF NOT EXISTS "order"(
    id          INTEGER PRIMARY KEY,
    create_time INTEGER NOT NULL,
    book_id     INTEGER NOT NULL,
    user_id     INTEGER NOT NULL,
    quantity    INTEGER NOT NULL,
    filled_size INTEGER NOT NULL DEFAULT 0,
    price       INTEGER NOT NULL,
    is_buy      INTEGER NOT NULL check(is_buy IN (0, 1)),
    status      TEXT NOT NULL check(
        status IN ('open', 'filled', 'cancelled')
    ),
    FOREIGN KEY (user_id) REFERENCES user(id),
    FOREIGN KEY (book_id) REFERENCES book(id)
);


CREATE TABLE IF NOT EXISTS trade(
    id          INTEGER NOT NULL PRIMARY KEY autoincrement,
    create_time INTEGER NOT NULL,
    tick        INTEGER NOT NULL,
    book_id     INTEGER NOT NULL,
    taker_id    INTEGER NOT NULL,
    maker_id    INTEGER NOT NULL,
    quantity    INTEGER NOT NULL,
    price       INTEGER NOT NULL,
    is_buy      INTEGER NOT NULL check(is_buy IN (0, 1)),
    FOREIGN KEY (book_id) REFERENCES book(id),
    FOREIGN KEY (taker_id) REFERENCES 'order'(id),
    FOREIGN KEY (maker_id) REFERENCES 'order'(id)
);


CREATE TABLE IF NOT EXISTS position(
    book_id     INTEGER NOT NULL,
    user_id     INTEGER NOT NULL,
    position    INTEGER NOT NULL,
    FOREIGN KEY (book_id) REFERENCES book(id),
    FOREIGN KEY (user_id) REFERENCES user(id),
    PRIMARY KEY (book_id, user_id)
);
