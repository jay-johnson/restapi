CREATE DATABASE mydb;
--
CREATE USER datawriter;

ALTER USER datawriter WITH PASSWORD '123321';
GRANT ALL PRIVILEGES ON DATABASE mydb TO datawriter;
--
CREATE TABLE users (
    id INT GENERATED ALWAYS AS IDENTITY,
    email TEXT NOT NULL,
    password character varying(512) NOT NULL,
    state INT DEFAULT 0 NOT NULL,
    verified INT DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT timezone('UTC'::text, now()) NOT NULL,
    updated_at timestamp with time zone,
    role character varying(20) NOT NULL,
    PRIMARY KEY(id)
);
ALTER TABLE users OWNER TO datawriter;
ALTER TABLE ONLY users ADD CONSTRAINT users_email_key UNIQUE (email);
CREATE INDEX idx_users_user_id ON users(id);
CREATE INDEX idx_users_email ON users(email);

CREATE TABLE users_verified (
    id INT GENERATED ALWAYS AS IDENTITY,
    user_id INT NOT NULL,
    token VARCHAR(512) NOT NULL,
    email TEXT NOT NULL,
    state INT DEFAULT 0 NOT NULL,
    exp_date timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT timezone('UTC'::text, now()) NOT NULL,
    verify_date timestamp with time zone,
    updated_at timestamp with time zone,
    PRIMARY KEY(id),
    CONSTRAINT fk_user_id
        FOREIGN KEY(user_id)
        REFERENCES users(id)
);
ALTER TABLE users_verified OWNER TO datawriter;
ALTER TABLE ONLY users_verified ADD CONSTRAINT users_verified_user_id_key UNIQUE (user_id);
ALTER TABLE ONLY users_verified ADD CONSTRAINT users_verified_email_key UNIQUE (email);
CREATE INDEX idx_users_verified_user_id ON users_verified(user_id);

CREATE TABLE users_tokens (
    id INT GENERATED ALWAYS AS IDENTITY,
    user_id INT,
    token VARCHAR(512) NOT NULL,
    state INT DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT timezone('UTC'::text, now()) NOT NULL,
    updated_at timestamp with time zone,
    PRIMARY KEY(id),
    CONSTRAINT fk_user_id
        FOREIGN KEY(user_id)
        REFERENCES users(id)
);
ALTER TABLE users_tokens OWNER TO datawriter;
CREATE INDEX idx_users_tokens_id ON users_tokens(id);
CREATE INDEX idx_users_tokens_user_id ON users_tokens(user_id);

CREATE TABLE users_data (
    id INT GENERATED ALWAYS AS IDENTITY,
    user_id INT,
    filename VARCHAR(512) NOT NULL,
    size_in_bytes BIGINT NOT NULL,
    comments VARCHAR(512) NOT NULL,
    data_type VARCHAR(64) NOT NULL,
    encoding VARCHAR(64) NOT NULL,
    sloc VARCHAR(1024) NOT NULL,
    created_at timestamp with time zone DEFAULT timezone('UTC'::text, now()) NOT NULL,
    updated_at timestamp with time zone,
    PRIMARY KEY(id),
    CONSTRAINT fk_user_id
        FOREIGN KEY(user_id)
        REFERENCES users(id)
);
ALTER TABLE users_data OWNER TO datawriter;
CREATE INDEX idx_users_data_id ON users_data(id);

CREATE TABLE users_otp (
    id INT GENERATED ALWAYS AS IDENTITY,
    user_id INT,
    token VARCHAR(512) NOT NULL,
    email TEXT,
    state INT DEFAULT 0 NOT NULL,
    exp_date timestamp with time zone,
    consumed_date timestamp with time zone,
    created_at timestamp with time zone DEFAULT timezone('UTC'::text, now()) NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk_user_id
        FOREIGN KEY(user_id)
        REFERENCES users(id)
);
ALTER TABLE users_otp OWNER TO datawriter;
CREATE INDEX idx_users_otp_id ON users_otp(id);
CREATE INDEX idx_users_otp_user_id ON users_otp(user_id);
