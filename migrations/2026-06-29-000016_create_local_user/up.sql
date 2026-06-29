CREATE TABLE public.local_user (
    id integer NOT NULL,
    person_id integer NOT NULL,
    password_encrypted text,
    email text,
    self_promotion boolean DEFAULT false NOT NULL,
    theme text DEFAULT 'browser'::text NOT NULL,
    default_post_sort_type public.post_sort_type_enum DEFAULT 'Active'::public.post_sort_type_enum NOT NULL,
    default_listing_type public.listing_type_enum DEFAULT 'Local'::public.listing_type_enum NOT NULL,
    interface_language character varying(20) DEFAULT 'browser'::character varying NOT NULL,
    show_avatars boolean DEFAULT true NOT NULL,
    send_notifications_to_email boolean DEFAULT false NOT NULL,
    show_bot_accounts boolean DEFAULT true NOT NULL,
    show_read_posts boolean DEFAULT true NOT NULL,
    email_verified boolean DEFAULT false NOT NULL,
    accepted_application boolean DEFAULT false NOT NULL,
    totp_2fa_secret text,
    open_links_in_new_tab boolean DEFAULT false NOT NULL,
    blur_self_promotion boolean DEFAULT true NOT NULL,
    infinite_scroll_enabled boolean DEFAULT false NOT NULL,
    admin boolean DEFAULT false NOT NULL,
    post_listing_mode public.post_listing_mode_enum DEFAULT 'List'::public.post_listing_mode_enum NOT NULL,
    totp_2fa_enabled boolean DEFAULT false NOT NULL,
    enable_keyboard_navigation boolean DEFAULT false NOT NULL,
    enable_animated_images boolean DEFAULT true NOT NULL,
    collapse_bot_proposals boolean DEFAULT false NOT NULL,
    default_proposal_sort_type public.proposal_sort_type_enum DEFAULT 'Hot'::public.proposal_sort_type_enum NOT NULL,
    auto_mark_fetched_posts_as_read boolean DEFAULT false NOT NULL,
    last_donation_notification_at timestamp with time zone DEFAULT (now() - (random() * '1 year'::interval)) NOT NULL,
    hide_media boolean DEFAULT false NOT NULL,
    default_post_time_range_seconds integer,
    show_score boolean DEFAULT false NOT NULL,
    show_upvotes boolean DEFAULT true NOT NULL,
    show_downvotes public.vote_show_enum DEFAULT 'Show'::public.vote_show_enum NOT NULL,
    show_upvote_percentage boolean DEFAULT false NOT NULL,
    show_person_votes boolean DEFAULT true NOT NULL,
    accepted_terms boolean DEFAULT false NOT NULL,
    secure_chat_enabled boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.local_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.local_user_id_seq OWNED BY public.local_user.id;

ALTER TABLE ONLY public.local_user ALTER COLUMN id SET DEFAULT nextval('public.local_user_id_seq'::regclass);

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_email_key UNIQUE (email);

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_person_id_key UNIQUE (person_id);

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_pkey PRIMARY KEY (id);
