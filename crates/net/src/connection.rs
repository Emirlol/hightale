use std::sync::Arc;

use anyhow::{
	anyhow,
	bail,
	Result,
};
use bytes::{
	Buf,
	BufMut,
	BytesMut,
};
use protocol::{
	codec::FixedAscii,
	v2,
	v2::{
		auth::{
			AuthGrant,
			ConnectAccept,
			ServerAuthToken,
		},
		connection::{
			Disconnect,
			DisconnectType,
		},
		Packet,
	},
};
use quinn::{
	Connection,
	RecvStream,
	SendStream,
};
use tokio::io::AsyncReadExt;
use tracing::{
	error,
	info,
	warn,
};
use uuid::Uuid;

use crate::auth::ServerAuthManager;

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

		let connect = match self.read_packet().await {
			Ok(Packet::Connect(payload)) => payload,
			Ok(packet) => {
				self.kick("Expected Connect Packet").await?;
				bail!("Protocol Error: Expected Connect(0), got {}", packet.id());
			}
			Err(e) => {
				error!("Error reading Connect packet: {}", e);
				self.kick("Internal Error").await?;
				return Err(e);
			}
		};

		if connect.protocol_crc != v2::PROTOCOL_CRC {
			warn!(
				"Client {} has incompatible protocol CRC: expected {:08X}, got {:08X}",
				connect.username,
				v2::PROTOCOL_CRC,
				connect.protocol_crc
			);
			self.kick("Incompatible Client Version").await?;
			return Err(anyhow!("Incompatible protocol CRC"));
		}

		if connect.protocol_build_number != v2::PROTOCOL_BUILD_NUMBER {
			warn!(
				"Client {} has incompatible protocol build number: expected {}, got {}",
				connect.username,
				v2::PROTOCOL_BUILD_NUMBER,
				connect.protocol_build_number
			);
			self.kick("Incompatible Client Version").await?;
			return Err(anyhow!("Incompatible protocol build number"));
		}

		if connect.referral_data.is_some() {
			if let Some(host_addr) = connect.referral_source {
				if host_addr.host.is_empty() {
					warn!("Client {} sent referral data with empty referral source", connect.username);
					self.kick("Referral source address is invalid").await?;
					return Err(anyhow!("Invalid referral data"));
				}
			} else {
				warn!("Client {} sent referral data without referral source", connect.username);
				self.kick("Referral connections must include source server address").await?;
				return Err(anyhow!("Invalid referral data"));
			};
		}

		self.username = connect.username.to_string();
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
			self.send_packet(ConnectAccept { password_challenge: None }).await?;
		}

		info!("Player {} authenticated.", self.username);

		while let Ok(_) = self.read_packet().await {}

		Ok(())
	}

	async fn perform_online_auth(&mut self, player_token: &str) -> Result<()> {
		let server_session = self.auth.get_session_token().await.ok_or(anyhow!("Server not logged in (Offline)"))?;

		let server_id = self.auth.get_server_id().await;
		let server_identity = self.auth.get_identity_token().await;

		let auth_grant_str = self.auth.get_api().request_auth_grant(player_token, &server_session, &server_id).await?;

		self.send_packet(AuthGrant {
			auth_grant: Some(auth_grant_str),
			server_identity,
		})
		.await?;

		let auth_response = match self.read_packet().await {
			Ok(Packet::AuthToken(payload)) => payload,
			Ok(packet) => bail!("Protocol Error: Expected AuthToken(10), got {}", packet.id()),
			Err(e) => bail!("Error reading AuthToken packet: {}", e),
		};

		let server_grant = auth_response.server_grant.ok_or(anyhow!("Client missing server grant"))?;

		let fingerprint = self.auth.get_cert_fingerprint();

		info!("--- Auth Debug Start ---");
		info!("Server Grant from Client: '{}'", server_grant);
		info!("Cert Fingerprint (Debug): {:?}", fingerprint);
		let access_token = self.auth.get_api().exchange_grant(&server_grant, fingerprint, &server_session).await?;

		self.send_packet(ServerAuthToken {
			server_access_token: Some(access_token),
			password_challenge: None,
		})
		.await?;

		Ok(())
	}

	// --- I/O Helpers ---

	async fn send_packet(&mut self, packet: impl Into<Packet>) -> Result<()> {
		let packet: Packet = packet.into();
		let mut payload = BytesMut::new();
		packet.encode(&mut payload);

		let payload = if packet.is_compressed() && !payload.is_empty() {
			// Pre-allocate at least the packet's size so there are fewer allocations
			let mut writer = BytesMut::with_capacity(payload.len()).writer();
			zstd::stream::copy_encode(payload.reader(), &mut writer, 3)?;
			writer.into_inner().freeze()
		} else {
			payload.freeze()
		};

		let mut header = BytesMut::new();
		header.put_i32_le(payload.len() as i32);
		header.put_i32_le(packet.id());

		self.send.write_all(&header).await?;
		self.send.write_all(&payload).await?;
		Ok(())
	}

	async fn kick(&mut self, reason: &str) -> Result<()> {
		let _ = self
			.send_packet(Disconnect {
				reason: Some(reason.to_string()),
				disconnect_type: DisconnectType::Disconnect,
			})
			.await;
		self.conn.close(0u32.into(), b"Kicked");
		Ok(())
	}

	async fn read_packet(&mut self) -> Result<Packet> {
		let len = self.recv.read_i32_le().await? as usize;
		let id = self.recv.read_i32_le().await?;

		if len > 1677721600 {
			bail!("Invalid Packet Length: {}", len);
		}

		let mut buf = BytesMut::zeroed(len); // This can't be just `with_capacity` because its length would be 0, which is what's used in read_exact. That means 0 bytes will be read.
		self.recv.read_exact(&mut buf).await?;

		let is_compressed = v2::is_id_compressed(id);

		let mut final_data = if is_compressed && !buf.is_empty() {
			let mut writer = BytesMut::with_capacity(buf.len() + 1024).writer();
			zstd::stream::copy_decode(buf.reader(), &mut writer)?;
			writer.into_inner().freeze()
		} else {
			buf.freeze()
		};

		let packet = Packet::decode(id, &mut final_data)?;

		Ok(packet)
	}
}
