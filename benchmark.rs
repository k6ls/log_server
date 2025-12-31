use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn main() {
    println!("日志服务器性能基准测试");
    println!("========================");
    
    let rt = Runtime::new().unwrap();
    
    // 测试1：单线程性能测试
    println!("\n1. 单线程日志写入性能测试:");
    rt.block_on(async {
        let start = Instant::now();
        
        for i in 0..10000 {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let content = format!("基准测试消息 #{} - 测试性能优化效果", i);
            let _ = log_with_level_optimized("INFO", &content, &timestamp).await;
        }
        
        let duration = start.elapsed();
        println!("写入10000条日志耗时: {:?}", duration);
        println!("平均每条日志耗时: {:?}", duration / 10000);
    });
    
    // 测试2：并发性能测试
    println!("\n2. 并发日志写入性能测试:");
    rt.block_on(async {
        let start = Instant::now();
        
        let mut handles = Vec::new();
        for task_id in 0..10 {
            handles.push(tokio::spawn(async move {
                for i in 0..1000 {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let content = format!("并发测试任务{} 消息#{} - 测试并发性能", task_id, i);
                    let _ = log_with_level_optimized("INFO", &content, &timestamp).await;
                }
            }));
        }
        
        for handle in handles {
            let _ = handle.await;
        }
        
        let duration = start.elapsed();
        println!("10个任务并发写入10000条日志耗时: {:?}", duration);
        println!("平均每条日志耗时: {:?}", duration / 10000);
    });
    
    println!("\n性能基准测试完成!");
}

// 简化的日志写入函数（用于基准测试）
async fn log_with_level_optimized(level: &str, content: &str, timestamp: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    
    // 预构建目录路径
    let now = chrono::Local::now();
    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();
    let day = now.format("%d").to_string();
    let hour = now.format("%H").to_string();
    
    // 使用PathBuf来构建路径，减少字符串操作
    let mut log_dir = PathBuf::from("logs/benchmark");
    log_dir.push(&year);
    log_dir.push(&month);
    log_dir.push(&day);
    
    let mut log_file = log_dir.clone();
    log_file.push(format!("{}.log", hour));
    
    // 确保目录存在
    let _ = fs::create_dir_all(&log_dir);
    
    // 使用write!直接写入文件，避免format!的中间字符串分配
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .and_then(|mut file| {
            write!(file, "[{}] [{}] {}\n", timestamp, get_level_abbreviation(level), content)
        });
    
    Ok(())
}

fn get_level_abbreviation(level: &str) -> &str {
    match level {
        "TRACE" => "T",
        "DEBUG" => "D",
        "INFO" => "I", 
        "WARN" => "W",
        "ERROR" => "E",
        "FATAL" => "F",
        _ => "I",
    }
}