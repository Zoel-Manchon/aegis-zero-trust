--
-- PostgreSQL database dump
--

\restrict L9gAvYHEe9an2w9N8jy4m2z8E8X9sdigbTEG8vZbaaddshpqmgl2hKir6NAg4Ua

-- Dumped from database version 17.11 (Debian 17.11-1.pgdg13+2)
-- Dumped by pg_dump version 17.11 (Debian 17.11-1.pgdg13+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON SCHEMA public IS '';


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: session_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.session_status AS ENUM (
    'active',
    'rotated',
    'revoked'
);


--
-- Name: user_role; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.user_role AS ENUM (
    'user',
    'admin'
);


--
-- Name: soc_notify_security_event(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.soc_notify_security_event() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM pg_notify('soc_events', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: email_verification_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.email_verification_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id bigint NOT NULL,
    token_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone
);


--
-- Name: mfa_backup_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mfa_backup_codes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id bigint NOT NULL,
    code_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    used_at timestamp with time zone
);


--
-- Name: passkey_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.passkey_credentials (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    credential_id text NOT NULL,
    public_key_cose bytea NOT NULL,
    sign_count bigint DEFAULT 0 NOT NULL,
    friendly_name text,
    transports text[] DEFAULT '{}'::text[] NOT NULL,
    aaguid text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone,
    revoked_at timestamp with time zone
);


--
-- Name: TABLE passkey_credentials; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.passkey_credentials IS 'WebAuthn/passkey public-key credentials. Private keys never touch the server.';


--
-- Name: passkey_credentials_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.passkey_credentials_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: passkey_credentials_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.passkey_credentials_id_seq OWNED BY public.passkey_credentials.id;


--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.password_reset_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id bigint NOT NULL,
    token_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone
);


--
-- Name: security_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.security_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id bigint,
    event_type text NOT NULL,
    severity text NOT NULL,
    ip_address inet,
    user_agent text,
    session_id uuid,
    jti uuid,
    family_id uuid,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    seq bigint,
    prev_hash text,
    event_hash text
);


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    family_id uuid NOT NULL,
    user_id bigint NOT NULL,
    refresh_token_hash text NOT NULL,
    jti uuid NOT NULL,
    device_name text NOT NULL,
    ip_address inet NOT NULL,
    user_agent text NOT NULL,
    status public.session_status DEFAULT 'active'::public.session_status NOT NULL,
    rotated_from uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    last_used_at timestamp with time zone
);


--
-- Name: user_mfa; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_mfa (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id bigint NOT NULL,
    secret text NOT NULL,
    enabled boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    verified_at timestamp with time zone,
    disabled_at timestamp with time zone,
    last_used_step bigint
);


--
-- Name: COLUMN user_mfa.secret; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_mfa.secret IS 'TOTP seed. Sealed rows carry the aegis.v1.<wrapped DEK>.<nonce>.<ciphertext> envelope (AES-256-GCM, key wrapped by Vault transit or a local KEK). Rows without that prefix are legacy plaintext seeds and are read as-is until re-enrolment.';


