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

        let mut buf = vec![0u8; 4096];
        let n = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.stream.read(&mut buf)
        )
        .await
        .map_err(|_| "RCON 读取超时".to_string())?
        .map_err(|e| format!("读取响应失败: {}", e))?;

        String::from_utf8(buf[..n].to_vec())
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
