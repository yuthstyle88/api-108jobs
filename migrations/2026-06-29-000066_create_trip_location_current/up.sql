CREATE TABLE public.trip_location_current (
    post_id integer NOT NULL,
    rider_id integer NOT NULL,
    lat double precision NOT NULL,
    lng double precision NOT NULL,
    heading double precision,
    speed_kmh double precision,
    accuracy_m double precision,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.trip_location_current
    ADD CONSTRAINT trip_location_current_pkey PRIMARY KEY (post_id);
