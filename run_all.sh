#!/bin/bash

for dir in /Images; do
    if [ ! -d "$dir" ]; then
        echo "ERROR: Required mount point $dir not found..."
        exit 1
    fi
done

cd /app/TagService
uvicorn main:app --host 0.0.0.0 --port 8000 --log-level warning &

cd /app/PixivDownloader
python main.py &

/usr/local/bin/tag_api &

sleep 30s

/usr/local/bin/tag_manager &


wait
