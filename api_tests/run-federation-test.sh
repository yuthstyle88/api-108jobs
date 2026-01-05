#!/usr/bin/env bash
set -e

export app_108jobs_DATABASE_URL=postgres://app_108jobs:password@localhost:5432
pushd ..
cargo build
rm target/app_108jobs_server || true
cp target/debug/app_108jobs_server target/app_108jobs_server
killall -s1 app_108jobs_server || true
./api_tests/prepare-drone-federation-test.sh
popd

pnpm i
pnpm api-test || true

killall -s1 app_108jobs_server || true
killall -s1 pict-rs || true
for INSTANCE in app_108jobs_alpha app_108jobs_beta app_108jobs_gamma app_108jobs_delta app_108jobs_epsilon; do
  psql "$app_108jobs_DATABASE_URL" -c "DROP DATABASE $INSTANCE"
done
rm -r /tmp/pictrs
