# Contractus 编程语言设计需求规范（最终版）

## 项目概述

本项目旨在设计并实现 Contractus —— 一个现代化的系统编程语言，采用 Rust 作为元语言，遵循实用主义设计哲学，通过增量自举的方式实现完整的编译器工具链。

**核心原则：C转译直到自举成功，内存模型从简单开始，功能优先于优化，可运行优先于完美。**

## 需求分析

### 需求 1：语言核心设计

**用户故事：** 作为语言设计者，我希望定义一个简洁而强大的核心语法，以便能够支持系统编程的基本需求。

#### 验收标准

1. WHEN 定义语言语法时 THEN 系统 SHALL 支持以下核心语法元素：
   - 函数定义：`fn name(params) -> return_type { body }`
   - 变量声明：`let name: type = value`
   - 结构体定义：`struct Name { field: type }`
   - 基本类型：i32, bool, u8, [T], [T; N], *T, struct
   - 控制流：if/else, while, for..in, return
   - 表达式：二元运算、函数调用、字面量、成员访问、数组索引、范围表达式

2. WHEN 设计类型系统时 THEN 系统 SHALL 采用静态类型检查
   - 支持基础类型推导
   - 明确的错误报告
   - 显式内存管理（malloc/free + Arena for compiler）

3. WHEN 确定语言定位时 THEN 系统 SHALL 面向系统编程领域
   - 零成本抽象
   - 可预测的性能
   - 直接的硬件访问能力

### 需求 2：编译器架构设计（C转译优先）

**用户故事：** 作为编译器开发者，我希望构建一个模块化的编译器架构，优先使用C转译实现快速原型验证。

#### 验收标准

1. WHEN 设计编译器管道时 THEN 系统 SHALL 采用多阶段架构：
   - Lexer → Parser → AST → Sema → MIR → C_CodeGen
   - 每个阶段可独立测试和替换
   - Phase 1-2: 仅支持C转译后端（唯一选择，2周可实现）
   - Phase 3+: 自举后考虑添加Cranelift/LLVM

2. WHEN 实现错误处理时 THEN 系统 SHALL 提供友好的错误报告：
   - 精确的错误位置（span）
   - 彩色终端输出
   - 多错误收集和报告
   - 错误恢复机制

3. WHEN 构建测试框架时 THEN 系统 SHALL 支持全面的测试：
   - 单元测试（每个编译器组件）
   - 集成测试（端到端编译）
   - 快照测试（AST结构验证）
   - 性能基准测试

### 需求 3：自举能力实现

**用户故事：** 作为语言实现者，我希望语言能够编译自己的编译器，以证明语言的完整性和实用性。

#### 验收标准

1. WHEN 准备自举时 THEN 编译器 SHALL 支持编译自身所需的所有特性：
   - 字符串处理和文件I/O
   - 数据结构（Vec、HashMap等）
   - 基础错误处理（不需要复杂的Result<T,E>）
   - 简单的模块系统

2. WHEN 执行自举过程时 THEN 系统 SHALL 采用增量替换策略：
   - Stage 0: Rust实现的bootstrap编译器
   - Stage 1: 用目标语言重写Lexer
   - Stage 2: 重写Parser和语义分析
   - Stage 3: 重写代码生成器
   - Stage 4: 完整自举验证

3. WHEN 验证自举成功时 THEN 系统 SHALL 确保：
   - stage2 编译器与 stage1 编译器字节级相同
   - 所有测试用例通过
   - 性能指标满足要求

### 需求 4：MVP程序验证

**用户故事：** 作为语言验证者，我希望通过编译实际的工具程序来验证语言的实用性。

#### 验收标准

1. WHEN 实现3周原型时 THEN 系统 SHALL 能编译运行：
   - fibonacci递归程序
   - 简单的算术表达式计算器
   - 基础的控制流程序

2. WHEN 实现3个月MVP时 THEN 系统 SHALL 能编译运行：
   - 命令行参数解析器（测试字符串处理）
   - wc工具（统计行数，测试文件I/O）
   - 简单JSON解析器（测试递归数据结构）

3. WHEN 达到6个月里程碑时 THEN 系统 SHALL 能编译：
   - 1万行以上的程序
   - 编译器自身的子集
   - 基础的系统工具

### 需求 5：内存模型设计

**用户故事：** 作为系统程序员，我希望有一个简单明确的内存模型，能够进行可预测的内存管理。

#### 验收标准

1. WHEN 实现初始内存模型时 THEN 系统 SHALL 采用：
   - 明确的手动内存管理（malloc/free包装）
   - Arena分配器用于编译器自身
   - 无垃圾回收、无引用计数、无借用检查

