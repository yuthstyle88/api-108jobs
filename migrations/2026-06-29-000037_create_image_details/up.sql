CREATE TABLE public.image_details (
    link text NOT NULL,
    width integer NOT NULL,
    height integer NOT NULL,
    content_type text NOT NULL,
    blurhash character varying(50)
);

ALTER TABLE ONLY public.image_details
    ADD CONSTRAINT image_details_pkey PRIMARY KEY (link);
