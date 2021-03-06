#!/bin/bash

set -e
set -o pipefail

while true; do
  TWEET=$(/bin/rand_poly)

  if [ -z "$TWEET" ]; then
    exit 1
  fi

  echo "tweet text will be $TWEET"

  MEDIA_ID=$(
      twurl -H upload.twitter.com -f tmp.png -F media -X POST /1.1/media/upload.json |\
          jq ".media_id_string"
        )

  echo "media id is $MEDIA_ID"
  sleep 1

  twurl -d "media_ids=${MEDIA_ID//\"/}&status=$TWEET" /1.1/statuses/update.json > /dev/null

  echo "posted successfully"

  sleep 5m
done
