# BF-GB: Gradient Boosting Feature Selection

Rust 重写的特征选择暴力穷举系统，使用 Gradient Boosting Classifier。

## 核心流程

```
生成组合 → 推送到 Redis → 训练 (GradientBoosting + 5折CV) → 收集到 DuckDB
```

## 构建

```bash
cargo build --release
```

## 使用

### 1. 推送任务

生成 C(43,4) = 123,410 个特征组合到 Redis：

```bash
./target/release/bf-gb push \
  --features 43 \
  --select 4 \
  --redis-host 127.0.0.1 \
  --redis-port 6379 \
  --batch-size 10000
```

### 2. 训练

消费 Redis 任务，训练 Gradient Boosting（默认100个estimator）：

```bash
./target/release/bf-gb train \
  --data data-98.csv \
  --redis-host 127.0.0.1 \
  --redis-port 6379 \
  --folds 5 \
  --n-estimators 100 \
  --learning-rate 0.1 \
  --max-depth 3
```

可以同时启动多个训练实例并行消费任务（多机部署）。

### 3. 收集结果

将 Redis 结果队列中的数据收集到 DuckDB：

```bash
./target/release/bf-gb collect \
  --redis-host 127.0.0.1 \
  --redis-port 6379 \
  --duckdb results.duckdb \
  --table results
```

## 数据格式

CSV 文件需要包含：
- 多列特征（列名任意，不包括 "label"）
- 一列标签（列名必须为 "label"，整数值）

## 项目结构

```
src/
├── main.rs              # CLI 入口
├── config.rs            # 配置和 CLI 参数
├── redis_client.rs      # Redis 操作封装
├── generator.rs         # 特征组合生成器
├── gradient_boosting.rs # Gradient Boosting 实现
├── trainer.rs           # 训练 + 5折交叉验证
├── collector.rs         # DuckDB 结果收集
└── lib.rs               # 模块导出
```

## Gradient Boosting 实现

自实现的 Gradient Boosting：
- **Boosting**: 串行训练，每棵树学习前一棵树的残差
- **学习率**: 控制每棵树贡献的权重（默认 0.1）
- **最大深度**: 控制单棵树的复杂度（默认 3）
- **Sigmoid 激活**: 将输出转换为概率

## 与 Python 版本的对比

| 特性 | Python (sklearn) | Rust (自实现) |
|-----|----------------|-------------|
| 模型 | GradientBoostingClassifier | 自实现 GradientBoosting |
| Redis 序列化 | Pickle | JSON |
| 并发 | 单线程 | 多进程支持 |
| 部署 | Python 环境 | 单二进制文件 |

## 默认参数

Gradient Boosting 的默认参数与 sklearn 保持一致：
- n_estimators: 100 (100棵树)
- learning_rate: 0.1 (学习率)
- max_depth: 3 (树的最大深度)