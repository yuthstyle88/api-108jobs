CREATE TABLE public.delivery_details (
    id integer NOT NULL,
    post_id integer NOT NULL,
    pickup_address text NOT NULL,
    pickup_lat double precision,
    pickup_lng double precision,
    dropoff_address text NOT NULL,
    dropoff_lat double precision,
    dropoff_lng double precision,
    package_description text,
    package_weight_kg double precision,
    package_size character varying,
    fragile boolean DEFAULT false NOT NULL,
    requires_signature boolean DEFAULT false NOT NULL,
    vehicle_required public.vehicle_type,
    latest_pickup_at timestamp with time zone,
    latest_dropoff_at timestamp with time zone,
    sender_name character varying,
    sender_phone character varying,
    receiver_name character varying,
    receiver_phone character varying,
    cash_on_delivery boolean DEFAULT false NOT NULL,
    cod_amount double precision,
    status public.trip_status DEFAULT 'Pending'::public.trip_status NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    cancellation_reason text,
    assigned_rider_id integer,
    assigned_at timestamp with time zone,
    assigned_by_person_id integer,
    linked_proposal_id integer,
    delivery_fee integer DEFAULT 0 NOT NULL,
    employer_confirmed_at timestamp with time zone,
    employer_wallet_transaction_id integer,
    rider_wallet_transaction_id integer
);

CREATE SEQUENCE public.delivery_details_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.delivery_details_id_seq OWNED BY public.delivery_details.id;

ALTER TABLE ONLY public.delivery_details ALTER COLUMN id SET DEFAULT nextval('public.delivery_details_id_seq'::regclass);

ALTER TABLE ONLY public.delivery_details
    ADD CONSTRAINT delivery_details_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.delivery_details
    ADD CONSTRAINT delivery_details_post_id_key UNIQUE (post_id);

CREATE INDEX idx_delivery_details_assigned_rider ON public.delivery_details USING btree (assigned_rider_id) WHERE (status = 'Assigned'::public.trip_status);

CREATE INDEX idx_delivery_details_post_id ON public.delivery_details USING btree (post_id);

CREATE INDEX idx_delivery_details_status ON public.delivery_details USING btree (status);
