# 日志服务器 (log_server) [English](./README_EN.md)

## 项目简介

这是一个高性能的日志服务器，专门设计用于从Kafka消息队列消费日志消息并写入文件系统。该服务器具备自动重连机制、日志文件管理、自动清理等企业级功能。

## 🚀 主要功能

### 核心功能
- **Kafka消费者**: 从Kafka消息队列实时消费日志消息
- **自动重连机制**: 连接失败时自动重连，支持可配置的重连间隔
- **高性能日志写入**: 优化的文件I/O操作，支持高并发日志处理
- **智能日志管理**: 按年/月/日/小时分层存储，自动文件轮转

### 日志功能
- **多级日志支持**: TRACE、DEBUG、INFO、WARN、ERROR、FATAL
- **自定义日志格式**: 标准化的日志格式 `[时间] [级别] 日志内容`
- **JSON消息解析**: 支持Kafka消息格式 `{"l":"INFO","S":"日志内容"}`
- **实时日志统计**: 消息处理计数和性能监控

### 系统功能
- **自动清理机制**: 可配置的日志保留天数，智能清理过期日志
- **定时任务**: 支持自定义清理时间（如每日凌晨1点）
- **错误恢复**: 连接断开和写入失败时的自动恢复
- **配置灵活**: 通过YAML文件灵活配置所有参数

## 📋 系统要求

- **操作系统**: Windows/Linux/macOS
- **Rust版本**: 1.70+
- **Kafka版本**: 0.8+
- **内存**: 最小256MB，推荐512MB+
- **磁盘**: 根据日志量需求

## 🛠️ 快速部署

### 1. 编译部署

```bash
# 克隆项目
git clone <repository-url>
cd log_server

# 编译项目
cargo build --release

# 运行服务
./target/release/log_server
```

### 2. Docker部署

```bash
# 构建Docker镜像
docker build -t log-server .

# 运行容器
docker run -d \
  --name log-server \
  -v $(pwd)/config.yaml:/app/config.yaml \
  -v $(pwd)/logs:/app/logs \
  log-server
```

### 3. 配置文件

创建 `config.yaml` 配置文件：

```yaml
logging:
  level: "trace"                    # 日志级别: trace/debug/info/warn/error/fatal
  path: "logs"                      # 日志存储路径
  compress: true                     # 是否压缩旧日志文件
  rotate: "hour"                     # 轮转频率: "hour" 或 "day"
  retention_days: 90                 # 日志保留天数
  cleanup_time: "01:00"             # 清理时间 (每天凌晨1点)

kafka:
  enabled: true                      # 启用Kafka消费者
  brokers:                           # Kafka broker列表
    - "localhost:9092"
    - "broker2:9092"
  group_id: "log_server_group"       # 消费者组ID
  topics:                            # 订阅的主题列表
    - "logs"
    - "app_logs"
  auto_offset_reset: "earliest"      # 偏移量重置策略
  session_timeout_ms: 30000          # 会话超时时间
  heartbeat_interval_ms: 3000        # 心跳间隔
  reconnect_interval_ms: 10000       # 重连间隔(毫秒)
```

### 4. Kafka消息格式

发送日志消息到Kafka主题：

```bash
# 发送INFO级别日志
echo '{"l":"INFO","S":"应用程序启动成功"}' | kafka-console-producer --broker-list localhost:9092 --topic logs

# 发送ERROR级别日志
echo '{"l":"ERROR","S":"数据库连接失败"}' | kafka-console-producer --broker-list localhost:9092 --topic logs

# 发送多行日志
echo '{"l":"DEBUG","S":"用户登录请求处理"}' | kafka-console-producer --broker-list localhost:9092 --topic logs
echo '{"l":"INFO","S":"用户认证成功"}' | kafka-console-producer --broker-list localhost:9092 --topic logs
echo '{"l":"WARN","S":"API响应时间较长: 2.5s"}' | kafka-console-producer --broker-list localhost:9092 --topic logs
```

## 📁 日志文件结构

日志文件按以下结构存储：

```
logs/
├── 2025/
│   ├── 12/
│   │   ├── 31/
│   │   │   ├── 20.log  # 20点日志
│   │   │   ├── 21.log  # 21点日志
│   │   │   └── 22.log  # 22点日志
│   └── 01/
│       └── 01/
│           └── 00.log  # 次日0点日志
└── 2026/
    └── 01/
        └── 01/
            └── 00.log
```

## 🔧 配置说明

### 日志配置 (logging)
- **level**: 控制台日志输出级别
- **path**: 日志文件存储根目录
- **compress**: 是否压缩历史日志文件
- **rotate**: 日志文件轮转频率
- **retention_days**: 日志保留天数
- **cleanup_time**: 自动清理时间 (HH:MM格式)

### Kafka配置 (kafka)
- **enabled**: 是否启用Kafka消费者
- **brokers**: Kafka broker地址列表
- **group_id**: 消费者组标识
- **topics**: 订阅的Kafka主题列表
- **auto_offset_reset**: 消费者偏移量重置策略
- **session_timeout_ms**: 消费者会话超时时间
- **heartbeat_interval_ms**: 消费者心跳间隔
- **reconnect_interval_ms**: 重连间隔时间(毫秒)

## 🔄 运维管理

### 启动服务
```bash
# 开发模式
cargo run

# 生产模式
./target/release/log_server

# 后台运行
nohup ./target/release/log_server &
```

### 监控日志
```bash
# 查看应用日志
tail -f logs/$(date +%Y/%m/%d)/$(date +%H).log

# 查看系统状态
ps aux | grep log_server

# 查看资源使用
htop
```

### 故障排查
- **连接失败**: 检查Kafka broker地址和网络连接
- **消息处理失败**: 查看应用日志确认消息格式
- **磁盘空间**: 监控日志目录大小，及时清理
- **内存使用**: 观察内存使用情况，适当调整配置

## 🚀 性能优化

### 高并发场景
- 调整 `reconnect_interval_ms` 优化重连频率
- 增加系统文件描述符限制
- 使用SSD存储提升I/O性能

### 大数据量处理
- 调整 `retention_days` 控制存储空间
- 启用日志压缩功能
- 监控磁盘使用率

## 🤝 技术支持

### 常见问题
1. **Kafka连接失败**: 确认broker地址和端口正确
2. **消息格式错误**: 使用标准JSON格式 `{"l":"级别","S":"内容"}`
3. **权限问题**: 确保应用有写入日志目录的权限

### 联系方式
- 项目地址: [GitHub Repository]
- 技术文档: [Documentation Link]
- 问题反馈: [Issues Page]

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**版本**: v0.1.0  
**更新时间**: 2025-12-31  
**维护者**: Development Team
