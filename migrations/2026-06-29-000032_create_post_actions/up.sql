CREATE TABLE public.post_actions (
    post_id integer NOT NULL,
    person_id integer NOT NULL,
    read_at timestamp with time zone,
    read_proposals_at timestamp with time zone,
    read_proposals_amount bigint,
    saved_at timestamp with time zone,
    liked_at timestamp with time zone,
    like_score smallint,
    hidden_at timestamp with time zone,
    notifications public.post_notifications_mode_enum,
    CONSTRAINT post_actions_check_liked CHECK (((liked_at IS NULL) = (like_score IS NULL))),
    CONSTRAINT post_actions_check_read_comments CHECK (((read_proposals_at IS NULL) = (read_proposals_amount IS NULL)))
);

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_pkey PRIMARY KEY (person_id, post_id);

CREATE INDEX idx_post_actions_hidden_not_null ON public.post_actions USING btree (person_id, post_id) WHERE (hidden_at IS NOT NULL);

CREATE INDEX idx_post_actions_like_score ON public.post_actions USING btree (post_id, like_score, person_id) WHERE (like_score IS NOT NULL);

CREATE INDEX idx_post_actions_liked_not_null ON public.post_actions USING btree (person_id, post_id) WHERE ((liked_at IS NOT NULL) OR (like_score IS NOT NULL));

CREATE INDEX idx_post_actions_on_read_read_not_null ON public.post_actions USING btree (person_id, read_at, post_id) WHERE (read_at IS NOT NULL);

CREATE INDEX idx_post_actions_person ON public.post_actions USING btree (person_id);

CREATE INDEX idx_post_actions_post ON public.post_actions USING btree (post_id);

CREATE INDEX idx_post_actions_read_not_null ON public.post_actions USING btree (person_id, post_id) WHERE (read_at IS NOT NULL);

CREATE INDEX idx_post_actions_read_proposals_not_null ON public.post_actions USING btree (person_id, post_id) WHERE ((read_proposals_at IS NOT NULL) OR (read_proposals_amount IS NOT NULL));

CREATE INDEX idx_post_actions_saved_not_null ON public.post_actions USING btree (person_id, post_id) WHERE (saved_at IS NOT NULL);
