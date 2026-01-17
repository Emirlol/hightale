#![allow(unused_variables, unused_imports)]

use std::collections::HashMap;
use std::io::Cursor;
use crate::{define_enum, define_packet, packets::Asset};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::codec::{HytaleCodec, PacketContext, PacketError, PacketResult};

// Simple signal packet with no data
define_packet!(AssetFinalize {});

define_packet!(
	AssetInitialize {
		size: i32,
		asset: Asset
	}
);

define_packet!(
	AssetPart {
		bitmask {
			opt part: Bytes
		}
	}
);

define_enum! {
	pub enum ClientFeature {
		SplitVelocity = 0,
		Mantling = 1,
		SprintForce = 2,
		CrouchSlide = 3,
		SafetyRoll= 4,
		DisplayHealthBars = 5,
		DisplayCombatText = 6,
	}
}

// PITA struct with bitmask + offsets + variable data
#[derive(Debug, Clone, Default)]
pub struct PlayerSkin {
	pub body_characteristic: Option<String>, // Byte 0, Bit 1
	pub underwear: Option<String>,           // Byte 0, Bit 2
	pub face: Option<String>,                // Byte 0, Bit 4
	pub eyes: Option<String>,                // Byte 0, Bit 8
	pub ears: Option<String>,                // Byte 0, Bit 16
	pub mouth: Option<String>,               // Byte 0, Bit 32
	pub facial_hair: Option<String>,         // Byte 0, Bit 64
	pub haircut: Option<String>,             // Byte 0, Bit 128

	pub eyebrows: Option<String>,            // Byte 1, Bit 1
	pub pants: Option<String>,               // Byte 1, Bit 2
	pub overpants: Option<String>,           // Byte 1, Bit 4
	pub undertop: Option<String>,            // Byte 1, Bit 8
	pub overtop: Option<String>,             // Byte 1, Bit 16
	pub shoes: Option<String>,               // Byte 1, Bit 32
	pub head_accessory: Option<String>,      // Byte 1, Bit 64
	pub face_accessory: Option<String>,      // Byte 1, Bit 128

	pub ear_accessory: Option<String>,       // Byte 2, Bit 1
	pub skin_feature: Option<String>,        // Byte 2, Bit 2
	pub gloves: Option<String>,              // Byte 2, Bit 4
	pub cape: Option<String>,                // Byte 2, Bit 8
}

