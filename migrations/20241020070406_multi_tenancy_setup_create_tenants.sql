-- Add migration script here
CREATE TABLE IF NOT EXISTS tenants (
       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
       name VARCHAR(255) NOT NULL UNIQUE,
       db_user VARCHAR(255) NOT NULL UNIQUE,
       db_password_encrypted VARCHAR(512) NOT NULL UNIQUE,
       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
       updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
