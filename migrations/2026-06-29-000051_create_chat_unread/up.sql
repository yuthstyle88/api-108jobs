CREATE TABLE public.chat_unread (
    local_user_id integer NOT NULL,
    room_id character varying NOT NULL,
    unread_count integer DEFAULT 0 NOT NULL,
    last_message_id character varying,
    last_message_at timestamp with time zone,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chat_unread_last_message_id_check CHECK (((last_message_id IS NULL) OR (char_length((last_message_id)::text) > 0)))
);

ALTER TABLE ONLY public.chat_unread
    ADD CONSTRAINT chat_unread_pkey PRIMARY KEY (local_user_id, room_id);