2. WHEN 预留并发基础时 THEN 系统 SHALL 支持：
   - 基础原子类型（AtomicI32, AtomicBool）
   - 原子操作（load, store, compare_exchange）
   - 内存屏障（memory_fence）
   - 推迟：线程、async/await、channels

3. WHEN 自举后演进时 THEN 系统 MAY 考虑：
   - Region-based内存管理
   - 简化的所有权系统
   - 完整借用检查（如果需要）

### 需求 6：开发工具链

**用户故事：** 作为语言用户，我希望拥有完整的开发工具链，以便高效地使用这门语言进行开发。

#### 验收标准

1. WHEN 提供编译器时 THEN 系统 SHALL 支持：
   - 命令行接口（CLI）
   - C代码输出（人类可读，便于调试）
   - 编译选项（优化级别、目标平台）
   - 自检模式（--self-test）

2. WHEN 提供调试支持时 THEN 系统 SHALL 包含：
   - AST可视化工具
   - MIR追踪和分析
   - 编译过程诊断
   - 性能分析工具

3. WHEN 提供文档时 THEN 系统 SHALL 包含：
   - 语言规范文档
   - 编译器架构文档
   - API参考文档
   - 示例程序集合

## 技术约束

### 开发环境约束

- 元语言：Rust (最新稳定版)
- 目标平台：x86_64 Linux/Windows/macOS
- 构建系统：Cargo
- 测试框架：内置测试 + insta快照测试
- 依赖策略：初期零依赖，纯手写实现

### 架构约束

- 编译器架构：多阶段管道
- 后端选择：C转译是Phase 1-2的唯一选择
- 内存模型：显式内存管理（malloc/free + Arena）
- 并发模型：基础原子操作，高级特性推迟

### 时间约束（激进但可行）

- Sprint (原型验证): 3周 - fib(10)运行
- MVP (核心语言): 3个月 - wc工具运行
- Bootstrap (编译器完整): 6个月 - 编译1万行代码
- Self-host (自举成功): 9-12个月 - stage2==stage1
- Evolution (高级特性): 12-18个月 - 借用检查器

## 成功标准

### 3周冲刺目标

- [ ] 能够编译并运行这个程序：

```contractus
struct Point {
    x: i32,
    y: i32,
}

fn add_points(a: Point, b: Point) -> Point {
    let result: Point = Point {
        x: a.x + b.x,
        y: a.y + b.y,
    };
    return result;
}

fn sum_array(arr: [i32; 5]) -> i32 {
    let total = 0;
    for item in arr {
        total = total + item;
    }
    return total;
}

fn main() -> i32 {
    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 30, y: 40 };
    let sum = add_points(p1, p2);

    let numbers: [i32; 5] = [1, 2, 3, 4, 5];
    let array_sum = sum_array(numbers);

    print(sum.x);      // 输出 40
    print(array_sum);  // 输出 15
    return 0;
}
```

### 3个月MVP目标

- [ ] 编译并运行wc工具（统计文件行数）
- [ ] 支持字符串处理和文件I/O
- [ ] 通过500个测试用例

### 12个月自举目标

- [ ] 编译器能够编译自身
- [ ] stage2 == stage1 (字节级相同)
- [ ] 所有测试用例通过
- [ ] 性能基准达标

## 风险评估与缓解

### 高风险项

1. **C转译性能担忧** - 生成的代码可能性能不佳
   - 缓解：利用C编译器优化，Zig已证明可行

2. **特性蔓延** - 可能导致无法按时自举
   - 缓解：严格控制MVP范围，推迟非核心特性

3. **3周目标过于激进** - 可能无法实现
   - 缓解：功能优先于完美，可运行即成功

### 中风险项

1. **内存管理复杂性** - 手动管理可能导致bug
2. **测试覆盖不足** - 可能影响代码质量
3. **文档滞后** - 可能影响项目可维护性

## 立即行动计划

### Day 0 (今天)

```bash
# 创建项目结构
mkdir -p contractus/{src,tests,examples,docs}
cd contractus

# 初始化Cargo项目
cat > Cargo.toml << EOF
[package]
name = "contractus"
version = "0.0.1"
edition = "2021"

[dependencies]
# 暂时不加任何依赖，纯手写

[dev-dependencies]
insta = "1.31"
EOF

# 创建最简单的主文件
cat > src/main.rs << EOF
mod lexer;
mod parser;
mod codegen_c;  // 明确是C后端

fn main() {
    println!("Contractus Compiler v0.0.1");
    // TODO: 明天实现
}
EOF

git init
git add .
git commit -m "Day 0: Contractus project initialized"
```

### Week 1 目标

让上面的add函数示例程序能够编译运行，输出30。

### 核心信念

**不完美的代码胜过完美的计划。今天写下第一行代码，3周后你将拥有自己的编程语言！**

---

*"The best language is the one that gets implemented." - 实践出真知*
