ALTER TABLE ONLY public.skills
    ADD CONSTRAINT skills_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.certificates
    ADD CONSTRAINT certificates_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;
