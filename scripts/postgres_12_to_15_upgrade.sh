#!/bin/sh
set -e

echo "Updating docker-compose to use postgres version 16."
sudo sed -i "s/image: .*postgres:.*/image: pgautoupgrade\/pgautoupgrade:16-alpine/" ./docker-compose.yml

echo "Starting up app_108jobs..."
sudo docker-compose up -d
