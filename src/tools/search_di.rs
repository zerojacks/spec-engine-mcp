use anyhow::Result;
use serde::{Deserialize, Serialize};

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
/// - engine: Engine 引用
/// - input: 搜索参数
///
/// # 返回
/// 匹配的DI列表
pub fn search_di(engine: &spec_engine::Engine, input: SearchDiInput) -> Result<SearchDiOutput> {
    // 使用 Engine 的搜索方法
    let search_results = engine.search_di(
        &input.keyword,
        input.protocol.as_deref(),
        input.region.as_deref(),
        input.limit + 1, // 多取一个来判断是否有更多结果
    );

    let has_more = search_results.len() > input.limit;
    
    let results: Vec<DiSearchResult> = search_results
        .into_iter()
        .take(input.limit)
        .map(|(protocol, di, region, name)| DiSearchResult {
            di: format!("{:08X}", di),
            name,
            protocol,
            region,
        })
        .collect();

    Ok(SearchDiOutput {
        count: results.len(),
        has_more,
        results,
    })
}
