# BF-Rust 到 BF-GB 迁移指南

## 概述

本项目从 Random Forest 算法迁移到 Gradient Boosting 算法，同时保持相同的项目架构和工作流程。

## 主要变更

### 1. 项目名称
- **名称**: bf-gb
- **含义**: Brute Force with Gradient Boosting

### 2. 算法: GradientBoostingClassifier

### 3. 核心文件变更

#### 新增文件
- `src/gradient_boosting.rs` - Gradient Boosting 实现

#### 修改文件
- `src/lib.rs` - 模块名从 `random_forest` 改为 `gradient_boosting`
- `src/trainer.rs` - 使用 GradientBoosting 替代 RandomForest
- `src/main.rs` - CLI 参数更新
- `src/config.rs` - 新增 GradientBoosting 参数
- `Cargo.toml` - 项目名称更新

### 4. CLI 参数变更

#### Train 命令参数变化

**旧参数 (RandomForest)**:
```bash
--n-trees 100           # 树的数量
--max-features          # 最大特征数（可选）
```

**新参数 (GradientBoosting)**:
```bash
--n-estimators 100     # 估计器数量（树的数量）
--learning-rate 0.1    # 学习率（新参数）
--max-depth 3          # 最大深度（新参数）
```

### 5. 实现差异

#### RandomForest
- **Bagging**: 并行训练，独立决策
- **投票机制**: 多棵树投票决定最终结果
- **随机性**: 通过 Bootstrap 采样和随机特征选择

#### GradientBoosting
- **Boosting**: 串行训练，每棵树学习前一棵树的残差
- **加权更新**: 通过学习率控制每棵树的贡献
- **Sigmoid 激活**: 将输出转换为概率

### 6. 默认参数设置

#### RandomForest (sklearn 默认)
- n_estimators: 100
- max_features: sqrt(n_features)

#### GradientBoosting (sklearn 默认)
- n_estimators: 100
- learning_rate: 0.1
- max_depth: 3

## 迁移步骤

### 1. 更新项目名称
```bash
# 修改 Cargo.toml
[package]
name = "bf-gb"
```

### 2. 替换算法实现
```bash
# 删除旧文件
rm src/random_forest.rs

# 创建新文件
touch src/gradient_boosting.rs
```

### 3. 更新模块引用
```bash
# lib.rs
- pub mod random_forest;
+ pub mod gradient_boosting;

# trainer.rs
- use crate::random_forest::RandomForest;
+ use crate::gradient_boosting::GradientBoosting;
```

### 4. 更新 CLI 配置
- 移除 `max_features` 参数
- 添加 `learning_rate` 和 `max_depth` 参数
- 将 `n_trees` 改为 `n_estimators`

### 5. 更新训练逻辑
```rust
// RandomForest
let mut rf = RandomForest::new(self.n_trees, n_selected_features);
if let Some(max_feat) = self.max_features {
    rf = rf.with_max_features(max_feat);
}
let rf = rf.fit(&x_train, &y_train)?;

// GradientBoosting
let mut gb = GradientBoosting::new(
    self.n_estimators,
    self.learning_rate,
    self.max_depth
);
let gb = gb.fit(&x_train, &y_train)?;
```

## 性能考虑

### 优点
- GradientBoosting 通常比 RandomForest 有更好的准确率
- 串行训练更容易优化和调试
- 学习率提供了额外的控制参数

### 缺点
- 训练时间更长（串行）
- 更容易过拟合（需要调整参数）
- 实现更复杂

## 使用示例

### 基本使用流程
1. **推送任务**: 相同的命令和参数
2. **训练**: 更新参数名称
3. **收集结果**: 完全相同的命令

### 训练命令对比

**旧 (RandomForest)**:
```bash
./bf-rust train \
  --data data.csv \
  --n-trees 100 \
  --max-features sqrt
```

**新 (GradientBoosting)**:
```bash
./bf-gb train \
  --data data.csv \
  --n-estimators 100 \
  --learning-rate 0.1 \
  --max-depth 3
```

## 测试验证

1. **编译检查**: `cargo check`
2. **功能测试**: 运行 demo 脚本
3. **结果对比**: 与原版本输出格式一致

## 故障排除

### 常见问题
1. **编译错误**: 检查模块引用是否正确
2. **参数错误**: 确保使用新的参数名称
3. **性能问题**: 调整学习率和树深度

### 调试建议
1. 使用较小的数据集进行测试
2. 逐步增加参数复杂度
3. 监控训练时间和内存使用