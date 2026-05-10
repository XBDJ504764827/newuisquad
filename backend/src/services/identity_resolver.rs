use std::collections::{HashMap, HashSet};
use serde::Serialize;
use sha2::{Sha256, Digest};
use sqlx::PgPool;

// ═══ Union-Find (并查集) ═══

struct UnionFind {
    parent: HashMap<String, String>,
    rank: HashMap<String, usize>,
}

impl UnionFind {
    fn new() -> Self {
        Self { parent: HashMap::new(), rank: HashMap::new() }
    }

    fn add(&mut self, x: &str) {
        if !self.parent.contains_key(x) {
            self.parent.insert(x.to_string(), x.to_string());
            self.rank.insert(x.to_string(), 0);
        }
    }

    fn find(&mut self, x: &str) -> String {
        let p = self.parent.get(x).cloned().unwrap_or_else(|| x.to_string());
        if p != x {
            let root = self.find(&p);
            self.parent.insert(x.to_string(), root.clone());
            root
        } else {
            p
        }
    }

    fn union(&mut self, x: &str, y: &str) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y { return; }

        let rank_x = *self.rank.get(&root_x).unwrap_or(&0);
        let rank_y = *self.rank.get(&root_y).unwrap_or(&0);

        if rank_x < rank_y {
            self.parent.insert(root_x.clone(), root_y);
        } else if rank_x > rank_y {
            self.parent.insert(root_y, root_x);
        } else {
            self.parent.insert(root_y, root_x.clone());
            *self.rank.entry(root_x).or_insert(0) += 1;
        }
    }

    fn all_elements(&self) -> Vec<String> {
        self.parent.keys().cloned().collect()
    }
}

// ═══ Identity Data ═══

#[derive(Debug, Clone, Serialize)]
pub struct PlayerIdentity {
    pub canonical_id: String,
    pub primary_steam_id: String,
    pub primary_eos_id: String,
    pub primary_name: String,
    pub all_steam_ids: Vec<String>,
    pub all_eos_ids: Vec<String>,
    pub all_names: Vec<String>,
    pub total_sessions: i32,
    pub first_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
struct IdentifierPair {
    steam_id: String,
    eos_id: String,
    player_name: String,
    first_seen: Option<chrono::DateTime<chrono::Utc>>,
    last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

// ═══ Identity Resolver ═══

pub struct IdentityResolver;

impl IdentityResolver {
    /// Compute identities from PostgreSQL event tables
    pub async fn compute(pool: &PgPool) -> Result<usize, String> {
        let pairs = Self::fetch_pairs(pool).await?;
        if pairs.is_empty() { return Ok(0); }

        let mut uf = UnionFind::new();

        // Build Union-Find from pairs
        for pair in &pairs {
            if !pair.steam_id.is_empty() {
                uf.add(&format!("steam:{}", pair.steam_id));
            }
            if !pair.eos_id.is_empty() {
                uf.add(&format!("eos:{}", pair.eos_id));
            }
            if !pair.player_name.is_empty() && pair.player_name != "-" {
                uf.add(&format!("name:{}", pair.player_name.to_lowercase()));
            }
            // Union steam↔eos
            if !pair.steam_id.is_empty() && !pair.eos_id.is_empty() {
                uf.union(
                    &format!("steam:{}", pair.steam_id),
                    &format!("eos:{}", pair.eos_id),
                );
            }
            // Union steam↔name
            if !pair.steam_id.is_empty() && !pair.player_name.is_empty() && pair.player_name != "-" {
                uf.union(
                    &format!("steam:{}", pair.steam_id),
                    &format!("name:{}", pair.player_name.to_lowercase()),
                );
            }
        }

        // Group by canonical root
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        for elem in uf.all_elements() {
            let root = uf.find(&elem);
            groups.entry(root).or_default().push(elem);
        }

        // Build identities
        let identities = Self::build_identities(&groups, &pairs);
        let count = identities.len();

        // Write to database
        Self::save_identities(pool, &identities).await?;

        Ok(count)
    }

