CREATE TABLE public.remote_image (
    link text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.remote_image
    ADD CONSTRAINT remote_image_pkey PRIMARY KEY (link);
