use anyhow::Result;
use serde::Serialize;
use spec_engine::get_spec_catalog;
use std::collections::HashSet;

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
/// # 返回
/// 协议列表，包括每个协议的DI数量和支持的区域
pub fn list_protocols() -> Result<ListProtocolsOutput> {
    let catalog = get_spec_catalog();
    
    // 按协议分组统计
    let mut protocol_map: std::collections::HashMap<String, (HashSet<u32>, HashSet<String>)> = 
        std::collections::HashMap::new();

    for ((protocol, di, region, _dir), _field) in catalog.iter() {
        let entry = protocol_map.entry(protocol.clone()).or_insert_with(|| {
            (HashSet::new(), HashSet::new())
        });
        entry.0.insert(*di);
        entry.1.insert(region.clone());
    }

    let mut protocols: Vec<ProtocolInfo> = protocol_map
        .into_iter()
        .map(|(name, (dis, regions))| {
            let mut regions_vec: Vec<String> = regions.into_iter().collect();
            regions_vec.sort();
            
            ProtocolInfo {
                name,
                di_count: dis.len(),
                regions: regions_vec,
            }
        })
        .collect();

    protocols.sort_by(|a, b| a.name.cmp(&b.name));
    let total = protocols.len();

    Ok(ListProtocolsOutput { protocols, total })
}
