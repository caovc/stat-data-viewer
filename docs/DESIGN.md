# Stat Data Viewer 设计决策文档

> 状态：已定稿（2026-08-18 grilling 会话产出）
> 定位：跨平台桌面软件，用于查看和查询各类统计分析软件的数据文件（SAS / SPSS / Stata），只读，不做编辑写回。

## 1. 产品概述

统计软件（SAS、SPSS、Stata）的数据文件是二进制专有格式，脱离原软件难以查看。本软件提供：

- 打开并浏览统计数据文件（数据 + 完整元数据：变量标签、值标签、缺失值定义、显示格式）；
- 灵活查询：数据网格上的排序/筛选（UI 操作），以及完整的 DuckDB SQL 编辑器，支持跨数据集 JOIN；
- 导出查询结果为 CSV / Parquet / Excel。

典型场景：临床数据（如 CDISC ADaM 数据集 ADSL、ADAE）的快速查看与核查。解析正确性是硬要求。

## 2. 支持的文件格式

覆盖 ReadStat 支持的全部读取格式：

| 来源 | 格式 | 说明 |
|---|---|---|
| SAS | `.sas7bdat` | 主数据集格式 |
| SAS | `.xpt` | Transport 格式，v5 与 v8 均支持 |
| SAS | `.sas7bcat` | 目录文件，仅作为元数据辅助文件（提取值标签），不能单独作为数据集打开 |
| SPSS | `.sav` / `.zsav` | 含 zlib 压缩变体 |
| SPSS | `.por` | Portable 格式 |
| Stata | `.dta` | ReadStat 覆盖 v104–v119（Stata 8 到 18+） |

- 文件对话框过滤器、拖拽、系统文件关联注册全部扩展名。
- 扩展名不可靠或无扩展名时，提供手动指定格式的兜底（与手动指定编码在同一个"重新导入"对话框）。
- `.xpt` / `.por` 是交换格式，本身不携带值标签（XPT 仅有变量标签和格式名），元数据面板如实显示"无值标签"。

## 3. 核心架构决策

### 3.1 ReadStat 接入方式：全 Rust 自控（FFI → Arrow → DuckDB）

**决策**：不使用 DuckDB 社区扩展 `read_stat`，改为 Rust 直接绑定 ReadStat C 库，数据经 Arrow 进入 DuckDB。

**备选与理由**：

- ~~方案 A（混合）：DuckDB `read_stat` 扩展查数据 + Rust 薄封装读元数据~~ — 曾被推荐（FFI 成本最低），但被否决；
- ~~方案 B（纯扩展）：只用 DuckDB 扩展~~ — 该扩展只返回数据行，不暴露变量标签/值标签/缺失值定义，对统计数据查看器不可接受；
- **方案 C（选定）：全 Rust**。元数据、错误处理、进度控制、取消全部自控，不依赖社区扩展的发布节奏和质量。代价是 FFI 工作量最大，接受。

### 3.2 绑定来源：自建 readstat-sys（bindgen + vendored C 源码）

**决策**：自建 `readstat-sys` crate（bindgen 生成绑定，vendor ReadStat C 源码进仓库随项目编译），外加一层安全 Rust 封装。

**备选与理由**：

- ~~纯 Rust 的 `polars-readstat-rs`~~ — 零 C 构建痛苦、Arrow 原生，但库很年轻、主要由 AI 生成、单人维护，解析正确性风险不可控；
- ~~主用 C 库 + 纯 Rust 库交叉验证~~ — 工作量最大，否决；
- **选定：ReadStat C**。它是 pandas/pyreadstat 背后久经考验的解析器，对 sas7bdat 这类无公开规范、逆向而来的格式，正确性积累无法替代。临床数据场景下解析错一个值都是大事。代价是 C 构建链（iconv/zlib，Windows 上需 vendor win-iconv），接受。

### 3.3 数据摄取模型：打开时流式全量导入磁盘临时库

**决策**：打开文件时一次性流式导入（ReadStat 回调 → Arrow RecordBatch → DuckDB Appender），写入磁盘上的会话级临时 DuckDB 文件库，而非纯内存库。

**要点**：

- 导入后所有 SQL/排序/筛选/分页均为 DuckDB 原生速度；内存有上限（DuckDB 自管 spill）；源文件只解析一遍；
- 大文件（临床 sas7bdat 可达数 GB）导入需进度条 + 可取消；先流式导入的前几千行立即展示，后台继续导入；
- ~~惰性模式（每次查询重新解析源文件）~~ — ReadStat 不支持随机行访问，每次查询都慢，否决；
- ~~纯内存库~~ — 大文件内存风险，否决。

### 3.4 元数据模型：原始值入数据表 + 元数据侧表 + UI 三态切换

**决策**：数据表永远存原始值；元数据存入同一 DuckDB 库的侧表：

- `meta_variables`：变量名、变量标签、类型、显示格式、缺失值定义等（类似 SPSS Variable View 的数据源）；
- `meta_value_labels`：值标签映射（如 `1 → 男`）。

**UI**：三态切换——显示原值 / 显示标签 / 两者并显（仿 SPSS 的 Value Labels 开关）。

**理由**：SQL 查询时原值和标签表都能 JOIN，用户想怎么查都行；标签不污染数据类型；数值运算不被破坏。~~导入时直接解码成字符串~~ 丢失原值、数值运算全坏，否决；~~DuckDB ENUM 承载~~ 边界情况多（部分标注、标签重复），否决。

