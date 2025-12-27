#!/bin/sh
set -e

echo "Fetching random Wikipedia article..."

WIKI_URL=$(curl -sI https://en.wikipedia.org/wiki/Special:Random | grep -i "^location:" | awk '{print $2}' | tr -d '\r')

if [ -z "$WIKI_URL" ]; then
    echo "Error: Could not fetch random Wikipedia URL"
    exit 1
fi

echo "Random Wikipedia article: $WIKI_URL"

TODO_CONTENT="Read $WIKI_URL"

echo "Creating todo: $TODO_CONTENT"

curl -X POST "${TODO_BACKEND_URL}/todos" \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"${TODO_CONTENT}\"}" \
    -v

echo ""
echo "Successfully created todo!"