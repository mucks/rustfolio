create table users (
    id varchar (36) primary key,
    username varchar(50) unique not null,
    email varchar(50) unique not null,
    password varchar (60) not null,
    created_on timestamp not null,
    last_login timestamp
);