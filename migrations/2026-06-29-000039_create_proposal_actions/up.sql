CREATE TABLE public.proposal_actions (
    person_id integer NOT NULL,
    comment_id integer NOT NULL,
    like_score smallint,
    liked_at timestamp with time zone,
    saved_at timestamp with time zone,
    CONSTRAINT comment_actions_check_liked CHECK (((liked_at IS NULL) = (like_score IS NULL)))
);

ALTER TABLE ONLY public.proposal_actions
    ADD CONSTRAINT proposal_actions_pkey PRIMARY KEY (person_id, comment_id);

CREATE INDEX idx_proposal_actions_like_score ON public.proposal_actions USING btree (comment_id, like_score, person_id) WHERE (like_score IS NOT NULL);

CREATE INDEX idx_proposal_actions_liked_not_null ON public.proposal_actions USING btree (person_id, comment_id) WHERE ((liked_at IS NOT NULL) OR (like_score IS NOT NULL));

CREATE INDEX idx_proposal_actions_proposal ON public.proposal_actions USING btree (comment_id);

CREATE INDEX idx_proposal_actions_saved_not_null ON public.proposal_actions USING btree (person_id, comment_id) WHERE (saved_at IS NOT NULL);
