CREATE TABLE public.person_actions (
    target_id integer NOT NULL,
    person_id integer NOT NULL,
    followed_at timestamp with time zone,
    follow_pending boolean,
    blocked_at timestamp with time zone,
    noted_at timestamp with time zone,
    note text,
    voted_at timestamp with time zone,
    upvotes integer,
    downvotes integer,
    CONSTRAINT person_actions_check_followed CHECK (((followed_at IS NULL) = (follow_pending IS NULL)))
);

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_pkey PRIMARY KEY (person_id, target_id);

CREATE INDEX idx_person_actions_blocked_not_null ON public.person_actions USING btree (person_id, target_id) WHERE (blocked_at IS NOT NULL);

CREATE INDEX idx_person_actions_followed_not_null ON public.person_actions USING btree (person_id, target_id) WHERE ((followed_at IS NOT NULL) OR (follow_pending IS NOT NULL));

CREATE INDEX idx_person_actions_person ON public.person_actions USING btree (person_id);

CREATE INDEX idx_person_actions_target ON public.person_actions USING btree (target_id);
