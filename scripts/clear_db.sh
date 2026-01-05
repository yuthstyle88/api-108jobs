#!/usr/bin/env bash

psql -U app_108jobs -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; DROP SCHEMA utils CASCADE;"
