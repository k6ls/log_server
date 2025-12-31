use chrono::{DateTime, Duration as ChronoDuration, Local};
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::SystemTime;
use tokio::net::{TcpStream as AsyncTcpStream};
use tokio::time::{interval, sleep};

// 静态字符串常量，避免重复创建
const LEVEL_TRACE: &str = "TRACE";
const LEVEL_DEBUG: &str = "DEBUG";
const LEVEL_INFO: &str = "INFO";
const LEVEL_WARN: &str = "WARN";
const LEVEL_ERROR: &str = "ERROR";
const LEVEL_FATAL: &str = "FATAL";

const LEVEL_ABBR_TRACE: &str = "T";
const LEVEL_ABBR_DEBUG: &str = "D";
const LEVEL_ABBR_INFO: &str = "I";
const LEVEL_ABBR_WARN: &str = "W";
const LEVEL_ABBR_ERROR: &str = "E";
const LEVEL_ABBR_FATAL: &str = "F";

const DEFAULT_LOG_PATH: &str = "logs";
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

const CLEANUP_TIME_ERROR_MSG: &str =
    "清理时间配置错误，请使用HH:MM或HH:MM:SS格式（0-23:0-59:0-59）";
const RETENTION_DAYS_ERROR: &str = "日志保留天数必须大于0";
const EMPTY_BROKERS_ERROR: &str = "Kafka启用时，brokers不能为空";
const EMPTY_TOPICS_ERROR: &str = "Kafka启用时，topics不能为空";
const EMPTY_GROUP_ID_ERROR: &str = "Kafka启用时，group_id不能为空";

// 使用枚举替代字符串，防止E122错误
#[derive(Debug, Clone, PartialEq)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    fn from_str(level: &str) -> Option<Self> {
        match level.to_uppercase().as_str() {
            LEVEL_TRACE => Some(LogLevel::Trace),
            LEVEL_DEBUG => Some(LogLevel::Debug),
            LEVEL_INFO => Some(LogLevel::Info),
            LEVEL_WARN => Some(LogLevel::Warn),
            LEVEL_ERROR => Some(LogLevel::Error),
            LEVEL_FATAL => Some(LogLevel::Fatal),
            _ => None,
        }
    }

    fn to_abbreviation(&self) -> &'static str {
        match self {
            LogLevel::Trace => LEVEL_ABBR_TRACE,
            LogLevel::Debug => LEVEL_ABBR_DEBUG,
            LogLevel::Info => LEVEL_ABBR_INFO,
            LogLevel::Warn => LEVEL_ABBR_WARN,
            LogLevel::Error => LEVEL_ABBR_ERROR,
            LogLevel::Fatal => LEVEL_ABBR_FATAL,
        }
    }
}

// JSON消息结构体
#[derive(Debug, serde::Deserialize)]
struct KafkaMessage {
    #[serde(rename = "L")]
    l: String, // 日志级别
    #[serde(rename = "S")]
    s: String, // 日志内容
}

#[derive(Debug, serde::Deserialize)]
struct Config {
    logging: LoggingConfig,
    kafka: KafkaConfig,
}

#[derive(Debug, serde::Deserialize)]
struct LoggingConfig {
    level: String,
    path: String,
    #[allow(dead_code)]
    compress: bool,
    #[allow(dead_code)]
    rotate: String,
    retention_days: u32,
    cleanup_time: Option<String>, // 日志清理时间（格式: "HH:MM"）
}

#[derive(Debug, serde::Deserialize)]
struct KafkaConfig {
    enabled: bool,
    brokers: Vec<String>,
    group_id: String,
    topics: Vec<String>,
    #[allow(dead_code)]
    auto_offset_reset: String,
    #[allow(dead_code)]
    session_timeout_ms: u32,
    #[allow(dead_code)]
    heartbeat_interval_ms: u32,
    reconnect_interval_ms: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取配置文件
    let config = load_config()?;

    // 初始化日志系统
    init_logging(&config.logging).await;

    tklog::async_info!("log_server|", "日志服务器启动中...");
    tklog::async_info!(
        "log_server|",
        &format!(
            "日志级别: {}, 日志路径: {}, 保存天数: {}",
            config.logging.level, config.logging.path, config.logging.retention_days
        )
    );

    // 启动日志清理任务
    let retention_days = config.logging.retention_days;
    let cleanup_time = config.logging.cleanup_time.clone();
    tokio::spawn(async move {
        start_log_cleanup_task(retention_days, cleanup_time).await;
    });

