use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const PACKET_PREFIX: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

pub struct SquadRcon {
    stream: TcpStream,
}

impl SquadRcon {
    pub async fn connect(ip: &str, port: u16, password: &str) -> Result<Self, String> {
        let addr = format!("{}:{}", ip, port);
        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("RCON 连接失败: {}", e))?;

        let auth_packet = build_packet(3, password);
        stream.write_all(&auth_packet).await.map_err(|e| format!("发送认证包失败: {}", e))?;

        Ok(Self { stream })
    }

    pub async fn execute(&mut self, command: &str) -> Result<String, String> {
        let cmd_packet = build_packet(2, command);
        self.stream.write_all(&cmd_packet).await.map_err(|e| format!("发送命令失败: {}", e))?;

        // 读取响应：先读取 12 字节头部（size + id + type），再根据 size 读取剩余 body
        let mut header = [0u8; 12];
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.stream.read_exact(&mut header)
        )
        .await
        .map_err(|_| "RCON 读取超时".to_string())?
        .map_err(|e| format!("读取响应头失败: {}", e))?;

        let size = i32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;
        if size < 10 {
            return Err("无效的 RCON 响应".to_string());
        }

        // body = size - 8 (id + type already read) = size - 8 bytes remaining
        let body_len = size.saturating_sub(8);
        let mut body_buf = vec![0u8; body_len];

        if body_len > 0 {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.stream.read_exact(&mut body_buf)
            )
            .await
            .map_err(|_| "RCON 读取 body 超时".to_string())?
            .map_err(|e| format!("读取响应 body 失败: {}", e))?;
        }

        // 去除尾部 null 字节
        let body_end = body_buf.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        String::from_utf8(body_buf[..body_end].to_vec())
            .map_err(|e| format!("解析响应失败: {}", e))
    }
}

fn build_packet(packet_type: i32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let size = (10 + body_bytes.len()) as i32;
    let mut packet = Vec::with_capacity(4 + size as usize);

    packet.extend_from_slice(&size.to_le_bytes());
    packet.extend_from_slice(&PACKET_PREFIX);
    packet.extend_from_slice(&packet_type.to_le_bytes());
    packet.extend_from_slice(body_bytes);
    packet.push(0x00);
    packet.push(0x00);

    packet
}
