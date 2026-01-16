#![allow(clippy::vec_init_then_push)]

#[macro_export]
macro_rules! define_packet {
    // PATTERN 1: Simple Sequential Packet (e.g., Connect)
    (
        $name:ident (id = $id:expr) {
            $($field:ident : $type:ty),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $type),*
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) {
                $( <$type as $crate::codec::HytaleCodec>::encode(&self.$field, buf); )*
            }

            fn decode(buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, $crate::codec::PacketError> {
                Ok(Self {
                    $( $field: <$type as $crate::codec::HytaleCodec>::decode(buf)? ),*
                })
            }
        }
    };

    // PATTERN 2: Hytale Offset Packet (AuthGrant, Status, etc.)
    // This handles the NullBitmask, Fixed Block, and Variable Block logic automatically.
    (
        $name:ident (id = $id:expr) {
            fixed {
                $($fix_field:ident : $fix_type:ty),* $(,)?
            }
            variable {
                $($var_field:ident : Option<$var_type:ty>),* $(,)?
            }
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $fix_field: $fix_type,)*
            $(pub $var_field: Option<$var_type>,)*
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) {
                use bytes::BufMut;

                // Hytale uses 1 byte for null bits if < 8 nullable fields
                let mut null_bits: u8 = 0;
                let mut _bit_idx = 0;

                $(
                    if self.$var_field.is_some() {
                        null_bits |= (1 << _bit_idx);
                    }
                    _bit_idx += 1;
                )*
                buf.put_u8(null_bits);

                $( <$fix_type as $crate::codec::HytaleCodec>::encode(&self.$fix_field, buf); )*

                // Reserve space for Offsets (4 bytes each for every variable field)
                let mut _offset_indices: Vec<usize> = vec![];
                $(
                    _offset_indices.push(buf.len());
                    buf.put_i32_le(0); // Placeholder offset
                    let _ = stringify!($var_field); // To make it work
                )*

                let var_block_start = buf.len();
                let mut _var_idx = 0;

                $(
                    if let Some(val) = &self.$var_field {
                        let offset = (buf.len() - var_block_start) as i32;

                        let place_idx = _offset_indices[_var_idx];
                        let mut slice = &mut buf[place_idx..place_idx+4];
                        slice.put_i32_le(offset);

                        <$var_type as $crate::codec::HytaleCodec>::encode(val, buf);
                    } else {
                        // Write -1 for null fields
                        let place_idx = _offset_indices[_var_idx];
                        let mut slice = &mut buf[place_idx..place_idx+4];
                        slice.put_i32_le(-1);
                    }
                    _var_idx += 1;
                )*
            }

            fn decode(buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, $crate::codec::PacketError> {
                use bytes::Buf;

                let start_pos = buf.position() as usize;

                if !buf.has_remaining() { return Err($crate::codec::PacketError::Incomplete); }
                let null_bits = buf.get_u8();

                $(
                    let $fix_field = <$fix_type as $crate::codec::HytaleCodec>::decode(buf)?;
                )*

                let mut _offsets: Vec<i32> = vec![];
                $(
                    _offsets.push(buf.get_i32_le());
                    // This ensures we loop once per var field
                    let _ = stringify!($var_field);
                )*

                // The 'variable block' starts theoretically at (1 + fixed_size + num_vars*4)
                // But we can just jump using the offsets.
                // 9 is: 1 (nullbyte) + 8 (2 * 4byte offsets).

                let var_block_start = buf.position();

                let mut _var_idx = 0;
                $(
                    let $var_field = if (null_bits & (1 << _var_idx)) != 0 {
                        let rel_offset = _offsets[_var_idx];
                        if rel_offset < 0 { return Err($crate::codec::PacketError::Incomplete); }

                        buf.set_position(var_block_start + rel_offset as u64);
                        Some(<$var_type as $crate::codec::HytaleCodec>::decode(buf)?)
                    } else {
                        None
                    };
                    _var_idx += 1;
                )*

                Ok(Self {
                    $( $fix_field, )*
                    $( $var_field, )*
                })
            }
        }
    };
}