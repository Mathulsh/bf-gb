#!/bin/bash

# BF-GB Demo 脚本
# 演示 Gradient Boosting 特征选择的基本流程

set -e

echo "=== BF-GB: Gradient Boosting Feature Selection Demo ==="
echo ""

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the bf-gb project root"
    exit 1
fi

# 检查 demo 数据是否存在
if [ ! -f "demo/data.csv" ]; then
    echo "Error: demo/data.csv not found"
    exit 1
fi

echo "1. 构建项目..."
cargo build --release
echo "✓ Build completed"
echo ""

echo "2. 推送特征组合任务（示例：使用小数据集演示）"
echo "   生成 C(10,3) = 120 个组合"
echo ""

./target/release/bf-gb push \
    --features 10 \
    --select 3 \
    --redis-host 127.0.0.1 \
    --redis-port 6379 \
    --batch-size 100 \
    --queue demo_queue

echo "✓ Tasks pushed to Redis"
echo ""

echo "3. 训练模型（使用 demo 数据）"
echo "   参数："
echo "   - 50 个 estimators"
echo "   - 学习率 0.1"
echo "   - 最大深度 3"
echo "   - 5 折交叉验证"
echo ""

./target/release/bf-gb train \
    --data demo/data.csv \
    --redis-host 127.0.0.1 \
    --redis-port 6379 \
    --queue demo_queue \
    --result_queue demo_results \
    --folds 5 \
    --n-estimators 50 \
    --learning-rate 0.1 \
    --max-depth 3

echo "✓ Training completed"
echo ""

echo "4. 收集结果到 DuckDB"
echo ""

./target/release/bf-gb collect \
    --redis-host 127.0.0.1 \
    --redis-port 6379 \
    --queue demo_results \
    --duckdb demo_results.duckdb \
    --table results \
    --batch-size 100

echo "✓ Results collected to demo_results.duckdb"
echo ""

echo "5. 查看最佳特征组合"
echo ""

echo "Top 5 results:"
echo "-----------------"

# 使用 DuckDB CLI 查询结果（如果可用）
if command -v duckdb &> /dev/null; then
    duckdb demo_results.duckdb "SELECT features, mean_f1_macro FROM results ORDER BY mean_f1_macro DESC LIMIT 5;"
else
    echo "Install DuckDB to run direct queries: brew install duckdb"
    echo "You can manually query the database later with:"
    echo "  duckdb demo_results.duckdb"
fi

echo ""
echo "=== Demo completed successfully! ==="
echo ""
echo "Files created:"
echo "- demo_results.duckdb: SQLite database containing all results"
echo ""
echo "Next steps:"
echo "1. Open DuckDB to analyze results:"
echo "   duckdb demo_results.duckdb"
echo ""
echo "2. Query the best performing features:"
echo "   SELECT features, mean_f1_macro FROM results ORDER BY mean_f1_macro DESC LIMIT 10;"
echo ""
echo "3. For real use, modify the parameters in run_demo.sh:"
echo "   - Increase features and select values for larger datasets"
echo "   - Adjust n_estimators, learning_rate, and max_depth based on your needs"