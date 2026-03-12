#!/usr/bin/env python3
"""
BF-GB 项目结构验证脚本
验证所有必要的文件和结构是否正确
"""

import os
import sys
from pathlib import Path

def check_file_exists(filepath, description):
    """检查文件是否存在"""
    if Path(filepath).exists():
        print(f"✓ {description}: {filepath}")
        return True
    else:
        print(f"✗ {description}: {filepath} - MISSING")
        return False

def check_file_contains(filepath, content, description):
    """检查文件是否包含特定内容"""
    if not Path(filepath).exists():
        return False

    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            text = f.read()
            if content in text:
                print(f"✓ {description}: {filepath}")
                return True
            else:
                print(f"✗ {description}: {filepath} - CONTENT NOT FOUND")
                return False
    except Exception as e:
        print(f"✗ Error reading {filepath}: {e}")
        return False

def main():
    print("=== BF-GB 项目结构验证 ===\n")

    # 基本项目结构检查
    print("1. 基本项目结构:")
    base_files = [
        ("Cargo.toml", "Cargo 配置文件"),
        ("src/lib.rs", "库文件"),
        ("src/main.rs", "主程序"),
        ("src/config.rs", "配置模块"),
        ("src/gradient_boosting.rs", "Gradient Boosting 实现"),
        ("src/trainer.rs", "训练器"),
        ("src/collector.rs", "数据收集器"),
        ("src/redis_client.rs", "Redis 客户端"),
        ("src/generator.rs", "生成器"),
        ("README.md", "文档"),
        ("MIGRATION.md", "迁移指南"),
    ]

    all_good = True
    for filepath, desc in base_files:
        if not check_file_exists(filepath, desc):
            all_good = False

    print("\n2. 配置验证:")

    # 检查 Cargo.toml 内容
    if check_file_contains("Cargo.toml", 'name = "bf-gb"', "项目名称"):
        check_file_contains("Cargo.toml", 'gradient_boosting', "模块依赖检查")

    # 检查 main.rs 内容
    print("\n3. 主程序验证:")
    check_file_contains("src/main.rs", "mod gradient_boosting", "模块导入")
    check_file_contains("src/main.rs", "n_estimators", "参数使用")
    check_file_contains("src/main.rs", "learning_rate", "参数使用")
    check_file_contains("src/main.rs", "max_depth", "参数使用")

    # 检查 config.rs 内容
    print("\n4. 配置模块验证:")
    check_file_contains("src/config.rs", "n_estimators", "CLI 参数定义")
    check_file_contains("src/config.rs", "learning_rate", "CLI 参数定义")
    check_file_contains("src/config.rs", "max_depth", "CLI 参数定义")
    check_file_contains("src/config.rs", 'command(name = "bf-gb")', "命令名称")

    # 检查 trainer.rs 内容
    print("\n5. 训练器验证:")
    check_file_contains("src/trainer.rs", "GradientBoosting", "算法使用")
    check_file_contains("src/trainer.rs", "n_estimators", "参数传递")
    check_file_contains("src/trainer.rs", "learning_rate", "参数传递")
    check_file_contains("src/trainer.rs", "max_depth", "参数传递")

    # 检查 gradient_boosting.rs 内容
    print("\n6. Gradient Boosting 实现:")
    check_file_contains("src/gradient_boosting.rs", "struct GradientBoosting", "结构定义")
    check_file_contains("src/gradient_boosting.rs", "fit", "训练方法")
    check_file_contains("src/gradient_boosting.rs", "predict", "预测方法")
    check_file_contains("src/gradient_boosting.rs", "learning_rate", "学习率参数")

    # 检查脚本文件
    print("\n7. 脚本文件:")
    script_files = [
        ("scripts/run_demo.sh", "演示脚本"),
    ]

    for filepath, desc in script_files:
        if check_file_exists(filepath, desc):
            # 检查脚本是否有执行权限
            if os.access(filepath, os.X_OK):
                print(f"  ✓ {desc} 有执行权限")
            else:
                print(f"  ⚠ {desc} 无执行权限")

    # 检查 demo 数据
    print("\n8. Demo 数据:")
    demo_files = [
        ("demo/data.csv", "演示数据"),
    ]

    for filepath, desc in demo_files:
        if check_file_exists(filepath, desc):
            # 简单检查 CSV 格式
            try:
                with open(filepath, 'r') as f:
                    first_line = f.readline().strip()
                    if "label" in first_line:
                        print(f"  ✓ {desc} 包含 'label' 列")
                    else:
                        print(f"  ⚠ {desc} 可能缺少 'label' 列")
            except:
                print(f"  ✗ 无法读取 {desc}")

    print("\n=== 验证结果 ===")
    if all_good:
        print("✓ 所有基本文件检查通过")
        print("\n项目已成功从 bf-rust 迁移到 bf-gb！")
        print("\n下一步:")
        print("1. 安装 Rust 环境: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
        print("2. 编译项目: cargo build --release")
        print("3. 运行 demo: ./scripts/run_demo.sh")
    else:
        print("✗ 部分文件检查失败，请检查项目结构")
        sys.exit(1)

if __name__ == "__main__":
    main()