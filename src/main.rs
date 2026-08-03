mod server;
mod tools;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    // 初始化日志（输出到stderr，避免干扰stdio通信）
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    // 创建并运行MCP服务器
    let mut server = server::McpServer::new();
    server.run()?;

    Ok(())
}
