CREATE TABLE public.ride_session (
    id integer NOT NULL,
    post_id integer NOT NULL,
    rider_id integer,
    employer_id integer NOT NULL,
    pricing_config_id integer,
    pickup_address text NOT NULL,
    pickup_lat double precision,
    pickup_lng double precision,
    dropoff_address text NOT NULL,
    dropoff_lat double precision,
    dropoff_lng double precision,
    pickup_note text,
    payment_method public.payment_method NOT NULL,
    payment_status character varying(20) DEFAULT 'pending'::character varying,
    status public.trip_status DEFAULT 'Pending'::public.trip_status NOT NULL,
    requested_at timestamp with time zone DEFAULT now() NOT NULL,
    rider_assigned_at timestamp with time zone,
    rider_confirmed_at timestamp with time zone,
    arrived_at_pickup_at timestamp with time zone,
    ride_started_at timestamp with time zone,
    ride_completed_at timestamp with time zone,
    current_price_coin integer DEFAULT 0,
    total_distance_km double precision,
    total_duration_minutes integer,
    final_price_coin integer,
    base_fare_applied_coin integer,
    time_charge_applied_coin integer,
    distance_charge_applied_coin integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    passenger_name text,
    passenger_phone text,
    cancellation_reason text
);

CREATE SEQUENCE public.ride_session_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.ride_session_id_seq OWNED BY public.ride_session.id;

ALTER TABLE ONLY public.ride_session ALTER COLUMN id SET DEFAULT nextval('public.ride_session_id_seq'::regclass);

ALTER TABLE ONLY public.ride_session
    ADD CONSTRAINT ride_session_pkey PRIMARY KEY (id);

CREATE INDEX idx_ride_session_post ON public.ride_session USING btree (post_id);

CREATE INDEX idx_ride_session_rider ON public.ride_session USING btree (rider_id);

CREATE INDEX idx_ride_session_status ON public.ride_session USING btree (status);
