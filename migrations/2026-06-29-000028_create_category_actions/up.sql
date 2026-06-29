CREATE TABLE public.category_actions (
    category_id integer NOT NULL,
    person_id integer NOT NULL,
    followed_at timestamp with time zone,
    follow_state public.category_follower_state,
    follow_approver_id integer,
    blocked_at timestamp with time zone,
    became_moderator_at timestamp with time zone,
    received_ban_at timestamp with time zone,
    ban_expires_at timestamp with time zone,
    CONSTRAINT category_actions_check_followed CHECK ((((followed_at IS NULL) = (follow_state IS NULL)) AND (NOT ((followed_at IS NULL) AND (follow_approver_id IS NOT NULL))))),
    CONSTRAINT category_actions_check_received_ban CHECK ((NOT ((received_ban_at IS NULL) AND (ban_expires_at IS NOT NULL))))
);

ALTER TABLE ONLY public.category_actions
    ADD CONSTRAINT category_actions_pkey PRIMARY KEY (person_id, category_id);

CREATE INDEX idx_category_actions_became_moderator ON public.category_actions USING btree (became_moderator_at) WHERE (became_moderator_at IS NOT NULL);

CREATE INDEX idx_category_actions_became_moderator_not_null ON public.category_actions USING btree (person_id, category_id) WHERE (became_moderator_at IS NOT NULL);

CREATE INDEX idx_category_actions_blocked_not_null ON public.category_actions USING btree (person_id, category_id) WHERE (blocked_at IS NOT NULL);

CREATE INDEX idx_category_actions_category ON public.category_actions USING btree (category_id);

CREATE INDEX idx_category_actions_followed ON public.category_actions USING btree (followed_at) WHERE (followed_at IS NOT NULL);

CREATE INDEX idx_category_actions_followed_not_null ON public.category_actions USING btree (person_id, category_id) WHERE ((followed_at IS NOT NULL) OR (follow_state IS NOT NULL));

CREATE INDEX idx_category_actions_received_ban_not_null ON public.category_actions USING btree (person_id, category_id) WHERE (received_ban_at IS NOT NULL);
