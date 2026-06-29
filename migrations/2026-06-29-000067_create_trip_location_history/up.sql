CREATE TABLE public.trip_location_history (
    id bigint NOT NULL,
    post_id integer NOT NULL,
    rider_id integer NOT NULL,
    lat double precision NOT NULL,
    lng double precision NOT NULL,
    heading double precision,
    speed_kmh double precision,
    accuracy_m double precision,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.trip_location_history_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.trip_location_history_id_seq OWNED BY public.trip_location_history.id;

ALTER TABLE ONLY public.trip_location_history ALTER COLUMN id SET DEFAULT nextval('public.trip_location_history_id_seq'::regclass);

ALTER TABLE ONLY public.trip_location_history
    ADD CONSTRAINT trip_location_history_pkey PRIMARY KEY (id);

CREATE INDEX idx_trip_location_history_post_time ON public.trip_location_history USING btree (post_id, recorded_at DESC);
