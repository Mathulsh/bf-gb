#!/bin/bash
set -e

# 解析 Redis URL
# INPUT_REDIS_URL format: redis://:password@host:port
REDIS_URL="${INPUT_REDIS_URL:-redis://localhost:6379}"

# 提取密码、主机、端口
if [[ "$REDIS_URL" =~ redis://:(.*)@(.*):(.*) ]]; then
    REDIS_PASSWORD="${BASH_REMATCH[1]}"
    REDIS_HOST="${BASH_REMATCH[2]}"
    REDIS_PORT="${BASH_REMATCH[3]}"
elif [[ "$REDIS_URL" =~ redis://(.*):(.*) ]]; then
    REDIS_HOST="${BASH_REMATCH[1]}"
    REDIS_PORT="${BASH_REMATCH[2]}"
    REDIS_PASSWORD=""
fi

INPUT_QUEUE="${INPUT_QUEUE:-tasks}"
OUTPUT_QUEUE="${OUTPUT_QUEUE:-results}"

echo "Starting bf-gb train..."
echo "Redis: $REDIS_HOST:$REDIS_PORT"
echo "Input queue: $INPUT_QUEUE"
echo "Output queue: $OUTPUT_QUEUE"

exec /usr/local/bin/bf-gb train \
    --data /data/data.csv \
    --redis-host "$REDIS_HOST" \
    --redis-port "$REDIS_PORT" \
    ${REDIS_PASSWORD:+--redis-password "$REDIS_PASSWORD"} \
    --queue "$INPUT_QUEUE" \
    --result-queue "$OUTPUT_QUEUE" \
    --folds 5 \
    --n-estimators 100
