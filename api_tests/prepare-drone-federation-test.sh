#!/usr/bin/env bash
# IMPORTANT NOTE: this script does not use the normal app_108jobs_DATABASE_URL format
#   it is expected that this script is called by run-federation-test.sh script.
set -e

if [ -z "$app_108jobs_LOG_LEVEL" ]; then
  app_108jobs_LOG_LEVEL=info
fi

export RUST_BACKTRACE=1
export RUST_LOG="warn,app_108jobs_server=$app_108jobs_LOG_LEVEL,app_108jobs_federate=$app_108jobs_LOG_LEVEL,app_108jobs_api=$app_108jobs_LOG_LEVEL,app_108jobs_api_common=$app_108jobs_LOG_LEVEL,app_108jobs_api_crud=$app_108jobs_LOG_LEVEL,app_108jobs_apub=$app_108jobs_LOG_LEVEL,app_108jobs_db_schema=$app_108jobs_LOG_LEVEL,app_108jobs_db_views=$app_108jobs_LOG_LEVEL,app_108jobs_routes=$app_108jobs_LOG_LEVEL,app_108jobs_utils=$app_108jobs_LOG_LEVEL,app_108jobs_websocket=$app_108jobs_LOG_LEVEL"

export app_108jobs_TEST_FAST_FEDERATION=1 # by default, the persistent federation queue has delays in the scale of 30s-5min

PICTRS_PATH="api_tests/pict-rs"
PICTRS_EXPECTED_HASH="7f7ac2a45ef9b13403ee139b7512135be6b060ff2f6460e0c800e18e1b49d2fd  api_tests/pict-rs"

# Pictrs setup. Download file with hash check and up to 3 retries.
if [ ! -f "$PICTRS_PATH" ]; then
  count=0
  while [ ! -f "$PICTRS_PATH" ] && [ "$count" -lt 3 ]; do
    # This one sometimes goes down
    curl "https://git.asonix.dog/asonix/pict-rs/releases/download/v0.5.17-pre.9/pict-rs-linux-amd64" -o "$PICTRS_PATH"
    # curl "https://codeberg.org/asonix/pict-rs/releases/download/v0.5.5/pict-rs-linux-amd64" -o "$PICTRS_PATH"
    PICTRS_HASH=$(sha256sum "$PICTRS_PATH")
    if [[ "$PICTRS_HASH" != "$PICTRS_EXPECTED_HASH" ]]; then
      echo "Pictrs binary hash mismatch, was $PICTRS_HASH but expected $PICTRS_EXPECTED_HASH"
      rm "$PICTRS_PATH"
      let count=count+1
    fi
  done
  chmod +x "$PICTRS_PATH"
fi

./api_tests/pict-rs \
  run -a 0.0.0.0:8080 \
  --danger-dummy-mode \
  --api-key "my-pictrs-key" \
  filesystem -p /tmp/pictrs/files \
  sled -p /tmp/pictrs/sled-repo 2>&1 &

for INSTANCE in app_108jobs_alpha app_108jobs_beta app_108jobs_gamma app_108jobs_delta app_108jobs_epsilon; do
  echo "DB URL: ${app_108jobs_DATABASE_URL} INSTANCE: $INSTANCE"
  psql "${app_108jobs_DATABASE_URL}/app_108jobs" -c "DROP DATABASE IF EXISTS $INSTANCE"
  echo "create database"
  psql "${app_108jobs_DATABASE_URL}/app_108jobs" -c "CREATE DATABASE $INSTANCE"
done

if [ -z "$DO_WRITE_HOSTS_FILE" ]; then
  if ! grep -q app_108jobs-alpha /etc/hosts; then
    echo "Please add the following to your /etc/hosts file, then press enter:

      127.0.0.1       app_108jobs-alpha
      127.0.0.1       app_108jobs-beta
      127.0.0.1       app_108jobs-gamma
      127.0.0.1       app_108jobs-delta
      127.0.0.1       app_108jobs-epsilon"
    read -p ""
  fi
else
  for INSTANCE in app_108jobs-alpha app_108jobs-beta app_108jobs-gamma app_108jobs-delta app_108jobs-epsilon; do
    echo "127.0.0.1 $INSTANCE" >>/etc/hosts
  done
fi

echo "$PWD"

LOG_DIR=target/log
mkdir -p $LOG_DIR

echo "start alpha"
app_108jobs_CONFIG_LOCATION=./docker/federation/app_108jobs_alpha.hjson \
  app_108jobs_DATABASE_URL="${app_108jobs_DATABASE_URL}/app_108jobs_alpha" \
  target/app_108jobs_server >$LOG_DIR/app_108jobs_alpha.out 2>&1 &

echo "start beta"
app_108jobs_CONFIG_LOCATION=./docker/federation/app_108jobs_beta.hjson \
  app_108jobs_DATABASE_URL="${app_108jobs_DATABASE_URL}/app_108jobs_beta" \
  target/app_108jobs_server >$LOG_DIR/app_108jobs_beta.out 2>&1 &

echo "start gamma"
app_108jobs_CONFIG_LOCATION=./docker/federation/app_108jobs_gamma.hjson \
  app_108jobs_DATABASE_URL="${app_108jobs_DATABASE_URL}/app_108jobs_gamma" \
  target/app_108jobs_server >$LOG_DIR/app_108jobs_gamma.out 2>&1 &

echo "start delta"
app_108jobs_CONFIG_LOCATION=./docker/federation/app_108jobs_delta.hjson \
  app_108jobs_DATABASE_URL="${app_108jobs_DATABASE_URL}/app_108jobs_delta" \
  target/app_108jobs_server >$LOG_DIR/app_108jobs_delta.out 2>&1 &

echo "start epsilon"
app_108jobs_CONFIG_LOCATION=./docker/federation/app_108jobs_epsilon.hjson \
  app_108jobs_PLUGIN_PATH=api_tests/plugins \
  app_108jobs_DATABASE_URL="${app_108jobs_DATABASE_URL}/app_108jobs_epsilon" \
  target/app_108jobs_server >$LOG_DIR/app_108jobs_epsilon.out 2>&1 &

echo "wait for all instances to start"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'app_108jobs-alpha:8541/api/v4/site')" != "200" ]]; do sleep 1; done
echo "alpha started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'app_108jobs-beta:8551/api/v4/site')" != "200" ]]; do sleep 1; done
echo "beta started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'app_108jobs-gamma:8561/api/v4/site')" != "200" ]]; do sleep 1; done
echo "gamma started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'app_108jobs-delta:8571/api/v4/site')" != "200" ]]; do sleep 1; done
echo "delta started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'app_108jobs-epsilon:8581/api/v4/site')" != "200" ]]; do sleep 1; done
echo "epsilon started. All started"
