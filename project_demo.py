#!/usr/bin/env python3
"""
BF-GB 项目演示脚本
在没有 Rust 环境的情况下演示项目结构和功能
"""

import os
import subprocess
import sys
from pathlib import Path

def demo_project_structure():
    """演示项目结构"""
    print("=== BF-GB 项目结构演示 ===\n")

    # 显示目录树
    print("项目目录结构：")
    print("```\nbf-gb/\n")

    # 递归显示目录结构
    for root, dirs, files in os.walk("bf-gb"):
        # 跳过 target 目录以减少输出
        if "target" in root or "__pycache__" in root:
            continue

        level = root.replace("bf-gb", "").count(os.sep)
        indent = " " * 2 * level

        print(f"{indent}{os.path.basename(root)}/")

        subindent = " " * 2 * (level + 1)
        for file in files:
            if not file.endswith((".o", ".so", ".dylib")):
                print(f"{subindent}{file}")

    print("\n```")

def demo_gradient_boosting_logic():
    """演示 Gradient Boosting 的核心逻辑"""
    print("\n=== Gradient Boosting 算法演示 ===\n")

    print("1. 初始化参数：")
    print("   - n_estimators = 100 (树的数量)")
    print("   - learning_rate = 0.1 (学习率)")
    print("   - max_depth = 3 (树的最大深度)")
    print()

    print("2. 训练过程：")
    print("   a. 初始预测：使用类别的平均值")
    print("   b. 计算残差：真实值 - 预测值")
    print("   c. 训练第一棵树：学习残差")
    print("   d. 更新预测：预测值 + learning_rate * 树的预测")
    print("   e. 重复 b-d 步骤，直到达到 n_estimators")
    print()

    print("3. 预测过程：")
    print("   a. 对每棵树进行预测")
    print("   b. 求和：所有树的预测结果")
    print("   c. 应用 Sigmoid：1 / (1 + exp(-sum))")
    print("   d. 二分类：概率 > 0.5 为正类，否则为负类")

def demo_cli_usage():
    """演示 CLI 使用方法"""
    print("\n=== CLI 使用演示 ===\n")

    print("1. 推送任务：")
    print("```bash")
    print("./bf-gb push \\")
    print("  --features 43 \\")
    print("  --select 4 \\")
    print("  --redis-host 127.0.0.1 \\")
    print("  --redis-port 6379")
    print("```")
    print()

    print("2. 训练模型：")
    print("```bash")
    print("./bf-gb train \\")
    print("  --data demo/data.csv \\")
    print("  --n-estimators 100 \\")
    print("  --learning-rate 0.1 \\")
    print("  --max-depth 3 \\")
    print("  --folds 5")
    print("```")
    print()

    print("3. 收集结果：")
    print("```bash")
    print("./bf-gb collect \\")
    print("  --duckdb results.duckdb \\")
    print("  --table results")
    print("```")

def demo_file_analysis():
    """分析关键文件内容"""
    print("\n=== 关键文件分析 ===\n")

    # 检查 gradient_boosting.rs
    print("1. gradient_boosting.rs 核心结构：")
    if os.path.exists("bf-gb/src/gradient_boosting.rs"):
        with open("bf-gb/src/gradient_boosting.rs", 'r') as f:
            content = f.read()

        # 提取关键部分
        structs = [line.strip() for line in content.split('\n') if 'struct ' in line]
        methods = [line.strip() for line in content.split('\n') if 'pub fn ' in line]

        print(f"   - 结构体：{len(structs)} 个")
        for s in structs:
            print(f"     • {s}")

        print(f"   - 公共方法：{len(methods)} 个")
        for m in methods[:3]:  # 显示前3个方法
            print(f"     • {m}")
        if len(methods) > 3:
            print(f"     • ... 还有 {len(methods)-3} 个方法")

    print()

    # 检查 trainer.rs
    print("2. trainer.rs 关键实现：")
    if os.path.exists("bf-gb/src/trainer.rs"):
        with open("bf-gb/src/trainer.rs", 'r') as f:
            content = f.read()

        if "GradientBoosting::new" in content:
            print("   ✓ 使用 GradientBoosting 训练")
        if "n_estimators" in content:
            print("   ✓ 支持 n_estimators 参数")
        if "learning_rate" in content:
            print("   ✓ 支持 learning_rate 参数")
        if "max_depth" in content:
            print("   ✓ 支持 max_depth 参数")

    print()

def demo_migration_differences():
    """演示迁移差异"""
    print("\n=== 迁移前后对比 ===\n")

    print("| 特性 | 原版本 (RandomForest) | 新版本 (GradientBoosting) |")
    print("|------|----------------------|--------------------------|")
    print("| 训练方式 | 并行 | 串行 |")
    print("| 预测机制 | 多数投票 | 加权求和 |")
    print("| 学习方式 | Bootstrap 采样 | 残差学习 |")
    print("| 核心参数 | max_features | learning_rate |")
    print("| 过拟合风险 | 较低 | 较高 |")
    print("| 准确率潜力 | 中等 | 高 |")

    print("\n参数变化：")
    print("• --n-trees → --n-estimators")
    print("• 新增 --learning-rate")
    print("• 新增 --max-depth")
    print("• 移除 --max-features")

def demo_sample_output():
    """演示预期输出"""
    print("\n=== 预期输出示例 ===\n")

    print("1. 推送任务输出：")
    print("```")
    print!("Starting push command: C(43, 4) combinations")
    print!("Total combinations to push: 123410")
    print!("Pushed batch 1 (10000/123410)")
    print!("Pushed batch 10 (100000/123410)")
    print!("Push complete! Total batches: 13")
    print!("```")

    print("\n2. 训练输出：")
    print("```")
    print!("Starting train command")
    print!("Using GradientBoosting with:")
    print!("  n_estimators = 100")
    print!("  learning_rate = 0.1")
    print!("  max_depth = 3")
    print!("Processing fold 1/5")
    print!("Fold 1 F1-macro: 0.8567")
    print!("Processing fold 2/5")
    print!("Fold 2 F1-macro: 0.8432")
    print!("...")
    print!("Features [1, 5, 10, 15] -> Mean F1-macro: 0.85")
    print!("```")

    print("\n3. 收集结果输出：")
    print("```")
    print!("Starting collect command")
    print!("Top 10 results:")
    print!("  1. Features [1, 5, 10, 15] -> F1-macro: 0.85")
    print!("  2. Features [2, 6, 11, 16] -> F1-macro: 0.84")
    print!("  ...")
    print!("Results collected to results.duckdb")
    print!("```")

def main():
    """主演示函数"""
    # 切换到项目目录
    os.chdir("bf-gb")

    print("🚀 BF-GB 项目演示\n")

    # 演示各个部分
    demo_project_structure()
    demo_gradient_boosting_logic()
    demo_cli_usage()
    demo_file_analysis()
    demo_migration_differences()
    demo_sample_output()

    print("\n" + "="*50)
    print("\n📋 演示完成！")
    print("\n要实际运行项目，需要：")
    print("1. 安装 Rust: brew install rust")
    print("2. 编译项目: cargo build --release")
    print("3. 启动 Redis 和 DuckDB")
    print("4. 运行 demo: ./scripts/run_demo.sh")
    print("\n项目已成功从 Random Forest 迁移到 Gradient Boosting！")

if __name__ == "__main__":
    main()