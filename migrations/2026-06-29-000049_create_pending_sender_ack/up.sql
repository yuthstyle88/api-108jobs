CREATE TABLE public.pending_sender_ack (
    id bigint NOT NULL,
    room_id text NOT NULL,
    sender_id integer NOT NULL,
    client_id uuid NOT NULL,
    server_id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.pending_sender_ack_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.pending_sender_ack_id_seq OWNED BY public.pending_sender_ack.id;

ALTER TABLE ONLY public.pending_sender_ack ALTER COLUMN id SET DEFAULT nextval('public.pending_sender_ack_id_seq'::regclass);

ALTER TABLE ONLY public.pending_sender_ack
    ADD CONSTRAINT pending_sender_ack_pkey PRIMARY KEY (id);

CREATE INDEX idx_pending_sender_ack_created_at ON public.pending_sender_ack USING btree (created_at);

CREATE INDEX idx_pending_sender_ack_stream ON public.pending_sender_ack USING btree (room_id, sender_id);
