use std::env;
use std::io::BufRead;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: import_log <server_id> <log_file_path>");
        return Ok(());
    }
    let server_id: i32 = args[1].parse()?;
    let log_path = &args[2];

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/admin_console".into());
    let pool = sqlx::PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("数据库迁移完成，开始导入...");

    let file = std::fs::File::open(log_path)?;
    let reader = std::io::BufReader::new(file);
    let mut count = 0u64;
    let mut fly_count = 0u64;
    let mut kill_count = 0u64;
    let mut player_count = 0u64;

    for line in reader.lines() {
        let line = match line { Ok(l) => l, Err(_) => continue };
        if line.trim().is_empty() { continue; }
        count += 1;

        if let Some(event) = admin_console_backend::services::squad_log_parser::parse_line(&line) {
            match event {
                admin_console_backend::services::squad_log_parser::ParsedEvent::PlayerLogin { player_name, eos_id, steam64, ip, logged_at } => {
                    if !steam64.is_empty() {
                        let _ = sqlx::query(
                            "INSERT INTO player_info (server_id, player_name, steam64, eos_id, ip, first_seen, last_seen) VALUES ($1,$2,$3,$4,$5,$6,$6) ON CONFLICT DO NOTHING"
                        ).bind(server_id).bind(&player_name).bind(&steam64).bind(&eos_id).bind(&ip).bind(logged_at).execute(&pool).await;
                        let _ = sqlx::query(
                            "UPDATE player_info SET player_name=$1, eos_id=$2, ip=$3, last_seen=$4 WHERE server_id=$5 AND steam64=$6"
                        ).bind(&player_name).bind(&eos_id).bind(&ip).bind(logged_at).bind(server_id).bind(&steam64).execute(&pool).await;
                        player_count += 1;
                    }
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::FlyEvent { player_name, eos_id, steam64, event_type, logged_at } => {
                    let _ = sqlx::query(
                        "INSERT INTO fly_events (server_id, player_name, eos_id, steam64, event_type, logged_at) VALUES ($1,$2,$3,$4,$5,$6)"
                    ).bind(server_id).bind(&player_name).bind(&eos_id).bind(&steam64).bind(&event_type).bind(logged_at).execute(&pool).await;
                    fly_count += 1;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::KillEvent { .. } => { kill_count += 1; }
                admin_console_backend::services::squad_log_parser::ParsedEvent::TeamAssignment { player_name, steam64, team_id, logged_at } => {
                    let _ = sqlx::query("INSERT INTO team_assignments (server_id, player_name, steam64, team_id, logged_at) VALUES ($1,$2,$3,$4,$5)").bind(server_id).bind(&player_name).bind(&steam64).bind(team_id).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::SquadCreation { player_name, steam64, squad_id, squad_name, faction, logged_at } => {
                    let _ = sqlx::query("INSERT INTO squad_creations (server_id, player_name, steam64, squad_id, squad_name, faction, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7)").bind(server_id).bind(&player_name).bind(&steam64).bind(&squad_id).bind(&squad_name).bind(&faction).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::MatchEvent { map_name, layer_name, team1_faction, team2_faction, winner_team, event_type, logged_at } => {
                    let _ = sqlx::query("INSERT INTO match_info (server_id, map_name, layer_name, team1_faction, team2_faction, winner_team, event_type, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)").bind(server_id).bind(&map_name).bind(&layer_name).bind(&team1_faction).bind(&team2_faction).bind(winner_team).bind(&event_type).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::DeployRole { player_name, steam64, logged_at, .. } => {
                    let _ = sqlx::query("UPDATE player_info SET player_name=$1 WHERE server_id=$2 AND steam64=$3 AND player_name=''").bind(&player_name).bind(server_id).bind(&steam64).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::ReviveEvent { reviver_name, reviver_steam64, revived_name, revived_steam64, logged_at } => {
                    let _ = sqlx::query("INSERT INTO revive_events (server_id, reviver_name, reviver_steam64, revived_name, revived_steam64, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(server_id).bind(&reviver_name).bind(&reviver_steam64).bind(&revived_name).bind(&revived_steam64).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::VehicleEvent { player_name, steam64, vehicle_name, event_type, logged_at } => {
                    let _ = sqlx::query("INSERT INTO vehicle_events (server_id, player_name, steam64, vehicle_name, event_type, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(server_id).bind(&player_name).bind(&steam64).bind(&vehicle_name).bind(&event_type).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::AdminAction { admin_name, action_type, target, message, raw_line, logged_at } => {
                    let _ = sqlx::query("INSERT INTO admin_actions (server_id, admin_name, action_type, target, message, raw_line, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7)").bind(server_id).bind(&admin_name).bind(&action_type).bind(&target).bind(&message).bind(&raw_line).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::PlayerDeath { player_name, steam64, killer_steam64, weapon, logged_at } => {
                    let _ = sqlx::query("INSERT INTO kill_events (server_id, attacker_name, attacker_steam64, victim_name, damage, weapon, is_kill, is_teamkill, logged_at) VALUES ($1,'',$2,$3,0,$4,true,false,$5)").bind(server_id).bind(&killer_steam64).bind(&player_name).bind(&weapon).bind(logged_at).execute(&pool).await;
                }
                admin_console_backend::services::squad_log_parser::ParsedEvent::ChatMessage { player_name, steam64, message, channel, logged_at } => {
                    let _ = sqlx::query("INSERT INTO chat_messages (server_id, player_name, steam64, message, channel, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(server_id).bind(&player_name).bind(&steam64).bind(&message).bind(&channel).bind(logged_at).execute(&pool).await;
                }
                _ => {}
            }
        }
        if count % 10000 == 0 { println!("已处理 {} 行...", count); }
    }

    println!("完成! 总行数: {}, 玩家: {}, 飞天: {}, 击倒: {}", count, player_count, fly_count, kill_count);
    Ok(())
}
