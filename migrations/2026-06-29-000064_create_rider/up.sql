CREATE TABLE public.rider (
    id integer NOT NULL,
    user_id integer NOT NULL,
    person_id integer NOT NULL,
    vehicle_type public.vehicle_type NOT NULL,
    vehicle_plate_number character varying,
    license_number character varying,
    license_expiry_date timestamp with time zone,
    is_verified boolean DEFAULT false NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    verification_status public.rider_verification_status DEFAULT 'Pending'::public.rider_verification_status NOT NULL,
    rating double precision DEFAULT 0 NOT NULL,
    completed_jobs integer DEFAULT 0 NOT NULL,
    total_jobs integer DEFAULT 0 NOT NULL,
    total_earnings double precision DEFAULT 0 NOT NULL,
    pending_earnings double precision DEFAULT 0 NOT NULL,
    is_online boolean DEFAULT false NOT NULL,
    accepting_jobs boolean DEFAULT true NOT NULL,
    joined_at timestamp with time zone,
    last_active_at timestamp with time zone,
    verified_at timestamp with time zone
);

CREATE SEQUENCE public.rider_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.rider_id_seq OWNED BY public.rider.id;

ALTER TABLE ONLY public.rider ALTER COLUMN id SET DEFAULT nextval('public.rider_id_seq'::regclass);

ALTER TABLE ONLY public.rider
    ADD CONSTRAINT rider_pkey PRIMARY KEY (id);

CREATE INDEX idx_rider_accepting_jobs ON public.rider USING btree (accepting_jobs);

CREATE INDEX idx_rider_is_online ON public.rider USING btree (is_online);

CREATE INDEX idx_rider_person_id ON public.rider USING btree (person_id);

CREATE INDEX idx_rider_user_id ON public.rider USING btree (user_id);
