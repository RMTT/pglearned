DROP TABLE IF EXISTS imdb_test CASCADE;
-- Create table imdb
CREATE TABLE imdb_test (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    year INTEGER,
    genre TEXT,
    rating NUMERIC(3, 1)
);

-- Insert fake data
INSERT INTO imdb_test (title, year, genre, rating) VALUES
('The Shawshank Redemption', 1994, 'Drama', 9.3),
('The Godfather', 1972, 'Crime', 9.2),
('The Dark Knight', 2008, 'Action', 9.0),
('Pulp Fiction', 1994, 'Crime', 8.9),
('Schindler''s List', 1993, 'Biography', 8.9),
('Forrest Gump', 1994, 'Drama', 8.8),
('Inception', 2010, 'Action', 8.8),
('Fight Club', 1999, 'Drama', 8.8),
('The Matrix', 1999, 'Action', 8.7),
('Goodfellas', 1990, 'Biography', 8.7);
