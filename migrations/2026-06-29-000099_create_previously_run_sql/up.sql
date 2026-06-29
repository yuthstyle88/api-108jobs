CREATE TABLE public.previously_run_sql (
    id boolean NOT NULL,
    content text NOT NULL
);

ALTER TABLE ONLY public.previously_run_sql
    ADD CONSTRAINT previously_run_sql_pkey PRIMARY KEY (id);

-- Singleton row required by run_replaceable_schema: UPDATE expects exactly 1 row.
INSERT INTO public.previously_run_sql (id, content) VALUES (true, '');
