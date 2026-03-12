# Shell 脚本使用说明

## 脚本说明

| 脚本 | 功能 |
|-----|------|
| `scripts/start_training.sh` | 启动多个训练实例 |
| `scripts/stop_training.sh` | 停止所有训练实例 |
| `scripts/status.sh` | 查看训练状态 |
| `scripts/run_full_pipeline.sh` | 一键执行完整流程 |

## 使用方法

### 1. 修改配置

编辑 `scripts/start_training.sh`，修改以下参数：

```bash
REDIS_HOST="192.168.31.82"        # Redis 地址
REDIS_PORT="23999"                # Redis 端口
REDIS_PASSWORD="your_password"    # Redis 密码
DATA_FILE="demo/data.csv"         # 数据文件路径
FOLDS=5                           # 交叉验证折数
N_TREES=100                       # 随机森林树数量
```

### 2. 启动训练

```bash
# 启动 4 个训练实例
./scripts/start_training.sh 4

# 或默认启动 2 个实例
./scripts/start_training.sh
```

### 3. 查看状态

```bash
./scripts/status.sh
```

输出示例：
```
运行中的实例数: 4
日志文件数: 4
最新日志摘要...
```

### 4. 查看实时日志

```bash
# 查看实例 1 的日志
tail -f logs/train_1.log

# 查看所有日志
tail -f logs/train_*.log
```

### 5. 停止训练

```bash
./scripts/stop_training.sh
```

### 6. 收集结果

训练完成后（队列为空）：

```bash
./target/release/bf-rust collect \
  --redis-host 192.168.31.82 \
  --redis-port 23999 \
  --redis-password your_password \
  --duckdb results.duckdb
```

## Mac 上使用

1. 复制 `bf-rust/target/release/bf-rust` 到 Mac
2. 复制 `scripts/` 目录和 `demo/data.csv`
3. 修改脚本中的路径（如需要）
4. 执行 `./scripts/start_training.sh`

## 手动启动单个实例

```bash
./bf-rust train \
  --data demo/data.csv \
  --redis-host 192.168.31.82 \
  --redis-port 23999 \
  --redis-password HKAL_A_zxim18jx \
  --folds 5 \
  --n-trees 100
```
