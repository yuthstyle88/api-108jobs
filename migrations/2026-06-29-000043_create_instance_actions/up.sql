CREATE TABLE public.instance_actions (
    person_id integer NOT NULL,
    instance_id integer NOT NULL,
    blocked_at timestamp with time zone,
    received_ban_at timestamp with time zone,
    ban_expires_at timestamp with time zone
);

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_pkey PRIMARY KEY (person_id, instance_id);

CREATE INDEX idx_instance_actions_blocked_not_null ON public.instance_actions USING btree (person_id, instance_id) WHERE (blocked_at IS NOT NULL);
