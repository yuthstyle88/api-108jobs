CREATE TABLE public.ride_meter_snapshot (
    id integer NOT NULL,
    ride_session_id integer NOT NULL,
    elapsed_minutes integer NOT NULL,
    distance_km double precision NOT NULL,
    current_price_coin integer NOT NULL,
    base_fare_coin integer NOT NULL,
    time_charge_coin integer NOT NULL,
    distance_charge_coin integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.ride_meter_snapshot_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.ride_meter_snapshot_id_seq OWNED BY public.ride_meter_snapshot.id;

ALTER TABLE ONLY public.ride_meter_snapshot ALTER COLUMN id SET DEFAULT nextval('public.ride_meter_snapshot_id_seq'::regclass);

ALTER TABLE ONLY public.ride_meter_snapshot
    ADD CONSTRAINT ride_meter_snapshot_pkey PRIMARY KEY (id);

CREATE INDEX idx_ride_meter_snapshot_session ON public.ride_meter_snapshot USING btree (ride_session_id, created_at);
