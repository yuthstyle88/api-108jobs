CREATE TABLE public.site_language (
    site_id integer NOT NULL,
    language_id integer NOT NULL
);

ALTER TABLE ONLY public.site_language
    ADD CONSTRAINT site_language_pkey PRIMARY KEY (site_id, language_id);
