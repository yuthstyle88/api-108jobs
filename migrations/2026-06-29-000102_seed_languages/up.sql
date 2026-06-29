-- Minimum language seed required for 108Jobs.
-- id=0 (und) is the default for post.language_id; id=1 (en), id=2 (th), id=3 (vi)
-- are used in user-language preferences and tests.
INSERT INTO public.language (id, code, name) VALUES
  (0, 'und', 'Undetermined'),
  (1, 'en', 'English'),
  (2, 'th', 'Thai'),
  (3, 'vi', 'Vietnamese');
