#!/bin/bash
# 停止所有训练实例

echo "停止所有 bf-gb train 进程..."

# 查找并终止所有 bf-gb train 进程
pkill -f "bf-gb train" 2>/dev/null || true

echo "已停止"
echo ""

# 显示剩余的 bf-gb 进程（如果有）
REMAINING=$(pgrep -c "bf-gb" 2>/dev/null || echo "0")
if [ "$REMAINING" -gt 0 ]; then
    echo "注意: 还有 $REMAINING 个 bf-gb 进程在运行"
    ps aux | grep bf-gb | grep -v grep
else
    echo "所有 bf-gb 进程已停止"
fi