    // 启动Kafka消费者
    if config.kafka.enabled {
        tklog::async_info!("log_server|", "启动Kafka消费者...");
        start_kafka_consumer(config.kafka).await?;
    } else {
        tklog::async_warn!("log_server|", "Kafka未启用，服务器空闲运行");
    }

    Ok(())
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content =
        fs::read_to_string("config.yaml").map_err(|e| format!("配置文件读取失败: {}", e))?;

    let config: Config =
        serde_yaml::from_str(&config_content).map_err(|e| format!("配置文件解析失败: {}", e))?;

    // 验证配置有效性
    validate_config(&config)?;

    Ok(config)
}

fn validate_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // 验证日志配置
    if config.logging.retention_days == 0 {
        return Err(RETENTION_DAYS_ERROR.into());
    }

    // 验证清理时间配置
    if let Some(ref cleanup_time) = config.logging.cleanup_time {
        if let Err(e) = parse_cleanup_time(cleanup_time) {
            let error_msg = format!("{}: {}", CLEANUP_TIME_ERROR_MSG, e);
            return Err(error_msg.into());
        }
    }

    // 验证Kafka配置
    if config.kafka.enabled {
        validate_kafka_config(&config.kafka)?;
    }

    Ok(())
}

async fn init_logging(log_config: &LoggingConfig) {
    // 创建日志目录结构：年/月/日/小时.log
    let now = chrono::Local::now();
    let timestamp = now.format(TIMESTAMP_FORMAT).to_string();

    // 使用PathBuf构建路径，减少字符串操作
    use std::path::PathBuf;
    let mut log_dir = PathBuf::from(&log_config.path);
    log_dir.push(now.format("%Y").to_string());
    log_dir.push(now.format("%m").to_string());
    log_dir.push(now.format("%d").to_string());

    let mut log_file = log_dir.clone();
    log_file.push(format!("{}.log", now.format("%H")));

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("创建日志目录失败: {:?}", e);
    }

    // 使用write!直接写入文件，避免format!的中间字符串分配
    if let Err(e) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .and_then(|mut file| {
            // 写入初始化消息
            writeln!(
                file,
                "[{}] [{}] 日志系统已初始化",
                timestamp, LEVEL_ABBR_INFO
            )?;
            writeln!(
                file,
                "[{}] [{}] 日志目录: {:?}",
                timestamp, LEVEL_ABBR_INFO, log_dir
            )?;
            writeln!(
                file,
                "[{}] [{}] 当前日志文件: {:?}",
                timestamp, LEVEL_ABBR_INFO, log_file
            )?;
            Ok(())
        })
    {
        eprintln!("写入初始化日志失败: {:?}", e);
    }

    tklog::async_info!("log_server|", "日志系统已初始化");
    tklog::async_info!("log_server|", &format!("日志目录: {:?}", log_dir));
    tklog::async_info!("log_server|", &format!("当前日志文件: {:?}", log_file));
}

async fn log_with_level(
    level: &str,
    content: &str,
    timestamp: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 使用传入的时间戳来确定日志文件路径
    let timestamp_naive = chrono::NaiveDateTime::parse_from_str(timestamp, TIMESTAMP_FORMAT)
        .unwrap_or_else(|_| chrono::Local::now().naive_local());

    // 预构建目录路径
    let year = timestamp_naive.format("%Y").to_string();
    let month = timestamp_naive.format("%m").to_string();
    let day = timestamp_naive.format("%d").to_string();
    let hour = timestamp_naive.format("%H").to_string();

    // 使用PathBuf来构建路径，减少字符串操作
    use std::path::PathBuf;
    let mut log_dir = PathBuf::from(DEFAULT_LOG_PATH);
    log_dir.push(&year);
    log_dir.push(&month);
    log_dir.push(&day);

    let mut log_file = log_dir.clone();
    log_file.push(format!("{}.log", hour));

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&log_dir) {
        tklog::async_error!(
            "log",
            &format!("创建日志目录失败: {:?}，目录: {:?}", e, log_dir)
        );
        return Ok(());
    }

    // 使用writeln!直接写入文件，避免format!的中间字符串分配
    if let Err(e) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .and_then(|mut file| {
            writeln!(
                file,
                "[{}] [{}] {}",
                timestamp,
                get_level_abbreviation(level),
                content
            )
        })
    {
        tklog::async_error!(
            "log",
            &format!("写入日志文件失败: {:?}，文件: {:?}", e, log_file)
        );
        return Ok(());
    }

    // 同时输出到控制台（这里使用format!因为是单次调用，影响较小）
    let trimmed_message = format!(
        "[{}] [{}] {}",
        timestamp,
        get_level_abbreviation(level),
        content
    );
    
    // 使用枚举进行安全匹配，防止E122错误
    if let Some(log_level) = LogLevel::from_str(level) {
        match log_level {
            LogLevel::Trace => tklog::async_trace!("simulated_log", &trimmed_message),
            LogLevel::Debug => tklog::async_debug!("simulated_log", &trimmed_message),
            LogLevel::Info => tklog::async_info!("simulated_log", &trimmed_message),
            LogLevel::Warn => tklog::async_warn!("simulated_log", &trimmed_message),
            LogLevel::Error => tklog::async_error!("simulated_log", &trimmed_message),
            LogLevel::Fatal => tklog::async_fatal!("simulated_log", &trimmed_message),
        }
    } else {
        // 如果是未知级别，默认使用INFO
        tklog::async_info!("simulated_log", &trimmed_message);
    }

    Ok(())
}

