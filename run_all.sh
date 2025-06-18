#!/bin/bash

for dir in /Images; do
    if [ ! -d "$dir" ]; then
        echo "ERROR: Required mount point $dir not found..."
        exit 1
    fi
done

# Start TagService (FastAPI)
cd /app/TagService
fastapi run main.py &

sleep 20s

# Start TagManager
/usr/local/bin/tag_manager &

# Start TagApi
/usr/local/bin/tag_api &



# Wait for all to finish (block)
wait