**SAS 值标签**：存在单独的 `.sas7bcat` 文件中。打开 `.sas7bdat` 时自动在同目录查找同名/关联 catalog，也支持手动指定。

**日期时间**：SAS/Stata/SPSS 各有纪元和格式体系（如 SAS 纪元 1960-01-01），在 Rust 侧按元数据显示格式格式化后再发给前端，前端不重复实现。

### 3.5 会话模型：单窗口多标签 + 共享会话库，支持跨数据集 JOIN

**决策**：

- 单窗口多标签页；所有打开的数据集导入同一个会话级临时 DuckDB 库；
- 表名取清洗后的文件名，重名加后缀；
- SQL 编辑器天然支持跨数据集 JOIN（临床常见：ADSL JOIN ADAE）——这是引入 DuckDB 的核心价值之一；
- 同一文件重复打开按"路径 + mtime"判断复用已导入的表；
- 临时库文件在应用退出时清理。

### 3.6 查询能力（MVP 范围）

两个独立入口，互不叠加（避免状态机复杂化）：

1. **数据网格 UI 操作**：列头排序、按列类型给出筛选条件（等于/包含/范围/空值等）、列隐藏。所有操作在后端编译成 SQL 执行；
2. **SQL 编辑器面板**：直接写 DuckDB SQL，可引用所有已打开的数据集，结果也进网格。

- SQL 编辑器允许完整 DuckDB SQL，不做语句白名单——表都是临时副本，改坏了重开文件即可；
- 拖拽式可视化查询构建器（选列/聚合/分组）放二期。

### 3.7 前端技术栈

- **框架**：Vue 3 + TypeScript + Pinia，构建工具 Vite；
- **数据网格**：TanStack Table + TanStack Virtual 自建（headless，UI 完全自控），虚拟滚动 + 后端分页，筛选面板、列交互自行实现；
  - ~~AG Grid Community Infinite Row Model~~ — 现成但 UI 自由度低，否决；
- **数据传输**：Tauri command + JSON 分页（每页 200~500 行，LIMIT/OFFSET 或基于行号的 WHERE）。视口级数据量 JSON 序列化开销可忽略（Tauri 2 IPC 已走二进制通道）；
  - Arrow IPC / 自定义二进制协议仅在将来前端需要大数据量可视化（一次拉几十万行画图）时再考虑。

### 3.8 功能边界：严格只读 + 导出

- 任何网格（原始数据或 SQL 结果）可导出 CSV / Parquet / Excel（DuckDB `COPY TO` 直接支持，几乎零成本）；
- **不做**统计格式写回（.sav/.dta/.xpt 写入）：写回意味着元数据完整往返、对"修改数据"负责，产品从"查看器"变"编辑器"，风险和工作量跃升一个量级，临床场景还有合规问题。

### 3.9 平台与分发

- **首发**：macOS（Apple Silicon + Intel universal）+ Windows x64；Linux 二期；
- **CI**：GitHub Actions 三平台矩阵从第一天建立——统计软件用户大多在 Windows，ReadStat C 在 MSVC 下的问题（iconv 依赖需 vendor win-iconv）必须尽早暴露；
- 签名/公证暂不做（内部/小范围分发阶段用不着）；
- **编码**：不自动猜测。中文环境老 SAS/SPSS 文件常见 GBK/GB18030 编码（ReadStat 靠 iconv 转码），导入失败或乱码时 UI 提供"指定编码重新导入"（GBK/GB18030/Latin1 等下拉选项），与手动指定格式同一对话框。

### 3.10 正确性验证策略：golden 测试

- 测试数据：ReadStat 官方仓库自带测试数据 + pyreadstat 生成的已知内容样本文件；
- 覆盖维度：全部六种格式、各数据类型、缺失值（含用户自定义缺失值）、中文编码、日期时间；
- XPT 需 v5、v8 各一份；dta 至少覆盖 v113 和 v118（跨越 Stata 13 格式大改版）；
- 断言：导入 DuckDB 后逐单元格一致，元数据（变量标签/值标签）一致；
- ~~pyreadstat 全量对比脚本~~ — 已评估，暂不做；遇到真实文件的边界情况（压缩变体、奇怪格式）靠用户报 bug 后补 golden 用例。

## 4. 技术栈汇总

| 层 | 选型 |
|---|---|
| 桌面框架 | Tauri 2 |
| 数据解析 | ReadStat C（vendored）+ 自建 readstat-sys（bindgen）+ 安全封装 crate |
| 数据中转 | Apache Arrow（RecordBatch 流） |
| 查询引擎 | DuckDB（`duckdb` crate，bundled 模式），磁盘临时库 |
| 前端 | Vue 3 + TypeScript + Pinia + Vite |
| 数据网格 | TanStack Table + TanStack Virtual（自建 UI） |
| IPC | Tauri command + JSON 分页 |
| CI | GitHub Actions（macOS / Windows / Linux 矩阵） |

## 5. 待定与二期事项

- 可视化查询构建器（选列/聚合/分组的拖拽 UI）；
- Linux 发行；
- 签名与公证；
- Arrow IPC 二进制通道（前端大数据量可视化时）；
- `.sas7bcat` 是否支持单独打开浏览格式定义（当前仅作元数据辅助文件）；
- 应用名称待定。
