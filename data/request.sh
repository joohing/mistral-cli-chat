#!/bin/bash

ct="Content-Type: application/json" 
data=`cat data/sample.json`

echo "Running request..."
curl --header 'Content-Type: application/json' \
     --data "${data}" \
     -H 'Accept: application/json' \
     -H "Authorization: Bearer ${MISTRAL_API_KEY}" \
     --location "https://api.mistral.ai/v1/chat/completions" | jq