fn get_level_abbreviation(level: &str) -> &'static str {
    // 使用枚举进行安全的模式匹配
    if let Some(log_level) = LogLevel::from_str(level) {
        log_level.to_abbreviation()
    } else {
        LEVEL_ABBR_INFO // 默认返回INFO级别缩写
    }
}

// 日志清理任务：每天 N 点执行（配置文件：cleanup_time）
async fn start_log_cleanup_task(retention_days: u32, cleanup_time: Option<String>) {
    tklog::async_info!(
        "cleanup|",
        &format!("启动日志清理任务，保留{}天", retention_days)
    );

    loop {
        let now = Local::now();
        let next_cleanup = match get_next_cleanup_time(now, cleanup_time.as_deref()) {
            Ok(time) => time,
            Err(_) => {
                // 在异步上下文外记录错误
                eprintln!("[ERROR] cleanup 清理时间计算错误，使用默认时间01:00");
                // 使用默认时间
                now.date_naive()
                    .and_hms_opt(1, 0, 0)
                    .and_then(|naive| naive.and_local_timezone(Local).single())
                    .unwrap_or_else(|| {
                        // 如果连默认时间都失败，使用当前时间+1小时
                        now + ChronoDuration::hours(1)
                    })
            }
        };
        let sleep_duration = next_cleanup.signed_duration_since(now);

        if sleep_duration.num_seconds() > 0 {
            tklog::async_info!(
                "cleanup",
                &format!("下次清理时间: {}", next_cleanup.format("%Y-%m-%d %H:%M:%S"))
            );
            tokio::time::sleep(Duration::from_secs(sleep_duration.num_seconds() as u64)).await;
        }

        // 执行清理
        cleanup_old_logs("logs", retention_days).await;
    }
}

// 计算下次清理时间（每天凌晨1点）
fn get_next_cleanup_time(
    now: DateTime<Local>,
    cleanup_time: Option<&str>,
) -> Result<DateTime<Local>, Box<dyn std::error::Error>> {
    let (hour, minute, second) = match cleanup_time {
        Some(time_str) => {
            parse_cleanup_time(time_str).map_err(|e| format!("清理时间解析失败: {}", e))?
        }
        None => (1, 0, 0), // 默认凌晨1点
    };

    let next = now
        .date_naive()
        .and_hms_opt(hour, minute, second)
        .ok_or_else(|| format!("无效的时间: {}:{}:{}", hour, minute, second))?;

    let next_datetime = match next.and_local_timezone(Local) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::None => return Err("时区转换失败：本地时间不存在".into()),
        chrono::LocalResult::Ambiguous(early, _) => early,
    };

    if now >= next_datetime {
        let next_with_delay = next_datetime + ChronoDuration::days(1);
        Ok(next_with_delay)
    } else {
        Ok(next_datetime)
    }
}

fn validate_kafka_config(kafka_config: &KafkaConfig) -> Result<(), Box<dyn std::error::Error>> {
    if kafka_config.brokers.is_empty() {
        return Err(EMPTY_BROKERS_ERROR.into());
    }
    if kafka_config.topics.is_empty() {
        return Err(EMPTY_TOPICS_ERROR.into());
    }
    if kafka_config.group_id.is_empty() {
        return Err(EMPTY_GROUP_ID_ERROR.into());
    }
    Ok(())
}



