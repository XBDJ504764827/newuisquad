use std::collections::HashMap;

/// 通过 Steam API 批量获取玩家名称
pub async fn fetch_player_names(api_key: &str, steam_ids: &[String]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if api_key.is_empty() || steam_ids.is_empty() {
        return result;
    }

    // 每次最多查询 100 个
    for chunk in steam_ids.chunks(100) {
        let ids = chunk.join(",");
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            api_key, ids
        );

        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(players) = json["response"]["players"].as_array() {
                        for p in players {
                            let steamid = p["steamid"].as_str().unwrap_or("").to_string();
                            let name = p["personaname"].as_str().unwrap_or("").to_string();
                            if !steamid.is_empty() {
                                result.insert(steamid, name);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Steam API 查询失败: {}", e);
            }
        }
    }

    result
}
