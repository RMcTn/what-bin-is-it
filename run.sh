#!/bin/bash
pkill geckodriver
# See https://firefox-source-docs.mozilla.org/testing/geckodriver/Usage.html#Running-Firefox-in-an-container-based-package 
# for why we need to define a profile root
mkdir $HOME/geckodriver-profiles
geckodriver --profile-root $HOME/geckodriver-profiles &

cargo run
# docker run -p 8000:8000 --env-file ./.env what-bin cargo run