fn parse_cleanup_time(time_str: &str) -> Result<(u32, u32, u32), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return Err("时间格式应为 HH:MM 或 HH:MM:SS".into());
    }

    let hour: u32 = parts[0]
        .parse()
        .map_err(|_| "小时格式无效（应为0-23的数字）".to_string())?;
    let minute: u32 = parts[1]
        .parse()
        .map_err(|_| "分钟格式无效（应为0-59的数字）".to_string())?;
    let second: u32 = if parts.len() == 3 {
        parts[2]
            .parse()
            .map_err(|_| "秒格式无效（应为0-59的数字）".to_string())?
    } else {
        0
    };

    if hour > 23 {
        return Err("小时必须在0-23之间".to_string().into());
    }
    if minute > 59 {
        return Err("分钟必须在0-59之间".to_string().into());
    }
    if second > 59 {
        return Err("秒必须在0-59之间".to_string().into());
    }

    Ok((hour, minute, second))
}

// 清理超过指定天数的日志文件
async fn cleanup_old_logs(log_path: &str, retention_days: u32) {
    let cutoff_date = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * retention_days as u64);
    let mut cleaned_count = 0;

    tklog::async_info!(
        "cleanup",
        &format!("开始清理{}天前的日志文件", retention_days)
    );

    let Ok(entries) = fs::read_dir(log_path) else {
        tklog::async_error!("cleanup", "无法读取日志目录");
        return;
    };

    for entry in entries {
        let Ok(entry) = entry else {
            continue; // 跳过无效的目录条目
        };

        let path = entry.path();
        if !path.is_dir() {
            continue; // 只处理目录，忽略文件
        }

        // 检查年份目录
        let Ok(metadata) = entry.metadata() else {
            continue; // 跳过无法获取元数据的目录
        };

        let Ok(modified) = metadata.modified() else {
            continue; // 跳过无法获取修改时间的目录
        };

        if modified >= cutoff_date {
            continue; // 目录未过期，跳过
        }

        // 删除过期的目录
        if let Err(e) = fs::remove_dir_all(&path) {
            tklog::async_error!(
                "cleanup",
                &format!("删除目录失败 {:?}: {}", path, e)
            );
        } else {
            cleaned_count += 1;
            tklog::async_info!(
                "cleanup",
                &format!("已删除过期目录: {:?}", path)
            );
        }
    }

    if cleaned_count > 0 {
        tklog::async_info!(
            "cleanup",
            &format!("清理完成，删除了{}个过期目录", cleaned_count)
        );
    } else {
        tklog::async_info!("cleanup", "没有找到过期的日志文件");
    }
}

// Kafka消费者功能 - 实现自动重连机制
async fn start_kafka_consumer(kafka_config: KafkaConfig) -> Result<(), Box<dyn std::error::Error>> {
    tklog::async_info!("kafka|", "启动Kafka消费者...");
    
    // 输出配置信息
    for broker in &kafka_config.brokers {
        tklog::async_info!("kafka|", &format!("broker: {}", broker));
    }
    tklog::async_info!("kafka|", &format!("消费组ID: {}", kafka_config.group_id));
    tklog::async_info!("kafka|", &format!("主题: {:?}", kafka_config.topics));
    tklog::async_info!("kafka|", &format!("重连间隔: {}ms", kafka_config.reconnect_interval_ms));

    // 自动重连循环
    loop {
        match kafka_consumer_loop(&kafka_config).await {
            Ok(_) => {
                tklog::async_info!("kafka|", "Kafka消费者正常结束");
                break;
            }
            Err(e) => {
                tklog::async_error!("kafka|", &format!("Kafka消费者错误: {}", e));
                tklog::async_info!("kafka|", &format!("Kafka {}ms 后重连...", kafka_config.reconnect_interval_ms));
                
                // 等待指定间隔后重连
                sleep(Duration::from_millis(kafka_config.reconnect_interval_ms)).await;
                continue;
            }
        }
    }

    Ok(())
}

