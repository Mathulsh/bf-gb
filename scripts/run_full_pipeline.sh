#!/bin/bash
# 完整流程：Push + 启动训练 + 等待 + Collect

set -e

# 配置参数（按需修改）
REDIS_HOST="127.0.0.1"
REDIS_PORT="6379"
REDIS_PASSWORD=""
DATA_FILE="demo/data.csv"

# 特征组合参数
FEATURES=98
SELECT=4

# 训练参数
FOLDS=5
N_ESTIMATORS=100
INSTANCES=7 # 目前测试7最快

# 程序路径
BF_GB="./target/release/bf-gb"

echo "========================================"
echo "BF-GB 完整流程"
echo "========================================"
echo "特征: $FEATURES 选 $SELECT"
echo "Redis: $REDIS_HOST:$REDIS_PORT"
echo "训练实例数: $INSTANCES"
echo ""

# Step 1: Push
echo "[1/4] 推送特征组合到 Redis..."
"$BF_GB" push \
    --features "$FEATURES" \
    --select "$SELECT" \
    --redis-host "$REDIS_HOST" \
    --redis-port "$REDIS_PORT" \
    --redis-password "$REDIS_PASSWORD" \
    --batch-size 10000

echo ""
echo "[2/4] 启动 $INSTANCES 个训练实例..."
./scripts/start_training.sh "$INSTANCES"

echo ""
echo "[3/4] 训练进行中..."
echo "按 Ctrl+C 停止查看日志（不会停止训练）"
echo ""

# 显示实时日志
trap 'echo ""; echo "停止查看日志，训练仍在后台运行"; echo "使用 ./scripts/status.sh 查看状态"' INT
tail -f logs/train_1.log &
TAIL_PID=$!

# 等待训练完成（检测 mylist 队列是否为空）
echo "等待训练完成（检测 Redis 队列）..."
while true; do
    sleep 30
    # 这里可以添加 Redis 队列检测逻辑
    # 简化：让用户手动 Ctrl+C 后执行 collect
done

kill $TAIL_PID 2>/dev/null || true

echo ""
echo "[4/4] 收集结果..."
"$BF_GB" collect \
    --redis-host "$REDIS_HOST" \
    --redis-port "$REDIS_PORT" \
    --redis-password "$REDIS_PASSWORD" \
    --duckdb results.duckdb \
    --table results

echo ""
echo "========================================"
echo "流程完成！"
echo "结果: results.duckdb"
echo "========================================"
