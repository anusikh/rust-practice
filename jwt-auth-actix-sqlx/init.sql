CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username varchar(255),
  password varchar(255),
);

CREATE TABLE articles (
  id SERIAL PRIMARY KEY,
  title varchar(255),
  content text,
  published_by bigint(20) unsigned,
  published_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_articles_users FOREIGN KEY (published_by) REFERENCES users (id)
);
