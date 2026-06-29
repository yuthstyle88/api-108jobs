CREATE TABLE public.local_image (
    pictrs_alias text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    person_id integer,
    thumbnail_for_post_id integer
);

ALTER TABLE ONLY public.local_image
    ADD CONSTRAINT image_upload_pkey PRIMARY KEY (pictrs_alias);

CREATE INDEX idx_image_upload_person_id ON public.local_image USING btree (person_id);
