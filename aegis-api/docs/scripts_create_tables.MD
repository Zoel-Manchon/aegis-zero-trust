//Create table sessions
-- Required for UUID generation
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- =========================
-- SESSIONS TABLE
-- =========================
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    user_id INTEGER NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    -- 🔐 store ONLY hashed refresh token
    refresh_token_hash TEXT NOT NULL,

    -- 🔄 token rotation support (optional but recommended)
    parent_id UUID REFERENCES sessions(id) ON DELETE SET NULL,

    -- ⏱ lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,

    -- 🚫 revocation
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,

    -- 🧠 device & security context
    user_agent TEXT,
    ip_address INET,
    device_name TEXT,

    -- 🔐 ensure no duplicate tokens
    CONSTRAINT unique_refresh_token UNIQUE (refresh_token_hash)
);

-- =========================
-- INDEXES (IMPORTANT)
-- =========================

-- lookup sessions by user (e.g. logout all devices)
CREATE INDEX idx_sessions_user_id
ON sessions(user_id);

-- fast expiration cleanup / queries
CREATE INDEX idx_sessions_expires_at
ON sessions(expires_at);

-- only active sessions (common query)
CREATE INDEX idx_sessions_active
ON sessions(user_id)
WHERE revoked = FALSE;

-- lookup by refresh token (CRITICAL for refresh endpoint)
CREATE INDEX idx_sessions_refresh_token
ON sessions(refresh_token_hash);

-- rotation chain lookup
CREATE INDEX idx_sessions_parent_id
ON sessions(parent_id);

-- =========================
-- OPTIONAL: CLEANUP STRATEGY (manual job later)
-- =========================
-- DELETE FROM sessions WHERE expires_at < now();
***supports multiple sessions per user***
Field by Field Explanation
id UUID
session identifier (not user id!)

user_id
FOREIGN KEY → users(id)
***Referential integrity***
***deletes sessions if user is deleted***

refresh_token_hash
Never store raw refresh token
***Store hashed version***

created_at

***when session started***

expires_at
***hard expiration***

revoked + revoked_at
logout, security invalidation

user_agent,ip_address
security->audit->anomaly detection

VERY IMPORTANT (security rule)
 NEVER store raw refresh token
Instead:
refresh_token → hash → store in DB

Indexes:
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

Optional (but recommended soon)
Token rotation support
parent_id UUID

Device tracking
device_name TEXT
***A proper sessions table must link to users, store only hashed refresh tokens, track expiration and revocation, and enable multi-device secure session management from the start.***

🔐 Security
✔ hashed refresh tokens (no leaks)
✔ revocation support
✔ rotation-ready (parent_id)
📱 Multi-device support
✔ multiple sessions per user
✔ device_name
✔ user_agent
✔ ip_address (INET type, not TEXT)

Performance

Indexes are not random—you will actually use them:

Index Why
user_id logout all sessions
expires_at cleanup job
refresh_token_hash refresh endpoint
active (partial) fast “valid sessions” queries
parent_id token rotation chain


----------------------------------------------------------------------------------------------
Access Token (JWT)
  └── short-lived
  └── contains jti

Refresh Token
  └── random 32 bytes
  └── hashed before DB storage

Session
  └── user_id
  └── jti
  └── refresh_token_hash
  └── device metadata ready
  └── expires_at















  /* NEW SCIPT SQL TABLES

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE TYPE user_role AS ENUM ('user', 'admin');


**USERS TABLE**
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    user_role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ DEFAULT now()
);

**SESSIONS TABLE (CLEAN ZERO TRUST MODEL)**
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- session identity pointer (JWT jti)
    jti UUID NOT NULL UNIQUE,

    -- refresh token (hashed, never raw stored)
    refresh_token_hash TEXT NOT NULL,

    -- lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,

    last_used_at TIMESTAMPTZ,
    is_revoked BOOLEAN NOT NULL DEFAULT false,

    -- rotation chain (auditability)
    rotated_from UUID REFERENCES sessions(id) ON DELETE SET NULL,

    -- device binding (STRICT MODEL - enforced by your extractor)
    ip_address INET NOT NULL,
    user_agent TEXT NOT NULL,
    device_name TEXT NOT NULL

);

INDEXES

-- fast session lookup
CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- fast auth validation
CREATE INDEX idx_sessions_jti ON sessions(jti);

-- refresh flow
CREATE INDEX idx_sessions_refresh_token_hash ON sessions(refresh_token_hash);

-- expiry cleanup jobs
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);


CONSTRAINTS

-- session must always be valid in time window
ALTER TABLE sessions
ADD CONSTRAINT sessions_valid_time
CHECK (expires_at > created_at);


ALTER TABLE sessions
ADD CONSTRAINT sessions_max_lifetime
CHECK (expires_at <= created_at + interval '7 days');


ROTATION SAFETY
FOREIGN KEY (rotated_from) REFERENCES sessions(id) ON DELETE SET NULL

  
  
  
  */