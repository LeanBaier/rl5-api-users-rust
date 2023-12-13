-- Your SQL goes here
CREATE TABLE IF NOT EXISTS movies
(
    id_movie     bigserial primary key,
    name         varchar(100) not null,
    description  text         not null,
    duration     int          not null,
    release_date timestamp default now(),
    image_link   text         not null
);

CREATE TABLE IF NOT EXISTS categories
(
    id_category serial primary key,
    description varchar(100) not null
);

CREATE TABLE IF NOT EXISTS category_movies
(
    id_category_movie bigserial primary key,
    id_movie          bigint not null,
    id_category       int not null,
    CONSTRAINT fk_category_movie FOREIGN KEY (id_movie) references movies (id_movie),
    CONSTRAINT fk_category_category FOREIGN KEY (id_category) references categories (id_category)
);

CREATE TABLE IF NOT EXISTS actors
(
    id_actor bigserial primary key,
    id_movie bigint       not null,
    name     varchar(100) not null,
    CONSTRAINT fk_actor_movie FOREIGN KEY (id_movie) references movies (id_movie)
);

CREATE TABLE IF NOT EXISTS directors
(
    id_director bigserial primary key,
    id_movie    bigint       not null,
    name        varchar(100) not null,
    CONSTRAINT fk_director_movie FOREIGN KEY (id_movie) references movies (id_movie)
);

CREATE TABLE IF NOT EXISTS rl_role
(
    id_role     serial primary key,
    description varchar(50) not null
);

CREATE TABLE IF NOT EXISTS rl_users
(
    id_user  uuid primary key,
    email    varchar(200) not null,
    nickname varchar(100) not null,
    password text         not null,
    id_role int not null,
    constraint fk_users_role foreign key (id_role) references rl_role (id_role)
);

CREATE TABLE IF NOT EXISTS reviews
(
    id_review bigserial primary key,
    id_movie  bigint not null,
    id_user   uuid   not null,
    score     int    not null,
    comment   text,
    CONSTRAINT fk_review_movie FOREIGN KEY (id_movie) references movies (id_movie),
    CONSTRAINT fk_review_user FOREIGN KEY (id_user) references rl_users (id_user)
);

CREATE TABLE IF NOT EXISTS connections
(
    id_connection uuid primary key,
    id_user       uuid not null,
    connect_at    timestamp default now(),
    ended_at      timestamp
);

CREATE TABLE IF NOT EXISTS user_favorites
(
    id_user_favorite bigserial primary key,
    id_user          uuid   not null,
    id_movie         bigint not null,
    CONSTRAINT fk_favorites_user FOREIGN KEY (id_user) references rl_users (id_user),
    CONSTRAINT fk_favorites_movie FOREIGN KEY (id_movie) references movies (id_movie)
);

INSERT INTO rl_role (description) values ('USER');