-- Users table stores user account information
CREATE TABLE users (
    -- Primary key
    id INTEGER PRIMARY KEY,
    -- Username for login
    username VARCHAR(255) NOT NULL,
    -- Email address
    email VARCHAR(255) UNIQUE NOT NULL,
    -- When the user registered
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Posts table stores blog posts
CREATE TABLE posts (
    -- Primary key
    id INTEGER PRIMARY KEY,
    -- Post title
    title VARCHAR(255) NOT NULL,
    -- Foreign key linking to users
    user_id INTEGER NOT NULL,
    -- Main body text
    body TEXT NOT NULL,
    -- When the post was created
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
