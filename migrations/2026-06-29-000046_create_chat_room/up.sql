CREATE TABLE public.chat_room (
    id character varying NOT NULL,
    room_name character varying,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    post_id integer,
    current_proposal_id integer,
    last_message_id character varying,
    last_message_at timestamp with time zone,
    serial_id bigint NOT NULL,
    CONSTRAINT chat_room_last_message_id_check CHECK (((last_message_id IS NULL) OR (char_length((last_message_id)::text) > 0)))
);

CREATE SEQUENCE public.chat_room_serial_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.chat_room_serial_id_seq OWNED BY public.chat_room.serial_id;

ALTER TABLE ONLY public.chat_room ALTER COLUMN serial_id SET DEFAULT nextval('public.chat_room_serial_id_seq'::regclass);

ALTER TABLE ONLY public.chat_room
    ADD CONSTRAINT chat_room_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.chat_room
    ADD CONSTRAINT chat_room_serial_id_unique UNIQUE (serial_id);

CREATE INDEX idx_chat_room_current_proposal_id ON public.chat_room USING btree (current_proposal_id);
