#!/bin/bash

# 确保数据库目录存在
mkdir -p stepflow-sqlite/data

# 启动服务
echo "Starting StepFlow Gateway..."
cargo run --bin stepflow-gateway 