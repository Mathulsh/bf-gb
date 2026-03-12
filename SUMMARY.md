# BF-GB 项目总结

## 项目概述

BF-GB 是一个基于 Rust 的特征选择暴力穷举系统，使用 Gradient Boosting Classifier 算法。该项目基于原有的 bf-rust 项目架构，将算法从 Random Forest 替换为 Gradient Boosting，保持了相同的工作流程和接口设计。

## 项目结构

```
bf-gb/
├── src/                        # 源代码目录
│   ├── main.rs                # CLI 入口点
│   ├── lib.rs                 # 模块导出
│   ├── config.rs              # CLI 参数配置
│   ├── gradient_boosting.rs   # Gradient Boosting 实现
│   ├── trainer.rs             # 训练和交叉验证
│   ├── collector.rs           # 结果收集器
│   ├── redis_client.rs        # Redis 客户端
│   ├── generator.rs           # 特征组合生成器
│   └── ...                    # 其他支持文件
├── demo/                      # 示例数据
│   └── data.csv              # 训练数据
├── scripts/                   # 脚本文件
│   └── run_demo.sh           # 演示脚本
├── Cargo.toml                # 项目配置
├── README.md                  # 使用文档
├── MIGRATION.md              # 迁移指南
├── SUMMARY.md                 # 项目总结
└── validate_structure.py     # 结构验证脚本
```

## 核心算法实现

### Gradient Boosting 特性

1. **串行训练**: 每棵树学习前一棵树的残差
2. **学习率**: 控制每棵树贡献的权重（默认 0.1）
3. **最大深度**: 限制单棵树的复杂度（默认 3）
4. **Sigmoid 激活**: 将输出转换为概率

### 主要方法

- `new(n_estimators, learning_rate, max_depth)`: 创建新实例
- `fit(x, y)`: 训练模型
- `predict(x)`: 预测类别
- `predict_proba(x)`: 预测概率

## 工作流程

```
1. 生成特征组合
   └─> 使用组合生成器创建所有可能的特征子集

2. 推送到 Redis
   └─> 将任务推送到队列，支持分布式处理

3. 训练模型
   └─> 消费队列任务，训练 Gradient Boosting
   └─> 使用 5 折交叉验证评估性能

4. 收集结果
   └─> 将结果存储到 DuckDB
   └─> 支持查询最佳特征组合
```

## 使用方法

### 1. 推送任务

```bash
./target/release/bf-gb push \
  --features 43 \
  --select 4 \
  --redis-host 127.0.0.1 \
  --redis-port 6379
```

### 2. 训练模型

```bash
./target/release/bf-gb train \
  --data data.csv \
  --redis-host 127.0.0.1 \
  --redis-port 6379 \
  --folds 5 \
  --n-estimators 100 \
  --learning-rate 0.1 \
  --max-depth 3
```

### 3. 收集结果

```bash
./target/release/bf-gb collect \
  --redis-host 127.0.0.1 \
  --redis-port 6379 \
  --duckdb results.duckdb \
  --table results
```

## 与原项目的区别

### 算法差异

| 特性 | Random Forest | Gradient Boosting |
|------|--------------|-------------------|
| 训练方式 | 并行 | 串行 |
| 预测机制 | 投票 | 加权求和 |
| 过拟合风险 | 较低 | 较高 |
| 可调参数 | max_features | learning_rate, max_depth |

### 性能对比

- **训练时间**: GradientBoosting 通常更长（串行训练）
- **准确率**: GradientBoosting 通常更高（如果参数调优得当）
- **内存使用**: 两者相似
- **可并行性**: RandomForest 更容易并行

## 默认参数

### GradientBoosting 参数
- `n_estimators: 100` - 树的数量
- `learning_rate: 0.1` - 学习率
- `max_depth: 3` - 树的最大深度

### Cross-Validation
- `folds: 5` - 5 折交叉验证

## 关键文件说明

### 1. gradient_boosting.rs
实现了完整的 Gradient Boosting 算法，包括：
- 梯度计算（残差）
- 树的训练
- 预测（类别和概率）
- 学习率调整

### 2. trainer.rs
负责：
- 数据加载和预处理
- 交叉验证实现
- 模型训练和评估
- F1-macro 分数计算

### 3. config.rs
定义了所有 CLI 参数，包括新的 GradientBoosting 参数：
- `--n-estimators`
- `--learning-rate`
- `--max-depth`

### 4. main.rs
处理命令行参数，调用相应的功能模块。

## 部署说明

### 环境要求
- Rust 1.70+
- Redis 服务器
- DuckDB（可选，用于结果分析）

### 编译
```bash
cargo build --release
```

### 运行示例
```bash
./scripts/run_demo.sh
```

## 扩展建议

1. **参数调优**: 可以添加网格搜索功能
2. **特征重要性**: 实现 Gradient Boosting 的特征重要性计算
3. **早停机制**: 添加早停防止过拟合
4. **更多算法**: 支持其他 boosting 算法如 XGBoost
5. **可视化**: 添加结果可视化功能

## 故障排除

### 常见问题
1. **编译错误**: 检查 Rust 版本和依赖
2. **Redis 连接**: 确认 Redis 服务运行
3. **内存不足**: 减少批处理大小
4. **过拟合**: 调整学习率和树深度

### 调试技巧
1. 使用小数据集测试
2. 启用详细日志输出
3. 检查数据格式（必须有 'label' 列）
4. 验证特征组合生成

## 总结

BF-GB 成功地将原有的 bf-rust 项目从 Random Forest 迁移到 Gradient Boosting，保持了相同的项目架构和工作流程。新项目提供了更高的预测准确率潜力，同时保持了良好的性能和可扩展性。通过使用 Rust 的性能优势和并发特性，该系统能够高效处理大规模的特征选择任务。