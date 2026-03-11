# ID Generator Service

一个基于 Rust 的高性能分布式 ID 生成服务，支持三种主流算法：**Snowflake**、**ULID** 和 **NanoID**。

## ✨ 特性

- 🚀 **高性能**: 基于 Tokio 异步运行时，支持高并发请求
- 🔢 **多种算法**: 
  - **Snowflake**: 经典雪花算法，64 位整数 ID
  - **ULID**: 按时间排序的 UUID，26 字符
  - **NanoID**: URL 友好的随机 ID，可配置长度
- 📊 **监控完善**: Prometheus 指标收集，支持 Grafana 可视化
- 📝 **日志追踪**: 使用 tracing 进行结构化日志记录
- ⚙️ **灵活配置**: YAML 配置文件 + 环境变量支持
- 🌐 **RESTful API**: 基于 Axum 框架的 HTTP 接口

## 🏗️ 项目结构

```
id-generator/
├── id_generator_service/
│   ├── src/
│   │   ├── main.rs              # 程序入口
│   │   ├── error.rs             # 错误定义
│   │   ├── config/              # 配置管理
│   │   ├── generator/           # ID 生成算法
│   │   │   ├── snowflake.rs     # Snowflake 算法
│   │   │   ├── ulid.rs          # ULID 算法
│   │   │   └── nanoid.rs        # NanoID 算法
│   │   ├── server/              # HTTP 服务层
│   │   └── utils/               # 工具函数
│   ├── Cargo.toml
│   └── config.yaml.example
└── README.md
```

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 安装

```bash
# 克隆项目
git clone https://github.com/YOUR_USERNAME/id-generator.git
cd id-generator/id_generator_service

# 编译
cargo build --release

# 运行
cargo run
```

### 配置

复制示例配置文件：

```bash
cp config.yaml.example config.yaml
```

编辑 `config.yaml`：

```yaml
server:
  host: "0.0.0.0"
  port: 3000

snowflake:
  worker_id: 1          # 机器 ID (0-31)
  datacenter_id: 1      # 机房 ID (0-31)

nanoid:
  length: 21
  alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
```

或使用环境变量（优先级更高）：

```bash
export IDGEN__SERVER__PORT=3000
export IDGEN__SNOWFLAKE__WORKER_ID=1
export IDGEN__SNOWFLAKE__DATACENTER_ID=1
```

## 📡 API 文档

服务启动后访问 `http://localhost:3000`

### 生成单个 ID

```bash
# Snowflake ID
curl -X POST http://localhost:3000/api/v1/id/snowflake \
  -H "Content-Type: application/json" \
  -d '{"count": 1}'

# ULID
curl -X POST http://localhost:3000/api/v1/id/ulid \
  -H "Content-Type: application/json" \
  -d '{}'

# NanoID
curl -X POST http://localhost:3000/api/v1/id/nanoid \
  -H "Content-Type: application/json" \
  -d '{}'
```

响应示例：
```json
{
  "ids": ["69134384458"],
  "algorithm": "snowflake",
  "count": 1
}
```

### 批量生成 ID

```bash
# 批量生成 10 个 Snowflake ID
curl http://localhost:3000/api/v1/id/batch/snowflake/10
```

### 健康检查

```bash
# 健康检查
curl http://localhost:3000/health

# 就绪检查
curl http://localhost:3000/ready
```

### 监控指标

```bash
# Prometheus 格式指标
curl http://localhost:3000/metrics
```

## 📊 Prometheus 指标

- `idgen_requests_total`: ID 生成请求总数（按算法和状态分类）
- `idgen_generation_duration_seconds`: ID 生成耗时直方图

## 🧪 测试

```bash
# 运行单元测试
cargo test

# 代码格式化检查
cargo fmt -- --check

# Clippy 检查
cargo clippy -- -D warnings
```

## 🛠️ 技术栈

- **Web 框架**: [Axum](https://github.com/tokio-rs/axum)
- **异步运行时**: [Tokio](https://tokio.rs/)
- **序列化**: [Serde](https://serde.rs/)
- **日志**: [tracing](https://github.com/tokio-rs/tracing)
- **指标**: [Prometheus](https://prometheus.io/)
- **配置**: [config-rs](https://github.com/mehcode/config-rs)

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

🤖 Generated with [Lingma][https://lingma.aliyun.com]
