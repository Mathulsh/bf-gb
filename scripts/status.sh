#!/bin/bash
# 查看训练状态

echo "=== bf-gb 训练状态 ==="
echo ""

# 运行中的实例数
RUNNING=$(pgrep -c "bf-gb train" 2>/dev/null || echo "0")
echo "运行中的实例数: $RUNNING"
echo ""

# 显示进程详情
if [ "$RUNNING" -gt 0 ]; then
    echo "进程详情:"
    ps aux | grep "bf-gb train" | grep -v grep | head -10
    echo ""
fi

# 日志文件数量
LOG_COUNT=$(ls -1 logs/train_*.log 2>/dev/null | wc -l)
echo "日志文件数: $LOG_COUNT"
echo ""

# 最新日志摘要
if [ "$LOG_COUNT" -gt 0 ]; then
    echo "最新日志摘要 (最近3个):"
    for log in $(ls -t logs/train_*.log 2>/dev/null | head -3); do
        echo "--- $log ---"
        tail -3 "$log" 2>/dev/null || echo "(空日志)"
        echo ""
    done
fi

# 总处理任务数估算（从日志中提取）
if [ "$LOG_COUNT" -gt 0 ]; then
    TOTAL_PROCESSED=$(grep -h "Processed" logs/train_*.log 2>/dev/null | tail -1 | grep -o "[0-9]\+ tasks" | grep -o "[0-9]\+" || echo "0")
    if [ "$TOTAL_PROCESSED" != "0" ]; then
        echo "最近报告处理任务数: $TOTAL_PROCESSED"
    fi
fi