    /// Collect identifier pairs from event tables
    async fn fetch_pairs(pool: &PgPool) -> Result<Vec<IdentifierPair>, String> {
        let mut pairs = Vec::new();

        // 1. player_info table (direct steam/eos/name + first_seen/last_seen)
        let rows = sqlx::query_as::<_, (String, String, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT COALESCE(steam64,''), COALESCE(eos_id,''), COALESCE(player_name,''),
                    first_seen, last_seen FROM player_info WHERE steam64 != '' OR eos_id != ''"
        ).fetch_all(pool).await.map_err(|e| e.to_string())?;
        for (steam, eos, name, fs, ls) in rows {
            pairs.push(IdentifierPair { steam_id: steam, eos_id: eos, player_name: name, first_seen: fs, last_seen: ls });
        }

        // 2. kill_events (attacker → attacker's steam and name tied together)
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT DISTINCT attacker_steam64, COALESCE(attacker_eos,''), attacker_name
             FROM kill_events WHERE attacker_steam64 != '' AND attacker_name != ''"
        ).fetch_all(pool).await.map_err(|e| e.to_string())?;
        for (steam, eos, name) in rows {
            pairs.push(IdentifierPair { steam_id: steam, eos_id: eos, player_name: name, first_seen: None, last_seen: None });
        }

        // 3. kill_events (victim → victim's steam and name tied together)
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT DISTINCT victim_steam64, COALESCE(victim_eos,''), victim_name
             FROM kill_events WHERE victim_steam64 != '' AND victim_name != ''"
        ).fetch_all(pool).await.map_err(|e| e.to_string())?;
        for (steam, eos, name) in rows {
            pairs.push(IdentifierPair { steam_id: steam, eos_id: eos, player_name: name, first_seen: None, last_seen: None });
        }

        // 4. chat_messages (steam → name)
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT DISTINCT steam64, player_name FROM chat_messages WHERE steam64 != '' AND player_name != ''"
        ).fetch_all(pool).await.map_err(|e| e.to_string())?;
        for (steam, name) in rows {
            pairs.push(IdentifierPair { steam_id: steam, eos_id: String::new(), player_name: name, first_seen: None, last_seen: None });
        }

