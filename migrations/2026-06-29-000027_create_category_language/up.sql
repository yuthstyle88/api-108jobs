CREATE TABLE public.category_language (
    category_id integer NOT NULL,
    language_id integer NOT NULL
);

ALTER TABLE ONLY public.category_language
    ADD CONSTRAINT category_language_pkey PRIMARY KEY (category_id, language_id);
