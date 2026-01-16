#!/usr/bin/env bash
set -e

pushd ../../app_108jobs-website
git fetch weblate
git merge weblate/main
git push
popd

git submodule update --remote
git add ../crates/utils/website
git commit -m"Updating translations."
git push
