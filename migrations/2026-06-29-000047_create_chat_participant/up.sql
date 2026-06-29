CREATE TABLE public.chat_participant (
    room_id character varying NOT NULL,
    member_id integer NOT NULL,
    joined_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.chat_participant
    ADD CONSTRAINT chat_participant_pkey PRIMARY KEY (room_id, member_id);