--
-- Name: COLUMN user_mfa.last_used_step; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_mfa.last_used_step IS 'Highest TOTP step consumed; codes from this step or earlier are rejected as replays.';


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    email text NOT NULL,
    password text NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    user_role public.user_role DEFAULT 'user'::public.user_role NOT NULL,
    email_verified_at timestamp with time zone
);


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: passkey_credentials id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.passkey_credentials ALTER COLUMN id SET DEFAULT nextval('public.passkey_credentials_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: email_verification_tokens email_verification_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT email_verification_tokens_pkey PRIMARY KEY (id);


--
-- Name: email_verification_tokens email_verification_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT email_verification_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: mfa_backup_codes mfa_backup_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mfa_backup_codes
    ADD CONSTRAINT mfa_backup_codes_pkey PRIMARY KEY (id);


--
-- Name: passkey_credentials passkey_credentials_credential_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.passkey_credentials
    ADD CONSTRAINT passkey_credentials_credential_id_key UNIQUE (credential_id);


--
-- Name: passkey_credentials passkey_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.passkey_credentials
    ADD CONSTRAINT passkey_credentials_pkey PRIMARY KEY (id);


--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (id);


--
-- Name: password_reset_tokens password_reset_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: security_events security_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_events
    ADD CONSTRAINT security_events_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_jti_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_jti_key UNIQUE (jti);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: users unique_email; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT unique_email UNIQUE (email);


--
-- Name: user_mfa user_mfa_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_mfa
    ADD CONSTRAINT user_mfa_pkey PRIMARY KEY (id);


--
-- Name: user_mfa user_mfa_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_mfa
    ADD CONSTRAINT user_mfa_user_id_key UNIQUE (user_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: idx_evt_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_evt_token_hash ON public.email_verification_tokens USING btree (token_hash);


--
-- Name: idx_evt_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_evt_user_id ON public.email_verification_tokens USING btree (user_id);


--
-- Name: idx_mfa_backup_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mfa_backup_user_id ON public.mfa_backup_codes USING btree (user_id);


--
-- Name: idx_passkey_credentials_user_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_passkey_credentials_user_active ON public.passkey_credentials USING btree (user_id) WHERE (revoked_at IS NULL);


--
-- Name: idx_prt_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_prt_token_hash ON public.password_reset_tokens USING btree (token_hash);


--
-- Name: idx_prt_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_prt_user_id ON public.password_reset_tokens USING btree (user_id);


--
-- Name: idx_security_events_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_events_created_at ON public.security_events USING btree (created_at);


--
-- Name: idx_security_events_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_events_event_type ON public.security_events USING btree (event_type);


--
-- Name: idx_security_events_seq; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_events_seq ON public.security_events USING btree (seq);


--
-- Name: idx_security_events_severity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_events_severity ON public.security_events USING btree (severity);


--
-- Name: idx_security_events_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_events_user_id ON public.security_events USING btree (user_id);


--
-- Name: idx_sessions_family; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_family ON public.sessions USING btree (family_id);


--
-- Name: idx_sessions_jti; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_jti ON public.sessions USING btree (jti);


--
-- Name: idx_sessions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_user ON public.sessions USING btree (user_id);


--
-- Name: idx_user_mfa_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_mfa_user_id ON public.user_mfa USING btree (user_id);


--
-- Name: one_active_per_family; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX one_active_per_family ON public.sessions USING btree (family_id) WHERE (status = 'active'::public.session_status);


--
-- Name: uq_mfa_backup_user_code; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_mfa_backup_user_code ON public.mfa_backup_codes USING btree (user_id, code_hash);


--
-- Name: uq_security_events_event_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_security_events_event_hash ON public.security_events USING btree (event_hash) WHERE (event_hash IS NOT NULL);


--
-- Name: users_lower_email_unique_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX users_lower_email_unique_idx ON public.users USING btree (lower(email));


--
-- Name: security_events trg_soc_notify_security_event; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_soc_notify_security_event AFTER INSERT ON public.security_events FOR EACH ROW EXECUTE FUNCTION public.soc_notify_security_event();


--
-- Name: email_verification_tokens email_verification_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT email_verification_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: mfa_backup_codes mfa_backup_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mfa_backup_codes
    ADD CONSTRAINT mfa_backup_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: passkey_credentials passkey_credentials_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.passkey_credentials
    ADD CONSTRAINT passkey_credentials_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: password_reset_tokens password_reset_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: security_events security_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_events
    ADD CONSTRAINT security_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: sessions sessions_rotated_from_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_rotated_from_fkey FOREIGN KEY (rotated_from) REFERENCES public.sessions(id);


--
-- Name: user_mfa user_mfa_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_mfa
    ADD CONSTRAINT user_mfa_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

\unrestrict L9gAvYHEe9an2w9N8jy4m2z8E8X9sdigbTEG8vZbaaddshpqmgl2hKir6NAg4Ua

