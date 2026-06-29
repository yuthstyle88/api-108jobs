CREATE TABLE public.last_reads (
    local_user_id integer NOT NULL,
    room_id character varying NOT NULL,
    last_read_msg_id character varying,
    updated_at timestamp with time zone,
    CONSTRAINT last_reads_last_read_msg_id_check CHECK (((last_read_msg_id IS NULL) OR (char_length((last_read_msg_id)::text) > 0)))
);

ALTER TABLE ONLY public.last_reads
    ADD CONSTRAINT last_reads_pkey PRIMARY KEY (local_user_id, room_id);

CREATE INDEX idx_last_reads_last_read_msg_id ON public.last_reads USING btree (last_read_msg_id);
