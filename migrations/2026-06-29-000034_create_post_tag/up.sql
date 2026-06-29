CREATE TABLE public.post_tag (
    post_id integer NOT NULL,
    tag_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.post_tag
    ADD CONSTRAINT post_tag_pkey PRIMARY KEY (post_id, tag_id);
