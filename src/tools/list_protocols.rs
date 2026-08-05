use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ListProtocolsOutput {
    /// 支持的协议列表
    pub protocols: Vec<ProtocolInfo>,
    /// 协议总数
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ProtocolInfo {
    /// 协议名称
    pub name: String,
    /// DI条目数量
    pub di_count: usize,
    /// 支持的区域
    pub regions: Vec<String>,
}

/// 列出所有支持的协议
///
/// # 参数
/// - engine: Engine 引用
///
/// # 返回
/// 协议列表，包括每个协议的DI数量和支持的区域
pub fn list_protocols(engine: &spec_engine::Engine) -> Result<ListProtocolsOutput> {
    // 使用 Engine 的新方法
    let protocol_infos = engine.list_protocols();
    
    let protocols: Vec<ProtocolInfo> = protocol_infos
        .into_iter()
        .map(|(name, di_count, regions)| ProtocolInfo {
            name,
            di_count,
            regions,
        })
        .collect();

    let total = protocols.len();

    Ok(ListProtocolsOutput { protocols, total })
}
