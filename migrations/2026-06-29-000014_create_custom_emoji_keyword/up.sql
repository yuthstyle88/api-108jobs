CREATE TABLE public.custom_emoji_keyword (
    custom_emoji_id integer NOT NULL,
    keyword character varying(128) NOT NULL
);

ALTER TABLE ONLY public.custom_emoji_keyword
    ADD CONSTRAINT custom_emoji_keyword_pkey PRIMARY KEY (custom_emoji_id, keyword);
