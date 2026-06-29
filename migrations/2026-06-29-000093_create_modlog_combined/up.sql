CREATE TABLE public.modlog_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    admin_allow_instance_id integer,
    admin_block_instance_id integer,
    admin_purge_proposal_id integer,
    admin_purge_category_id integer,
    admin_purge_person_id integer,
    admin_purge_post_id integer,
    mod_add_id integer,
    mod_add_category_id integer,
    mod_ban_id integer,
    mod_ban_from_category_id integer,
    mod_feature_post_id integer,
    mod_lock_post_id integer,
    mod_remove_proposal_id integer,
    mod_remove_category_id integer,
    mod_remove_post_id integer,
    mod_transfer_category_id integer,
    mod_change_category_visibility_id integer,
    CONSTRAINT modlog_combined_check CHECK ((num_nonnulls(admin_allow_instance_id, admin_block_instance_id, admin_purge_proposal_id, admin_purge_category_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_category_id, mod_ban_id, mod_ban_from_category_id, mod_feature_post_id, mod_change_category_visibility_id, mod_lock_post_id, mod_remove_proposal_id, mod_remove_category_id, mod_remove_post_id, mod_transfer_category_id) = 1))
);

CREATE SEQUENCE public.modlog_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.modlog_combined_id_seq OWNED BY public.modlog_combined.id;

ALTER TABLE ONLY public.modlog_combined ALTER COLUMN id SET DEFAULT nextval('public.modlog_combined_id_seq'::regclass);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_allow_instance_id_key UNIQUE (admin_allow_instance_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_block_instance_id_key UNIQUE (admin_block_instance_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_category_id_key UNIQUE (admin_purge_category_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_person_id_key UNIQUE (admin_purge_person_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_post_id_key UNIQUE (admin_purge_post_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_proposal_id_key UNIQUE (admin_purge_proposal_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_category_id_key UNIQUE (mod_add_category_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_id_key UNIQUE (mod_add_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_from_category_id_key UNIQUE (mod_ban_from_category_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_id_key UNIQUE (mod_ban_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_feature_post_id_key UNIQUE (mod_feature_post_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_key UNIQUE (mod_lock_post_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_category_id_key UNIQUE (mod_remove_category_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_key UNIQUE (mod_remove_post_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_proposal_id_key UNIQUE (mod_remove_proposal_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_transfer_category_id_key UNIQUE (mod_transfer_category_id);

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_pkey PRIMARY KEY (id);

CREATE INDEX idx_modlog_combined_published ON public.modlog_combined USING btree (published_at DESC, id DESC);
