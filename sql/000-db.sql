
CREATE TABLE IF NOT EXISTS users (
    -- Unique identifier for each user in the database
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Telegram's unique user ID (required for API interactions)
    telegram_id INTEGER NOT NULL UNIQUE,

    -- User's Telegram username (optional, can be null)
    username TEXT UNIQUE,

    -- User's first name from Telegram profile
    first_name TEXT,

    -- User's last name from Telegram profile
    last_name TEXT,

    -- Timestamp of when the user first interacted with the bot
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Flag to track if user is active (1 = active, 0 = inactive/blocked)
    is_active BOOLEAN DEFAULT 1
);

CREATE TABLE IF NOT EXISTS Conversation (
    -- Unique identifier for each message/conversation record
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Foreign key linking to the user who sent/received the message
    user_id INTEGER NOT NULL,

    -- The actual message content
    message TEXT NOT NULL,

    -- Type of message (text, photo, video, command, etc.)
    message_type TEXT DEFAULT 'text',

    -- Flag to identify if message was sent by bot (1) or user (0)
    is_bot_message BOOLEAN DEFAULT 0,

    -- Timestamp of when the message was created
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Links conversation to user, cascades delete if user is removed
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

