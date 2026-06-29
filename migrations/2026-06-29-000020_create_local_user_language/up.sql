CREATE TABLE public.local_user_language (
    local_user_id integer NOT NULL,
    language_id integer NOT NULL
);

ALTER TABLE ONLY public.local_user_language
    ADD CONSTRAINT local_user_language_pkey PRIMARY KEY (local_user_id, language_id);
