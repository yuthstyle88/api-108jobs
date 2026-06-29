CREATE TABLE public.captcha_answer (
    uuid uuid DEFAULT gen_random_uuid() NOT NULL,
    answer text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER TABLE ONLY public.captcha_answer
    ADD CONSTRAINT captcha_answer_pkey PRIMARY KEY (uuid);
