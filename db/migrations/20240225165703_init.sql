CREATE TABLE IF NOT EXISTS user(
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username        TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    balance         INTEGER NOT NULL DEFAULT 0 CHECK(balance >= 0),
    available       INTEGER NOT NULL DEFAULT 0 CHECK(available >= 0)
);


CREATE TABLE IF NOT EXISTS invite(
    id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    used_by     INTEGER,
    created_by  INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    FOREIGN KEY (used_by) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES user(id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS session(
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     INTEGER NOT NULL,
    ip_address  TEXT NOT NULL,
    user_agent  TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS market(
    id          INTEGER NOT NULL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL CHECK (length(title) <= 50),
    description TEXT NOT NULL,
    status      TEXT NOT NULL CHECK (status IN ('active', 'resolved')),
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL
);


CREATE TABLE IF NOT EXISTS book(
    id          INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    market_id   INTEGER NOT NULL,
    title       TEXT NOT NULL,
    -- status      TEXT NOT NULL CHECK (status IN ('active', 'resolved')),
    value       INTEGER,
    FOREIGN KEY (market_id) REFERENCES market(id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS "order"(
    id          INTEGER PRIMARY KEY,
    created_at  INTEGER NOT NULL,
    book_id     INTEGER NOT NULL,
    user_id     INTEGER NOT NULL,
    quantity    INTEGER NOT NULL CHECK (quantity > 0),
    remaining   INTEGER NOT NULL,
    price       INTEGER NOT NULL CHECK (price > 0),
    is_buy      INTEGER NOT NULL CHECK (is_buy IN (0, 1)),
    status      TEXT NOT NULL CHECK(
        status IN ('open', 'filled', 'cancelled')
    ),
    FOREIGN KEY (user_id) REFERENCES user(id),
    FOREIGN KEY (book_id) REFERENCES book(id)
);


CREATE TABLE IF NOT EXISTS trade(
    id          INTEGER NOT NULL PRIMARY KEY,
    created_at  INTEGER NOT NULL,
    tick        INTEGER NOT NULL,
    book_id     INTEGER NOT NULL,
    taker_id    INTEGER NOT NULL,
    maker_id    INTEGER NOT NULL,
    taker_oid   INTEGER NOT NULL,
    maker_oid   INTEGER NOT NULL,
    quantity    INTEGER NOT NULL,
    price       INTEGER NOT NULL,
    is_buy      INTEGER NOT NULL CHECK (is_buy IN (0, 1)),
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
