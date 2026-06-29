CREATE TABLE public.chat_message (
    id bigint NOT NULL,
    msg_ref_id character varying NOT NULL,
    room_id character varying NOT NULL,
    sender_id integer,
    receiver_id integer,
    content text NOT NULL,
    status smallint DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    sender_ack_confirmed_at timestamp with time zone
);

CREATE SEQUENCE public.chat_message_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.chat_message_id_seq OWNED BY public.chat_message.id;

ALTER TABLE ONLY public.chat_message ALTER COLUMN id SET DEFAULT nextval('public.chat_message_id_seq'::regclass);

ALTER TABLE ONLY public.chat_message
    ADD CONSTRAINT chat_message_msg_ref_id_unique UNIQUE (msg_ref_id);

ALTER TABLE ONLY public.chat_message
    ADD CONSTRAINT chat_message_pkey PRIMARY KEY (id);

CREATE INDEX idx_chat_message_room_id ON public.chat_message USING btree (room_id);
