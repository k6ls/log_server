# 使用Rust官方镜像进行Linux编译
FROM rust:1.75-slim as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 编译项目
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

# 使用小的运行时镜像
FROM alpine:3.19

# 创建非root用户
RUN addgroup -g 1001 logserver && \
    adduser -D -s /bin/sh -u 1001 -G logserver logserver

# 复制编译好的二进制文件
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/log_server /usr/local/bin/log_server

# 设置权限
RUN chown logserver:logserver /usr/local/bin/log_server && \
    chmod +x /usr/local/bin/log_server

# 切换到非root用户
USER logserver

# 暴露端口（如果需要）
# EXPOSE 8080

# 运行程序
CMD ["log_server"]