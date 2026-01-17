#![allow(clippy::vec_init_then_push)]

#[macro_export]
macro_rules! define_packet {
    // PATTERN 1: Sequential
    (
        $name:ident {
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

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                Ok(Self {
                    $( $field: <$type as $crate::codec::HytaleCodec>::decode(buf)? ),*
                })
            }
        }
    };

    // PATTERN 2: Offset Table
    (
        $name:ident {
            fixed { $($fix_field:ident : $fix_type:ty),* $(,)? }
            variable { $($var_field:ident : Option<$var_type:ty>),* $(,)? }
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

                let mut null_bits: u8 = 0;
                let mut _bit_idx = 0;
                $(
                    if self.$var_field.is_some() { null_bits |= (1 << _bit_idx); }
                    _bit_idx += 1;
                )*
                buf.put_u8(null_bits);

                $( <$fix_type as $crate::codec::HytaleCodec>::encode(&self.$fix_field, buf); )*

                let mut _offset_indices: Vec<usize> = vec![];
                $(
                    _offset_indices.push(buf.len());
                    buf.put_i32_le(0);
                    let _ = stringify!($var_field);
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
                        let place_idx = _offset_indices[_var_idx];
                        let mut slice = &mut buf[place_idx..place_idx+4];
                        slice.put_i32_le(-1);
                    }
                    _var_idx += 1;
                )*
            }

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                use bytes::Buf;
                if !buf.has_remaining() { return Err($crate::codec::PacketError::Incomplete); }
                let null_bits = buf.get_u8();

                $( let $fix_field = <$fix_type as $crate::codec::HytaleCodec>::decode(buf)?; )*

                let mut _offsets: Vec<i32> = vec![];
                $(
                    _offsets.push(buf.get_i32_le());
                    let _ = stringify!($var_field);
                )*

                let mut buf = std::io::Cursor::new(buf.copy_to_bytes(buf.remaining()));

                let var_block_start = buf.position();
                let mut _var_idx = 0;
                $(
                    let $var_field = if (null_bits & (1 << _var_idx)) != 0 {
                        let rel_offset = _offsets[_var_idx];
                        if rel_offset < 0 { return Err($crate::codec::PacketError::Incomplete); }
                        buf.set_position(var_block_start + rel_offset as u64);
                        Some(<$var_type as $crate::codec::HytaleCodec>::decode(&mut buf)?)
                    } else {
                        None
                    };
                    _var_idx += 1;
                )*

                Ok(Self { $( $fix_field, )* $( $var_field, )* })
            }
        }
    };
}