// Kafka消费者主循环 - 包含连接和消息处理逻辑
async fn kafka_consumer_loop(kafka_config: &KafkaConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 尝试连接到第一个可用的broker
    let mut connected = false;
    for broker in &kafka_config.brokers {
        match connect_to_broker(broker).await {
            Ok(_) => {
                tklog::async_info!("kafka|", &format!("成功连接到broker: {}", broker));
                connected = true;
                break;
            }
            Err(e) => {
                tklog::async_warn!("kafka|", &format!("连接broker {} 失败: {}", broker, e));
                continue;
            }
        }
    }

    if !connected {
        return Err("所有broker连接失败".into());
    }

    // 模拟消息消费循环
    let mut message_count = 0u64;
    let mut reconnect_interval = interval(Duration::from_secs(10));

    loop {
        // 定期检查连接状态
        reconnect_interval.tick().await;

        // 模拟从Kafka接收消息
        match simulate_kafka_message_reception().await {
            Ok(Some(message)) => {
                message_count += 1;
                
                // 处理接收到的消息
                if let Err(e) = process_kafka_message(&message).await {
                    tklog::async_error!("kafka|", &format!("处理消息失败: {}", e));
                    
                    // 如果写日志失败，触发重连
                    if is_connection_error(&e.to_string()) {
                        tklog::async_warn!("kafka|", "检测到连接错误，触发重连...");
                        return Err(e);
                    }
                }

                // 每处理100条消息输出统计信息
                if message_count.is_multiple_of(100) {
                    tklog::async_info!("kafka|", &format!("已处理 {} 条消息", message_count));
                }
            }
            Ok(None) => {
                // 没有消息时短暂等待
                sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => {
                tklog::async_error!("kafka|", &format!("接收消息失败: {}", e));
                
                // 检查是否是连接错误
                if is_connection_error(&e.to_string()) {
                    tklog::async_warn!("kafka|", "检测到连接错误，触发重连...");
                    return Err(e);
                }
                
                // 其他错误短暂等待后重试
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// 模拟连接到broker
async fn connect_to_broker(broker: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 尝试解析broker地址
    let addr: SocketAddr = broker.parse()
        .map_err(|_| format!("无效的broker地址: {}", broker))?;

    // 尝试建立TCP连接（模拟Kafka连接）
    match AsyncTcpStream::connect(&addr).await {
        Ok(_) => {
            tklog::async_info!("kafka|", &format!("成功建立连接: {}", broker));
            Ok(())
        }
        Err(e) => {
            tklog::async_error!("kafka|", &format!("连接失败: {} - {}", broker, e));
            Err(format!("连接到 {} 失败: {}", broker, e).into())
        }
    }
}

// 模拟Kafka消息接收
async fn simulate_kafka_message_reception() -> Result<Option<String>, Box<dyn std::error::Error>> {
    // 模拟随机消息接收失败（10%概率）
    if rand::random::<f32>() < 0.1 {
        return Err("模拟网络连接中断".into());
    }

    // 模拟JSON格式的Kafka消息
    let levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
    let level = levels[rand::random::<usize>() % levels.len()];
    
    let messages = [
        "系统启动成功",
        "数据库连接已建立",
        "用户登录请求处理中",
        "API调用成功",
        "文件上传完成",
        "定时任务执行",
        "缓存清理完成",
        "性能监控数据收集",
    ];
    
    let message = messages[rand::random::<usize>() % messages.len()];
    let json_message = format!(r#"{{"L":"{}","S":"{}"}}"#, level, message);
    
    Ok(Some(json_message))
}

// 处理Kafka消息
async fn process_kafka_message(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 解析JSON消息
    let kafka_msg: KafkaMessage = serde_json::from_str(message)
        .map_err(|e| format!("解析Kafka消息失败: {} - 原始消息: {}", e, message))?;

    let now = chrono::Local::now();
    let timestamp = now.format(TIMESTAMP_FORMAT).to_string();

    // 使用日志记录功能写入文件
    let result = log_with_level(&kafka_msg.l, &kafka_msg.s, &timestamp).await;
    
    match result {
        Ok(_) => {
            tklog::async_debug!("kafka|", &format!("成功处理消息: {:?}", kafka_msg));
            Ok(())
        }
        Err(e) => {
            tklog::async_error!("kafka|", &format!("写入日志失败: {}", e));
            Err(e)
        }
    }
}

// 判断是否为连接错误
fn is_connection_error(error_msg: &str) -> bool {
    let connection_errors = [
        "connection refused",
        "connection reset",
        "network unreachable",
        "timeout",
        "broken pipe",
        "连接失败",
        "网络中断",
        "连接中断",
    ];

    connection_errors.iter().any(|err| error_msg.to_lowercase().contains(err))
}
