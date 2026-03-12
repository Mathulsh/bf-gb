#!/bin/bash
# 启动多个训练实例
# 用法: ./start_training.sh [实例数量]
# 示例: ./start_training.sh 4

set -e

# 配置参数（按需修改）
REDIS_HOST="127.0.0.1"
REDIS_PORT="6379"
REDIS_PASSWORD=""
DATA_FILE="demo/data-98.csv"
FOLDS=5
N_ESTIMATORS=100

# 实例数量，默认 2
INSTANCES=${1:-2}

# 程序路径
BF_GB="./target/release/bf-gb"

# 检查程序是否存在
if [ ! -f "$BF_GB" ]; then
    echo "错误: 找不到 $BF_GB"
    echo "请先编译: cargo build --release"
    exit 1
fi

# 检查数据文件
if [ ! -f "$DATA_FILE" ]; then
    echo "错误: 找不到数据文件 $DATA_FILE"
    exit 1
fi

# 创建日志目录
mkdir -p logs

echo "启动 $INSTANCES 个训练实例..."
echo "Redis: $REDIS_HOST:$REDIS_PORT"
echo "数据文件: $DATA_FILE"
echo "日志目录: logs/"
echo ""

for i in $(seq 1 $INSTANCES); do
    LOG_FILE="logs/train_$i.log"
    
    # 检查是否已有实例在运行
    if pgrep -f "bf-gb train.*train_$i.log" > /dev/null 2>&1; then
        echo "实例 $i 已在运行，跳过"
        continue
    fi
    
    echo "启动实例 $i -> $LOG_FILE"
    
    nohup "$BF_GB" train \
        --data "$DATA_FILE" \
        --redis-host "$REDIS_HOST" \
        --redis-port "$REDIS_PORT" \
        --redis-password "$REDIS_PASSWORD" \
        --folds "$FOLDS" \
        --n-estimators "$N_ESTIMATORS" \
        > "$LOG_FILE" 2>&1 &
    
    sleep 0.5
done

echo ""
echo "所有实例已启动"
echo "查看日志: tail -f logs/train_1.log"
echo "查看状态: ./status.sh"
echo "停止所有: ./stop_training.sh"
