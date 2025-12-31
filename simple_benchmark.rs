use std::time::{Duration, Instant};
use std::io::Write;

fn main() {
    println!("日志服务器简单性能基准测试");
    println!("========================");
    
    // 测试字符串格式化性能
    println!("\n1. 字符串格式化性能测试:");
    test_string_formatting_performance();
    
    // 测试路径构建性能
    println!("\n2. 路径构建性能测试:");
    test_path_building_performance();
    
    // 测试文件写入性能
    println!("\n3. 文件写入性能测试:");
    test_file_write_performance();
    
    println!("\n性能基准测试完成!");
}

fn test_string_formatting_performance() {
    let iterations = 10000;
    
    // 测试 format! 宏的性能
    let start = Instant::now();
    for i in 0..iterations {
        let _ = format!("[{}] [{}] {}", "2024-01-01 12:00:00", "I", format!("日志消息 #{}", i));
    }
    let duration = start.elapsed();
    println!("format! 宏 {} 次调用耗时: {:?}", iterations, duration);
    
    // 测试简单字符串拼接的性能
    let start = Instant::now();
    for i in 0..iterations {
        let level = "I";
        let timestamp = "2024-01-01 12:00:00";
        let message = format!("日志消息 #{}", i);
        let _ = format!("[{}] [{}] {}", timestamp, level, message);
    }
    let duration = start.elapsed();
    println!("字符串拼接 {} 次调用耗时: {:?}", iterations, duration);
}

fn test_path_building_performance() {
    let iterations = 10000;
    
    // 测试使用 format! 构建路径的性能
    let start = Instant::now();
    for i in 0..iterations {
        let year = "2024";
        let month = "01";
        let day = "01";
        let hour = "12";
        let _ = format!("logs/{}/{}/{}/{}.log", year, month, day, hour);
    }
    let duration = start.elapsed();
    println!("format! 路径构建 {} 次调用耗时: {:?}", iterations, duration);
    
    // 测试使用 PathBuf 构建路径的性能
    let start = Instant::now();
    for i in 0..iterations {
        use std::path::PathBuf;
        let mut path = PathBuf::from("logs");
        path.push("2024");
        path.push("01");
        path.push("01");
        path.push(format!("{}.log", "12"));
        let _ = path;
    }
    let duration = start.elapsed();
    println!("PathBuf 路径构建 {} 次调用耗时: {:?}", iterations, duration);
}

fn test_file_write_performance() {
    let iterations = 1000; // 减少迭代次数，因为文件操作较慢
    
    // 创建测试目录
    std::fs::create_dir_all("benchmark_test").ok();
    
    // 测试直接写入文件的性能
    let start = Instant::now();
    for i in 0..iterations {
        let test_file = format!("benchmark_test/test_{}.log", i % 10);
        let content = format!("[2024-01-01 12:00:00] [I] 测试日志消息 #{}\n", i);
        
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&test_file)
            .and_then(|mut file| {
                std::io::Write::write_all(&mut file, content.as_bytes())
            });
    }
    let duration = start.elapsed();
    println!("文件写入 {} 次调用耗时: {:?}", iterations, duration);
    
    // 清理测试目录
    std::fs::remove_dir_all("benchmark_test").ok();
    
    // 测试使用 write! 宏的性能
    let start = Instant::now();
    for i in 0..iterations {
        let test_file = format!("benchmark_test/test_{}.log", i % 10);
        
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&test_file)
            .and_then(|mut file| {
                write!(file, "[2024-01-01 12:00:00] [I] 测试日志消息 #{}\n", i)
            });
    }
    let duration = start.elapsed();
    println!("write! 文件写入 {} 次调用耗时: {:?}", iterations, duration);
    
    // 清理测试目录
    std::fs::remove_dir_all("benchmark_test").ok();
}