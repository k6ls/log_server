# Log Server (log_server) [中文](./README.md)

## Project Overview

This is a high-performance log server specifically designed to consume log messages from Kafka message queues and write them to file systems. The server includes enterprise-grade features such as automatic reconnection mechanisms, log file management, and automatic cleanup.

## 🚀 Key Features

### Core Features
- **Kafka Consumer**: Real-time consumption of log messages from Kafka message queues
- **Automatic Reconnection**: Automatic reconnection on connection failure, supporting configurable reconnection intervals
- **High-Performance Log Writing**: Optimized file I/O operations supporting high-concurrency log processing
- **Intelligent Log Management**: Hierarchical storage by year/month/day/hour with automatic file rotation

### Log Features
- **Multi-level Log Support**: TRACE, DEBUG, INFO, WARN, ERROR, FATAL
- **Custom Log Format**: Standardized log format `[Time] [Level] Log Content`
- **JSON Message Parsing**: Supports Kafka message format `{"L":"INFO","S":"Log Content"}`
- **Real-time Log Statistics**: Message processing count and performance monitoring

### System Features
- **Automatic Cleanup**: Configurable log retention days with intelligent cleanup of expired logs
- **Scheduled Tasks**: Support for custom cleanup times (e.g., daily at 1 AM)
- **Error Recovery**: Automatic recovery from connection disconnections and write failures
- **Flexible Configuration**: Flexible configuration of all parameters through YAML files

## 📋 System Requirements

- **Operating System**: Windows/Linux/macOS
- **Rust Version**: 1.70+
- **Kafka Version**: 0.8+
- **Memory**: Minimum 256MB, recommended 512MB+
- **Disk**: Based on log volume requirements

## 🛠️ Quick Deployment

### 1. Build and Deploy

```bash
# Clone the project
git clone <repository-url>
cd log_server

# Build the project
cargo build --release

# Run the service
./target/release/log_server
```

### 2. Docker Deployment

```bash
# Build Docker image
docker build -t log-server .

# Run container
docker run -d \
  --name log-server \
  -v $(pwd)/config.yaml:/app/config.yaml \
  -v $(pwd)/logs:/app/logs \
  log-server
```

### 3. Configuration File

Create `config.yaml` configuration file:

```yaml
logging:
  level: "trace"                    # Log level: trace/debug/info/warn/error/fatal
  path: "logs"                      # Log storage path
  compress: true                     # Whether to compress old log files
  rotate: "hour"                     # Rotation frequency: "hour" or "day"
  retention_days: 90                 # Log retention days
  cleanup_time: "01:00"             # Cleanup time (daily at 1 AM)

kafka:
  enabled: true                      # Enable Kafka consumer
  brokers:                           # Kafka broker list
    - "localhost:9092"
    - "broker2:9092"
  group_id: "log_server_group"       # Consumer group ID
  topics:                            # Subscribed topic list
    - "logs"
    - "app_logs"
  auto_offset_reset: "earliest"      # Offset reset strategy
  session_timeout_ms: 30000          # Session timeout
  heartbeat_interval_ms: 3000        # Heartbeat interval
  reconnect_interval_ms: 10000       # Reconnection interval (milliseconds)
```

### 4. Kafka Message Format

Send log messages to Kafka topics:

```bash
# Send INFO level log
echo '{"L":"INFO","S":"Application started successfully"}' | kafka-console-producer --broker-list localhost:9092 --topic logs

# Send ERROR level log
echo '{"L":"ERROR","S":"Database connection failed"}' | kafka-console-producer --broker-list localhost:9092 --topic logs

# Send multi-line logs
echo '{"L":"DEBUG","S":"User login request processing"}' | kafka-console-producer --broker-list localhost:9092 --topic logs
echo '{"L":"INFO","S":"User authentication successful"}' | kafka-console-producer --broker-list localhost:9092 --topic logs
echo '{"L":"WARN","S":"API response time is long: 2.5s"}' | kafka-console-producer --broker-list localhost:9092 --topic logs
```

## 📁 Log File Structure

Log files are stored in the following structure:

```
logs/
├── 2025/
│   ├── 12/
│   │   ├── 31/
│   │   │   ├── 20.log  # 20:00 logs
│   │   │   ├── 21.log  # 21:00 logs
│   │   │   └── 22.log  # 22:00 logs
│   └── 01/
│       └── 01/
│           └── 00.log  # Next day 00:00 logs
└── 2026/
    └── 01/
        └── 01/
            └── 00.log
```

## 🔧 Configuration Details

### Logging Configuration (logging)
- **level**: Console log output level
- **path**: Log file storage root directory
- **compress**: Whether to compress historical log files
- **rotate**: Log file rotation frequency
- **retention_days**: Log retention days
- **cleanup_time**: Automatic cleanup time (HH:MM format)

### Kafka Configuration (kafka)
- **enabled**: Whether to enable Kafka consumer
- **brokers**: Kafka broker address list
- **group_id**: Consumer group identifier
- **topics**: Subscribed Kafka topic list
- **auto_offset_reset**: Consumer offset reset strategy
- **session_timeout_ms**: Consumer session timeout
- **heartbeat_interval_ms**: Consumer heartbeat interval
- **reconnect_interval_ms**: Reconnection interval (milliseconds)

## 🔄 Operations Management

### Service Startup
```bash
# Development mode
cargo run

# Production mode
./target/release/log_server

# Background execution
nohup ./target/release/log_server &
```

### Log Monitoring
```bash
# View application logs
tail -f logs/$(date +%Y/%m/%d)/$(date +%H).log

# View system status
ps aux | grep log_server

# View resource usage
htop
```

### Troubleshooting
- **Connection failure**: Check Kafka broker address and network connection
- **Message processing failure**: Check application logs to confirm message format
- **Disk space**: Monitor log directory size and clean up promptly
- **Memory usage**: Monitor memory usage and adjust configuration appropriately

## 🚀 Performance Optimization

### High Concurrency Scenarios
- Adjust `reconnect_interval_ms` to optimize reconnection frequency
- Increase system file descriptor limits
- Use SSD storage to improve I/O performance

### Large Data Volume Processing
- Adjust `retention_days` to control storage space
- Enable log compression functionality
- Monitor disk usage

## 🤝 Technical Support

### Common Issues
1. **Kafka connection failure**: Confirm broker address and port are correct
2. **Message format error**: Use standard JSON format `{"L":"Level","S":"Content"}`
3. **Permission issues**: Ensure application has write permissions to log directory

### Contact Information
- Project URL: [GitHub Repository]
- Technical Documentation: [Documentation Link]
- Issue Feedback: [Issues Page]

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Version**: v0.1.0  
**Last Updated**: 2025-12-31  
**Maintainer**: Development Team
