/* Users table stores user account information 
multiline */
CREATE TABLE users (
    /* Primary key 
    multiline */
    id INTEGER PRIMARY KEY,
    /* Username for login 
    multiline */
    username VARCHAR(255) NOT NULL,
    /* Email address 
    multiline */
    email VARCHAR(255) UNIQUE NOT NULL,
    /* When the user registered 
    multiline */
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

/* Posts table stores blog posts 
multiline */
CREATE TABLE posts (
    /* Primary key 
    multiline */
    id INTEGER PRIMARY KEY,
    /* Post title 
    multiline */
    title VARCHAR(255) NOT NULL,
    /* Foreign key linking to users 
    multiline */
    user_id INTEGER NOT NULL,
    /* Main body text 
    multiline */
    body TEXT NOT NULL,
    /* When the post was created 
    multiline */
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
