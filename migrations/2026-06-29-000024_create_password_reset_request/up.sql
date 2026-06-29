CREATE TABLE public.password_reset_request (
    id integer NOT NULL,
    token text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    local_user_id integer NOT NULL
);

CREATE SEQUENCE public.password_reset_request_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.password_reset_request_id_seq OWNED BY public.password_reset_request.id;

ALTER TABLE ONLY public.password_reset_request ALTER COLUMN id SET DEFAULT nextval('public.password_reset_request_id_seq'::regclass);

ALTER TABLE ONLY public.password_reset_request
    ADD CONSTRAINT password_reset_request_pkey PRIMARY KEY (id);
