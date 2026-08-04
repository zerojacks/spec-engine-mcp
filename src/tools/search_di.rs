use anyhow::Result;
use serde::{Deserialize, Serialize};
use spec_engine::DEFAULT_REGION;

#[derive(Debug, Deserialize)]
pub struct SearchDiInput {
    /// 搜索关键词
    pub keyword: String,
    /// 可选的协议过滤
    #[serde(default)]
    pub protocol: Option<String>,
    /// 可选的区域过滤
    #[serde(default)]
    pub region: Option<String>,
    /// 最大返回结果数量
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct SearchDiOutput {
    /// 搜索结果
    pub results: Vec<DiSearchResult>,
    /// 结果总数（可能被limit限制）
    pub count: usize,
    /// 是否有更多结果
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct DiSearchResult {
    /// DI码（十六进制）
    pub di: String,
    /// DI名称
    pub name: String,
    /// 协议
    pub protocol: String,
    /// 区域
    pub region: String,
}

/// 搜索DI定义
///
/// # 输入参数
/// - catalog: DynamicCatalog 引用
/// - input: 搜索参数
///
/// # 返回
/// 匹配的DI列表
pub fn search_di(catalog: &spec_engine::DynamicCatalog, input: SearchDiInput) -> Result<SearchDiOutput> {
    let keyword_lower = input.keyword.to_lowercase();

    let mut results = Vec::new();
    let mut total_matches = 0;

    for ((protocol, di, region, _dir), field) in catalog.iter_all() {
        // 应用过滤条件
        if let Some(ref proto_filter) = input.protocol {
            if protocol != proto_filter {
                continue;
            }
        }

        if let Some(ref region_filter) = input.region {
            if region != region_filter && region != DEFAULT_REGION {
                continue;
            }
        }

        // 关键词匹配（不区分大小写）
        let name_lower = field.name.to_lowercase();
        if name_lower.contains(&keyword_lower) {
            total_matches += 1;

            if results.len() < input.limit {
                results.push(DiSearchResult {
                    di: format!("{:08X}", di),
                    name: field.name.clone(),
                    protocol: protocol.clone(),
                    region: region.clone(),
                });
            }
        }
    }

    // 按DI码排序
    results.sort_by(|a, b| a.di.cmp(&b.di));

    Ok(SearchDiOutput {
        count: results.len(),
        has_more: total_matches > results.len(),
        results,
    })
}