        // 5. kill_events cross-reference (same event links attacker↔victim)
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT DISTINCT a.attacker_steam64, a.victim_steam64
             FROM kill_events a WHERE a.attacker_steam64 != '' AND a.victim_steam64 != ''
             AND a.attacker_steam64 != a.victim_steam64 LIMIT 100000"
        ).fetch_all(pool).await.map_err(|e| e.to_string())?;
        for (attacker, victim) in rows {
            pairs.push(IdentifierPair {
                steam_id: attacker,
                eos_id: String::new(),
                player_name: format!("linked_to_{}", victim),
                first_seen: None, last_seen: None,
            });
        }

        Ok(pairs)
    }

    /// Build identity clusters from groups
    fn build_identities(
        groups: &HashMap<String, Vec<String>>,
        pairs: &[IdentifierPair],
    ) -> Vec<PlayerIdentity> {
        let mut identities = Vec::new();

        for elements in groups.values() {
            let mut steam_ids = HashSet::new();
            let mut eos_ids = HashSet::new();
            let mut names = HashSet::new();
            let mut first_seen: Option<chrono::DateTime<chrono::Utc>> = None;
            let mut last_seen: Option<chrono::DateTime<chrono::Utc>> = None;
            let mut total_sessions = 0i32;

            for elem in elements {
                if let Some(steam) = elem.strip_prefix("steam:") {
                    steam_ids.insert(steam.to_string());
                } else if let Some(eos) = elem.strip_prefix("eos:") {
                    eos_ids.insert(eos.to_string());
                } else if let Some(_name) = elem.strip_prefix("name:") {
                    names.insert(elem["name:".len()..].to_string());
                }
            }

            // Enrich from pairs
            for pair in pairs {
                let in_cluster = (!pair.steam_id.is_empty() && steam_ids.contains(&pair.steam_id))
                    || (!pair.eos_id.is_empty() && eos_ids.contains(&pair.eos_id))
                    || (!pair.player_name.is_empty() && names.contains(&pair.player_name.to_lowercase()));

                if in_cluster {
                    if !pair.player_name.is_empty() && pair.player_name != "-" {
                        names.insert(pair.player_name.to_lowercase());
                    }
                    if let Some(fs) = pair.first_seen {
                        first_seen = Some(first_seen.map_or(fs, |old| old.min(fs)));
                    }
                    if let Some(ls) = pair.last_seen {
                        last_seen = Some(last_seen.map_or(ls, |old| old.max(ls)));
                    }
                    total_sessions += 1;
                }
            }

            let mut steam_list: Vec<String> = steam_ids.into_iter().collect();
            let mut eos_list: Vec<String> = eos_ids.into_iter().collect();
            steam_list.sort();
            eos_list.sort();

            let primary_steam = steam_list.first().cloned().unwrap_or_default();
            let primary_eos = eos_list.first().cloned().unwrap_or_default();
            let primary_name = names.iter().next().cloned().unwrap_or_default();

            let canonical_id = Self::generate_canonical_id(&steam_list, &eos_list);

            identities.push(PlayerIdentity {
                canonical_id,
                primary_steam_id: primary_steam,
                primary_eos_id: primary_eos,
                primary_name,
                all_steam_ids: steam_list,
                all_eos_ids: eos_list,
                all_names: names.into_iter().collect(),
                total_sessions,
                first_seen,
                last_seen,
            });
        }

        identities
    }

    /// SHA-256 hash of sorted "s:STEAM|e:EOS" → first 16 bytes hex
    fn generate_canonical_id(steam_ids: &[String], eos_ids: &[String]) -> String {
        let mut all: Vec<String> = Vec::new();
        for s in steam_ids { all.push(format!("s:{}", s)); }
        for e in eos_ids { all.push(format!("e:{}", e)); }
        all.sort();
        let combined = all.join("|");
        let hash = Sha256::digest(combined.as_bytes());
        hex::encode(&hash[..16])
    }

    /// Save identities to PostgreSQL
    async fn save_identities(pool: &PgPool, identities: &[PlayerIdentity]) -> Result<(), String> {
        // Truncate existing
        let _ = sqlx::query("DELETE FROM player_identity_lookup").execute(pool).await;
        let _ = sqlx::query("DELETE FROM player_identities").execute(pool).await;

        for identity in identities {
            let _ = sqlx::query(
                "INSERT INTO player_identities (canonical_id, primary_steam_id, primary_eos_id, primary_name,
                 all_steam_ids, all_eos_ids, all_names, total_sessions, first_seen, last_seen)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
            ).bind(&identity.canonical_id).bind(&identity.primary_steam_id).bind(&identity.primary_eos_id)
             .bind(&identity.primary_name).bind(&identity.all_steam_ids).bind(&identity.all_eos_ids)
             .bind(&identity.all_names).bind(identity.total_sessions).bind(identity.first_seen).bind(identity.last_seen)
             .execute(pool).await.map_err(|e| e.to_string())?;

            // Lookup entries
            for steam in &identity.all_steam_ids {
                let _ = sqlx::query(
                    "INSERT INTO player_identity_lookup (identifier_type, identifier_value, canonical_id) VALUES ('steam',$1,$2) ON CONFLICT DO NOTHING"
                ).bind(steam).bind(&identity.canonical_id).execute(pool).await;
            }
            for eos in &identity.all_eos_ids {
                let _ = sqlx::query(
                    "INSERT INTO player_identity_lookup (identifier_type, identifier_value, canonical_id) VALUES ('eos',$1,$2) ON CONFLICT DO NOTHING"
                ).bind(eos).bind(&identity.canonical_id).execute(pool).await;
            }
            for name in &identity.all_names {
                let _ = sqlx::query(
                    "INSERT INTO player_identity_lookup (identifier_type, identifier_value, canonical_id) VALUES ('name',$1,$2) ON CONFLICT DO NOTHING"
                ).bind(name).bind(&identity.canonical_id).execute(pool).await;
            }
        }
        Ok(())
    }

    /// Lookup identity by any identifier
    pub async fn lookup(pool: &PgPool, identifier: &str) -> Result<Option<PlayerIdentity>, String> {
        let canonical = sqlx::query_scalar::<_, String>(
            "SELECT canonical_id FROM player_identity_lookup WHERE identifier_value = $1 LIMIT 1"
        ).bind(identifier).fetch_optional(pool).await.map_err(|e| e.to_string())?;

        match canonical {
            Some(cid) => {
                let row = sqlx::query_as::<_, (String, String, String, String, Vec<String>, Vec<String>, Vec<String>, i32, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
                    "SELECT canonical_id, primary_steam_id, primary_eos_id, primary_name, all_steam_ids, all_eos_ids, all_names, total_sessions, first_seen, last_seen FROM player_identities WHERE canonical_id=$1"
                ).bind(&cid).fetch_one(pool).await.map_err(|e| e.to_string())?;
                Ok(Some(PlayerIdentity {
                    canonical_id: row.0, primary_steam_id: row.1, primary_eos_id: row.2, primary_name: row.3,
                    all_steam_ids: row.4, all_eos_ids: row.5, all_names: row.6,
                    total_sessions: row.7, first_seen: row.8, last_seen: row.9,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get all identities (paginated)
    pub async fn list_all(pool: &PgPool, page: i64, per_page: i64) -> Result<(Vec<PlayerIdentity>, i64), String> {
        let offset = (page - 1) * per_page;
        let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM player_identities")
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

        let rows = sqlx::query_as::<_, (String, String, String, String, Vec<String>, Vec<String>, Vec<String>, i32, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
            "SELECT canonical_id, primary_steam_id, primary_eos_id, primary_name, all_steam_ids, all_eos_ids, all_names, total_sessions, first_seen, last_seen FROM player_identities ORDER BY last_seen DESC NULLS LAST LIMIT $1 OFFSET $2"
        ).bind(per_page).bind(offset).fetch_all(pool).await.map_err(|e| e.to_string())?;

        let identities = rows.into_iter().map(|row| PlayerIdentity {
            canonical_id: row.0, primary_steam_id: row.1, primary_eos_id: row.2, primary_name: row.3,
            all_steam_ids: row.4, all_eos_ids: row.5, all_names: row.6,
            total_sessions: row.7, first_seen: row.8, last_seen: row.9,
        }).collect();

        Ok((identities, total))
    }

    /// Find related accounts for a Steam ID
    pub async fn find_related(pool: &PgPool, steam_id: &str) -> Result<Vec<PlayerIdentity>, String> {
        match Self::lookup(pool, steam_id).await? {
            Some(identity) => Ok(vec![identity]),
            None => Ok(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_find_basic() {
        let mut uf = UnionFind::new();
        uf.add("a"); uf.add("b"); uf.add("c");
        uf.union("a", "b");
        uf.union("b", "c");
        assert_eq!(uf.find("a"), uf.find("c"));
    }

    #[test]
    fn test_canonical_id_deterministic() {
        let id1 = IdentityResolver::generate_canonical_id(
            &["76561198000000001".to_string()],
            &["eos0001".to_string()],
        );
        let id2 = IdentityResolver::generate_canonical_id(
            &["76561198000000001".to_string()],
            &["eos0001".to_string()],
        );
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 32);
    }

    #[test]
    fn test_canonical_id_different() {
        let id1 = IdentityResolver::generate_canonical_id(
            &["76561198000000001".to_string()],
            &[],
        );
        let id2 = IdentityResolver::generate_canonical_id(
            &["76561198000000002".to_string()],
            &[],
        );
        assert_ne!(id1, id2);
    }
}
