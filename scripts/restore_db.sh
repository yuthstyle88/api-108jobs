#!/usr/bin/env bash

psql -U app_108jobs -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
cat docker/app_108jobs_dump_2021-01-29_16_13_40.sqldump | psql -U app_108jobs
psql -U app_108jobs -c "alter user app_108jobs with password 'password'"
