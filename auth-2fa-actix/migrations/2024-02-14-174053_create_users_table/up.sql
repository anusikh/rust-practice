-- Your SQL goes here
CREATE TABLE users
  (
     id           VARCHAR(255) PRIMARY KEY,
     email        VARCHAR(255) NOT NULL,
     NAME         VARCHAR(255) NOT NULL,
     password     VARCHAR(255) NOT NULL,
     otp_enabled  BOOLEAN NOT NULL,
     otp_verified BOOLEAN NOT NULL,
     otp_base32   VARCHAR(255),
     otp_auth_url VARCHAR(255),
     created_at   TIMESTAMP,
     updated_at   TIMESTAMP
  ) 