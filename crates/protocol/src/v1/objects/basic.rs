use macros::define_packet;

define_packet! { InstantData { seconds: i64, nanos: i32 } }

define_packet! { Vector2i { x: i32, y: i32 } }

define_packet! { Vector2f { x: f32, y: f32 } }

define_packet! { Vector3i { x: i32, y: i32, z: i32 } }

define_packet! { Vector3f { x: f32, y: f32, z: f32 } }

define_packet! { DirectionF { yaw: f32, pitch: f32, roll: f32 } }

define_packet! { PositionF { x: f64, y: f64, z: f64 } }

// This is the same as RangeF. Don't ask why. I don't have the answers.
define_packet! { FloatRange { inclusive_min: f32, inclusive_max: f32 } }

define_packet! {
	RangeVector2f {
		fixed {
			opt(1) x: RangeF,
			opt(2) y: RangeF,
		}
	}
}

define_packet! {
	RangeVector3f {
		fixed {
			opt(1) x: RangeF,
			opt(2) y: RangeF,
			opt(4) z: RangeF,
		}
	}
}

define_packet! { RangeF { min: f32, max: f32 } }

define_packet! { RangeI { min: i32, max: i32 } }

define_packet! { RangeB { min: u8, max: u8 } }

define_packet! { Color { red: u8, green: u8, blue: u8 } }

define_packet! { ColorAlpha { alpha: u8, red: u8, green: u8, blue: u8 } }

// Basically the same as ColorAlpha, but with a different first field name.
define_packet! { ColorLight { radius: u8, red: u8, green: u8, blue: u8 } }
