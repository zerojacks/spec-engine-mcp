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
/// - catalog: DynamicCatalog 引用
///
/// # 返回
/// 协议列表，包括每个协议的DI数量和支持的区域
pub fn list_protocols(catalog: &spec_engine::DynamicCatalog) -> Result<ListProtocolsOutput> {
    // 使用 DynamicCatalog 的新方法
    let protocol_names = catalog.list_protocols();
    
    let mut protocols = Vec::new();
    for protocol_name in protocol_names {
        let dis = catalog.list_dis_for_protocol(&protocol_name);
        let regions = catalog.list_regions_for_protocol(&protocol_name);
        
        protocols.push(ProtocolInfo {
            name: protocol_name,
            di_count: dis.len(),
            regions,
        });
    }

    protocols.sort_by(|a, b| a.name.cmp(&b.name));
    let total = protocols.len();

    Ok(ListProtocolsOutput { protocols, total })
}