impl HytaleCodec for PlayerSkin {
	fn encode(&self, buf: &mut BytesMut) {
		let start_pos = buf.len();
		let mut mask = [0u8; 3];

		if self.body_characteristic.is_some() { mask[0] |= 1; }
		if self.underwear.is_some() { mask[0] |= 2; }
		if self.face.is_some() { mask[0] |= 4; }
		if self.eyes.is_some() { mask[0] |= 8; }
		if self.ears.is_some() { mask[0] |= 16; }
		if self.mouth.is_some() { mask[0] |= 32; }
		if self.facial_hair.is_some() { mask[0] |= 64; }
		if self.haircut.is_some() { mask[0] |= 128; }

		if self.eyebrows.is_some() { mask[1] |= 1; }
		if self.pants.is_some() { mask[1] |= 2; }
		if self.overpants.is_some() { mask[1] |= 4; }
		if self.undertop.is_some() { mask[1] |= 8; }
		if self.overtop.is_some() { mask[1] |= 16; }
		if self.shoes.is_some() { mask[1] |= 32; }
		if self.head_accessory.is_some() { mask[1] |= 64; }
		if self.face_accessory.is_some() { mask[1] |= 128; }

		if self.ear_accessory.is_some() { mask[2] |= 1; }
		if self.skin_feature.is_some() { mask[2] |= 2; }
		if self.gloves.is_some() { mask[2] |= 4; }
		if self.cape.is_some() { mask[2] |= 8; }

		buf.put_slice(&mask);

		let offsets_start = buf.len();
		buf.put_bytes(0, 20 * 4);

		let var_block_start = buf.len();

		// Helper closure to write field and update offset
		let mut write_field = |field: &Option<String>, idx: usize| {
			if let Some(s) = field {
				let current_offset = (buf.len() - var_block_start) as i32;
				let place_idx = offsets_start + (idx * 4);
				let mut slice = &mut buf[place_idx..place_idx+4];
				slice.put_i32_le(current_offset);
				<String as HytaleCodec>::encode(s, buf);
			}
		};

		write_field(&self.body_characteristic, 0);
		write_field(&self.underwear, 1);
		write_field(&self.face, 2);
		write_field(&self.eyes, 3);
		write_field(&self.ears, 4);
		write_field(&self.mouth, 5);
		write_field(&self.facial_hair, 6);
		write_field(&self.haircut, 7);
		write_field(&self.eyebrows, 8);
		write_field(&self.pants, 9);
		write_field(&self.overpants, 10);
		write_field(&self.undertop, 11);
		write_field(&self.overtop, 12);
		write_field(&self.shoes, 13);
		write_field(&self.head_accessory, 14);
		write_field(&self.face_accessory, 15);
		write_field(&self.ear_accessory, 16);
		write_field(&self.skin_feature, 17);
		write_field(&self.gloves, 18);
		write_field(&self.cape, 19);
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		// Fixed Header: 3 bytes mask + 80 bytes offsets = 83 bytes
		if buf.remaining() < 83 { return Err(PacketError::Incomplete); }

		let mut buf = Cursor::new(buf.copy_to_bytes(buf.remaining()));
		let start_pos = buf.position();
		let mut mask = [0u8; 3];
		buf.copy_to_slice(&mut mask);

		let mut offsets = [0i32; 20];
		for i in 0..20 {
			offsets[i] = buf.get_i32_le();
		}

		let var_block_start = buf.position(); // Should be start_pos + 83

		// Helper to read field
		let mut read_field = |byte_idx: usize, bit_val: u8, offset_idx: usize| -> PacketResult<Option<String>> {
			if (mask[byte_idx] & bit_val) != 0 {
				let rel_offset = offsets[offset_idx];
				if rel_offset < 0 { return Err(PacketError::Incomplete); }

				// Jump and read
				let target = var_block_start + rel_offset as u64;
				buf.set_position(target);
				Ok(Some(<String as HytaleCodec>::decode(&mut buf)?))
			} else {
				Ok(None)
			}
		};

		Ok(PlayerSkin {
			body_characteristic: read_field(0, 1, 0).context("body_characteristic")?,
			underwear: read_field(0, 2, 1).context("underwear")?,
			face: read_field(0, 4, 2).context("face")?,
			eyes: read_field(0, 8, 3).context("eyes")?,
			ears: read_field(0, 16, 4).context("ears")?,
			mouth: read_field(0, 32, 5).context("mouth")?,
			facial_hair: read_field(0, 64, 6).context("facial_hair")?,
			haircut: read_field(0, 128, 7).context("haircut")?,

			eyebrows: read_field(1, 1, 8).context("eyebrows")?,
			pants: read_field(1, 2, 9).context("pants")?,
			overpants: read_field(1, 4, 10).context("overpants")?,
			undertop: read_field(1, 8, 11).context("undertop")?,
			overtop: read_field(1, 16, 12).context("overtop")?,
			shoes: read_field(1, 32, 13).context("shoes")?,
			head_accessory: read_field(1, 64, 14).context("head_accessory")?,
			face_accessory: read_field(1, 128, 15).context("face_accessory")?,

			ear_accessory: read_field(2, 1, 16).context("ear_accessory")?,
			skin_feature: read_field(2, 2, 17).context("skin_feature")?,
			gloves: read_field(2, 4, 18).context("gloves")?,
			cape: read_field(2, 8, 19).context("cape")?,
		})
	}
}


define_packet!(
	PlayerOptions {
		bitmask {
			opt player_skin: Box<PlayerSkin>, // Box it since it's like 480 bytes and it bloats the largest packet size in the packet enum
		}
	}
);

define_packet!(
	RemoveAssets {
		bitmask {
			opt assets: Vec<Asset>,
		}
	}
);

define_packet!(
	RequestAssets {
		bitmask {
			opt assets: Vec<Asset>,
		}
	}
);

// Simple signal packet with no data
define_packet!(
	RequestCommonAssetsRebuild {}
);

define_packet!(
	ServerTags {
		bitmask {
			opt tags: HashMap<String, i32>,
		}
	}
);

define_packet!(
	SetTimeDilation {
		time_dilation: f32
	}
);

define_packet!(
	SetUpdateRate {
		updates_per_second: i32
	}
);

define_packet!(
	UpdateFeatures {
		bitmask {
			opt features: HashMap<ClientFeature, bool>,
		}
	}
);

define_packet!(
	ViewRadius {
		value: i32
	}
);

// Simple signal packet with no data
define_packet!(WorldLoadFinished {});

define_packet!(
	WorldLoadProgress {
		bitmask {
			required percent_complete: i32,
			required percent_complete_subitem: i32,
			opt message: String,
		}
	}
);

define_packet!(
	WorldSettings {
		bitmask {
			required world_height: i32,
			opt required_assets: Vec<Asset>
		}
	}
);
