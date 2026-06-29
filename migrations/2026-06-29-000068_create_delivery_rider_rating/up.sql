CREATE TABLE public.delivery_rider_rating (
    id integer NOT NULL,
    post_id integer NOT NULL,
    employer_id integer NOT NULL,
    rider_id integer NOT NULL,
    rating smallint NOT NULL,
    comment text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    CONSTRAINT delivery_rider_rating_rating_check CHECK (((rating >= 1) AND (rating <= 5)))
);

CREATE SEQUENCE public.delivery_rider_rating_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.delivery_rider_rating_id_seq OWNED BY public.delivery_rider_rating.id;

ALTER TABLE ONLY public.delivery_rider_rating ALTER COLUMN id SET DEFAULT nextval('public.delivery_rider_rating_id_seq'::regclass);

ALTER TABLE ONLY public.delivery_rider_rating
    ADD CONSTRAINT delivery_rider_rating_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.delivery_rider_rating
    ADD CONSTRAINT uq_delivery_rider_rating UNIQUE (post_id, employer_id, rider_id);

CREATE INDEX idx_delivery_rider_rating_employer_id ON public.delivery_rider_rating USING btree (employer_id);

CREATE INDEX idx_delivery_rider_rating_post_id ON public.delivery_rider_rating USING btree (post_id);

CREATE INDEX idx_delivery_rider_rating_rider_id ON public.delivery_rider_rating USING btree (rider_id);
