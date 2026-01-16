use std::{
	io::Cursor,
	sync::Arc,
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use bytes::{Buf, BufMut, BytesMut};
use protocol::{
	codec::{
		HytaleCodec,
		VarInt,
	},
	packets::*,
};
use quinn::{
	Connection,
	RecvStream,
	SendStream,
};
use tokio::io::AsyncReadExt;
use tracing::{
	debug,
	error,
	info,
	warn,
};
use uuid::Uuid;

use crate::auth::ServerAuthManager;

const EXPECTED_HASH: &str = "6708f121966c1c443f4b0eb525b2f81d0a8dc61f5003a692a8fa157e5e02cea9";

pub struct PlayerConnection {
	conn: Connection,
	send: SendStream,
	recv: RecvStream,
	auth: Arc<ServerAuthManager>,
	pub username: String,
	pub uuid: Uuid,
}

impl PlayerConnection {
	pub fn new(conn: Connection, send: SendStream, recv: RecvStream, auth: Arc<ServerAuthManager>) -> Self {
		Self {
			conn,
			send,
			recv,
			auth,
			username: String::new(),
			uuid: Uuid::nil(),
		}
	}

	pub async fn run(mut self) -> Result<()> {
		info!("New connection from {}", self.conn.remote_address());

		let (id, payload_vec) = self.read_frame().await?;

		if id != 0 {
			self.kick("Expected Connect Packet").await?;
			bail!("Protocol Error: Expected Connect(0), got {}", id);
		}

		let mut cursor = Cursor::new(payload_vec.as_slice());
		let connect = ConnectPacket::decode(&mut cursor)?;

		if connect.protocol_hash != EXPECTED_HASH {
			warn!("Protocol mismatch: {}", connect.protocol_hash);
			self.kick("Protocol Version Mismatch").await?;
			bail!("Protocol Mismatch");
		}

		self.username = connect.username;
		self.uuid = connect.uuid;
		info!("Login Request: {} ({})", self.username, self.uuid);

		if let Some(token) = connect.identity_token {
			if let Err(e) = self.perform_online_auth(&token).await {
				error!("Auth failed for {}: {}", self.username, e);
				self.kick("Authentication Failed").await?;
				return Err(e);
			}
		} else {
			info!("Player {} connecting without token (Offline Mode)", self.username);
			self.send_packet(14, &ConnectAcceptPacket { password_challenge: None }).await?;
		}

		info!("Player {} authenticated.", self.username);

		while let Ok((_, _)) = self.read_frame().await {}

		Ok(())
	}

	async fn perform_online_auth(&mut self, player_token: &str) -> Result<()> {
		let server_session = self.auth.get_session_token().await.ok_or(anyhow!("Server not logged in (Offline)"))?;

		let server_id = self.auth.get_server_id().await;
		let server_identity = self.auth.get_identity_token().await;

		let auth_grant_str = self.auth.get_api().request_auth_grant(player_token, &server_session, &server_id).await?;

		self.send_packet(
			11,
			&AuthGrantPacket {
				auth_grant: Some(auth_grant_str),
				server_identity,
			},
		)
		.await?;

		let (id, payload_vec) = self.read_frame().await?;
		if id != 12 {
			bail!("Expected AuthToken(12), got {}", id);
		}

		let mut cursor = Cursor::new(payload_vec.as_slice());
		let auth_response = AuthTokenPacket::decode(&mut cursor)?;

		let server_grant = auth_response.server_grant.ok_or(anyhow!("Client missing server grant"))?;

		let fingerprint = self.auth.get_cert_fingerprint();
		let access_token = self.auth.get_api().exchange_grant(&server_grant, fingerprint, &server_session).await?;

		self.send_packet(
			13,
			&ServerAuthTokenPacket {
				server_access_token: Some(access_token),
				password_challenge: None,
			},
		)
		.await?;

		Ok(())
	}

	// --- I/O Helpers ---

	async fn send_packet<T: HytaleCodec>(&mut self, id: i32, packet: &T) -> Result<()> {
		let mut payload = BytesMut::new();
		packet.encode(&mut payload);

		let mut header = BytesMut::new();
		header.put_i32_le(payload.len() as i32);
		header.put_i32_le(id);

		self.send.write_all(&header).await?;
		self.send.write_all(&payload).await?;
		Ok(())
	}

	async fn kick(&mut self, reason: &str) -> Result<()> {
		let _ = self
			.send_packet(
				1,
				&DisconnectPacket {
					reason: reason.to_string(),
					type_id: VarInt(0),
				},
			)
			.await;
		self.conn.close(0u32.into(), b"Kicked");
		Ok(())
	}

	/// Reads length prefix, reads body, parses ID.
	/// Returns (PacketID, BodyBytes).
	async fn read_frame(&mut self) -> Result<(i32, Vec<u8>)> {
		let len = self.recv.read_i32_le().await? as usize;
		let id = self.recv.read_i32_le().await?;

		if len == 0 {
			bail!("Invalid Packet Length: 0");
		} else if len > 1677721600 {
			bail!("Invalid Packet Length: {}", len);
		}

		let mut buf = vec![0u8; len];
		self.recv.read_exact(&mut buf).await?;

		#[cfg(debug_assertions)]
		{
			let hex: Vec<String> = buf.iter().map(|b| format!("{:02X}", b)).collect();
			info!("RX Frame: Len={} Bytes=[{}]", len, hex.join(" "));
		}

		Ok((id, buf))
	}
}
