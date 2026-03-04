CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    date DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS fighters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    rating REAL NOT NULL DEFAULT 1000,
    max_rating REAL NOT NULL DEFAULT 1000,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    ko_wins INTEGER NOT NULL DEFAULT 0,
    sub_wins INTEGER NOT NULL DEFAULT 0,
    dec_wins INTEGER NOT NULL DEFAULT 0,
    weight_class TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS fights (
    id INTEGER PRIMARY KEY,
    event_id INTEGER NOT NULL,
    winner_id INTEGER NOT NULL,
    loser_id INTEGER NOT NULL,
    date DATETIME NOT NULL,
    fight_time INTEGER NOT NULL,
    weight_class TEXT,
    finish_method TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_id) REFERENCES fighters(id) ON DELETE CASCADE,
    FOREIGN KEY (loser_id) REFERENCES fighters(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ratings (
    id INTEGER PRIMARY KEY,
    fighter_id INTEGER NOT NULL,
    fight_id INTEGER NOT NULL,
    rating REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (fighter_id) REFERENCES fighters(id),
    FOREIGN KEY (fight_id) REFERENCES fights(id),

    UNIQUE (fighter_id, fight_id)
);

CREATE TABLE fight_stats (
    id INTEGER PRIMARY KEY,
    fighter_id INTEGER NOT NULL,
    fight_id INTEGER NOT NULL,
    knock_downs INTEGER,
    total_strikes_hit INTEGER,
    total_strikes_missed INTEGER,
    sig_strikes INTEGER,
    head_strikes INTEGER,
    body_strikes INTEGER,
    leg_strikes INTEGER,
    time_in_control INTEGER,
    takedowns_hit INTEGER,
    takedowns_missed INTEGER,
    submissions_hit INTEGER,
    submissions_missed INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (fighter_id) REFERENCES fighters(id),
    FOREIGN KEY (fight_id) REFERENCES fights(id),

    UNIQUE (fighter_id, fight_id)
);
