CREATE TABLE public.local_user_keyword_block (
    local_user_id integer NOT NULL,
    keyword character varying(50) NOT NULL
);

ALTER TABLE ONLY public.local_user_keyword_block
    ADD CONSTRAINT local_user_keyword_block_pkey PRIMARY KEY (local_user_id, keyword);
