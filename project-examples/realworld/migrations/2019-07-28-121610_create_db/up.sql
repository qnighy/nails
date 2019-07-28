CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  email VARCHAR(50) NOT NULL,
  token VARCHAR(250) NOT NULL,
  username VARCHAR(150) NOT NULL,
  bio TEXT,
  image VARCHAR(250)
);
CREATE UNIQUE INDEX index_users_on_email ON users (email);
CREATE UNIQUE INDEX index_users_on_username ON users (username);

CREATE TABLE followings (
  id BIGSERIAL PRIMARY KEY,
  following_id BIGINT NOT NULL,
  follower_id BIGINT NOT NULL,

  CONSTRAINT fk_followings_following_id FOREIGN KEY (following_id)
    REFERENCES users (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE,
  CONSTRAINT fk_followings_follower_id FOREIGN KEY (follower_id)
    REFERENCES users (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE
);
CREATE UNIQUE INDEX index_followings_on_following_id_and_follower_id ON followings (
  following_id,
  follower_id
);
CREATE INDEX index_followings_on_follower_id ON followings (follower_id);

CREATE TABLE tags (
  id BIGSERIAL PRIMARY KEY,
  tag VARCHAR(250) NOT NULL
);
CREATE UNIQUE INDEX index_tags_on_tag ON tags (tag);

CREATE TABLE articles (
  id BIGSERIAL PRIMARY KEY,
  slug VARCHAR(250) NOT NULL,
  title VARCHAR(250) NOT NULL,
  description VARCHAR(250) NOT NULL,
  body TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  author_id BIGINT NOT NULL,

  CONSTRAINT fk_articles_author_id FOREIGN KEY (author_id)
    REFERENCES users (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE
);
CREATE UNIQUE INDEX index_articles_on_slug ON articles (slug);
CREATE INDEX index_articles_on_author_id ON articles (author_id);

CREATE TABLE favorited_articles (
  id BIGSERIAL PRIMARY KEY,
  article_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,

  CONSTRAINT fk_favorited_articles_article_id FOREIGN KEY (article_id)
    REFERENCES articles (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE,
  CONSTRAINT fk_favorited_articles_user_id FOREIGN KEY (user_id)
    REFERENCES users (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE
);
CREATE UNIQUE INDEX index_favorited_articles_on_article_id_and_user_id ON favorited_articles (
  article_id,
  user_id
);
CREATE INDEX index_favorited_articles_on_user_id ON favorited_articles (user_id);

CREATE TABLE comments (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  body TEXT NOT NULL,
  article_id BIGINT NOT NULL,
  author_id BIGINT NOT NULL,

  CONSTRAINT fk_comments_articles FOREIGN KEY (article_id)
    REFERENCES articles (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE,
  CONSTRAINT fk_comments_author_id FOREIGN KEY (author_id)
    REFERENCES users (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE
);
CREATE INDEX index_comments_on_article_id ON comments (article_id);
CREATE INDEX index_comments_on_author_id ON comments (author_id);

CREATE TABLE article_tags (
  id BIGSERIAL PRIMARY KEY,
  article_id BIGINT NOT NULL,
  tag_id BIGINT NOT NULL,

  CONSTRAINT fk_articletags_article_id FOREIGN KEY (article_id)
    REFERENCES articles (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE,
  CONSTRAINT fk_articletags_tag_id FOREIGN KEY (tag_id)
    REFERENCES tags (id)
    ON DELETE RESTRICT
    ON UPDATE RESTRICT
    NOT DEFERRABLE
);
CREATE UNIQUE INDEX index_article_tags_on_article_id_and_tag_id ON article_tags (
  article_id,
  tag_id
);
CREATE UNIQUE INDEX index_article_tags_on_tag_id ON article_tags (tag_id);
