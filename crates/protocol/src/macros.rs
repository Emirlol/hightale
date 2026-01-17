#[macro_export]
macro_rules! define_packet {
    // PATTERN 1: Offset Table (AuthGrant, ServerInfo)
    (
        $name:ident {
            fixed { $($fix_field:ident : $fix_type:ty),* $(,)? }
            variable { $($mode:ident $var_field:ident : $var_type:ty),* $(,)? }
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $fix_field: $fix_type,)*
            $(pub $var_field: $crate::define_packet!(@field_type $mode $var_type),)*
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) {
                #[allow(unused_imports)]
                use bytes::BufMut;

                let mut null_bits: u8 = 0;
                let mut _bit_idx = 0;

                $(
                    $crate::define_packet!(@offset_encode_mask self $var_field $mode _bit_idx null_bits);
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
                    $crate::define_packet!(@offset_encode_body self $var_field $mode _offset_indices _var_idx buf var_block_start);
                    _var_idx += 1;
                )*
            }

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                #[allow(unused_imports)]
                use bytes::Buf;
                #[allow(unused_imports)]
                use std::io::Cursor;
                #[allow(unused_imports)]
                use $crate::codec::PacketContext;

                let mut buf = Cursor::new(buf.copy_to_bytes(buf.remaining()));
                let start_pos = buf.position() as usize;

                if !buf.has_remaining() { return Err($crate::codec::PacketError::Incomplete); }
                let null_bits = buf.get_u8();

                $( let $fix_field = <$fix_type as $crate::codec::HytaleCodec>::decode(&mut buf).context(stringify!($fix_field))?; )*

                let mut _offsets: Vec<i32> = vec![];
                $(
                    _offsets.push(buf.get_i32_le());
                    let _ = stringify!($var_field);
                )*

                let var_block_start = buf.position();
                let mut _var_bit_idx = 0;
                let mut _var_offset_idx = 0;

                 $(
                    let $var_field = $crate::define_packet!(@offset_decode_body buf null_bits _var_bit_idx _offsets _var_offset_idx var_block_start $mode $var_type, $var_field);

                    $crate::define_packet!(@inc_bit_idx $mode _var_bit_idx);
                    _var_offset_idx += 1;
                )*

                Ok(Self { $( $fix_field, )* $( $var_field, )* })
            }
        }
    };

    // PATTERN 2: Sequential with Bitmask
    // Usage: bitmask {
    //     required a: i32,
    //     opt b: String,
    //     opt(2) c: String [pad=16]
    // }
    // The number inside of opt() is the bit position of the field. 0 means the first bit, 1 means the second bit, etc.
    // If no bit position is specified, the bits are assigned sequentially in the order they are defined.
    // Explicitly defined bit positions do not change the counter, so if you have opt(0) and then opt without a number, the second opt will still be assigned bit 1.
    // Padding is also optional, if omitted `None` types will not write any data. If padding is specified, that many zero bytes will be written when the field is `None`.
    (
        $name:ident {
            bitmask {
                $($mode:ident $( ( $bit:literal ) )? $field:ident : $type:ty $( [ $pad_lbl:ident = $pad:literal ] )? ),* $(,)?
            }
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $( pub $field: $crate::define_packet!(@field_type $mode $type), )*
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) {
                #[allow(unused_imports)]
                use bytes::BufMut;

                let mut null_bits: u8 = 0;
                let mut _shift = 0;

                $(
                    $crate::define_packet!(@encode_mask self $field $mode _shift null_bits $( $bit )?);
                )*

                buf.put_u8(null_bits);

                $(
                    $crate::define_packet!(@encode_field self $field $mode buf $( $pad )?);
                )*
            }

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                #[allow(unused_imports)]
                use bytes::Buf;
                #[allow(unused_imports)]
                use $crate::codec::PacketContext;

                if !buf.has_remaining() { return Err($crate::codec::PacketError::Incomplete); }
                let null_bits = buf.get_u8();
                let mut _shift = 0;

                Ok(Self {
                    $(
                        $field: $crate::define_packet!(@decode_field buf null_bits _shift $field $mode $type, $( ( $bit ) )?  $( [$pad] )?),
                    )*
                })
            }
        }
    };

    // PATTERN 3: Simple Sequential
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
                #[allow(unused_imports)]
                use $crate::codec::PacketContext;
                Ok(Self {
                    $( $field: <$type as $crate::codec::HytaleCodec>::decode(buf).context(stringify!($field))? ),*
                })
            }
        }
    };

    // --- Helper: Determine Field Type ---
    (@field_type required $t:ty) => { $t };
    (@field_type opt $t:ty) => { Option<$t> };

    (@inc_bit_idx required $idx:ident) => {};
    (@inc_bit_idx opt $idx:ident) => { $idx += 1; };

    // --- Pattern 1 Helpers: Encode Mask ---
    (@offset_encode_mask $self:ident $field:ident required $bit:ident $mask:ident) => {}; // Required: has no bit
    (@offset_encode_mask $self:ident $field:ident opt $bit:ident $mask:ident) => {
        if $self.$field.is_some() { $mask |= (1 << $bit); }
        $bit += 1;
    };

    // --- Pattern 1 Helpers: Encode Body ---
    (@offset_encode_body $self:ident $field:ident required $offsets:ident $idx:ident $buf:ident $start:ident) => {
        {
            let current_pos = $buf.len();
            let offset_val = (current_pos - $start) as i32;
            let patch_pos = $offsets[$idx];
            // Patch the placeholder offset
            $buf[patch_pos..patch_pos+4].copy_from_slice(&offset_val.to_le_bytes());

            $crate::codec::HytaleCodec::encode(&$self.$field, $buf);
        }
    };

    (@offset_encode_body $self:ident $field:ident opt $offsets:ident $idx:ident $buf:ident $start:ident) => {
        {
            let current_pos = $buf.len();
            let offset_val = (current_pos - $start) as i32;
            let patch_pos = $offsets[$idx];
            $buf[patch_pos..patch_pos+4].copy_from_slice(&offset_val.to_le_bytes());

            if let Some(v) = &$self.$field {
                $crate::codec::HytaleCodec::encode(v, $buf);
            }
        }
    };

    // --- Pattern 1 Helpers: Decode Body ---
    (@offset_decode_body $buf:ident $bits:ident $bit_idx:ident $offsets:ident $off_idx:ident $start:ident required $type:ty, $field:ident) => {
        {
             let rel_offset = $offsets[$off_idx];
             if rel_offset < 0 { return Err($crate::codec::PacketError::Incomplete); }
             $buf.set_position($start + rel_offset as u64);
             <$type as $crate::codec::HytaleCodec>::decode(&mut $buf).context(stringify!($field))?
        }
    };

    (@offset_decode_body $buf:ident $bits:ident $bit_idx:ident $offsets:ident $off_idx:ident $start:ident opt $type:ty, $field:ident) => {
        if ($bits & (1 << $bit_idx)) != 0 {
             let rel_offset = $offsets[$off_idx];
             if rel_offset < 0 { return Err($crate::codec::PacketError::Incomplete); }
             $buf.set_position($start + rel_offset as u64);
             Some(<$type as $crate::codec::HytaleCodec>::decode(&mut $buf).context(stringify!($field))?)
        } else {
             None
        }
    };

    // --- Pattern 2 Helpers: Encode Mask ---

    // Case 1: Required field (ignores bit args if present)
    (@encode_mask $self:ident $field:ident required $shift:ident $bits:ident) => {}; // Do nothing
    // Case 2: Optional field with explicit bit arg
    (@encode_mask $self:ident $field:ident opt $shift:ident $bits:ident) => {
        if $self.$field.is_some() { $bits |= (1 << $shift); }
        $shift += 1;
    };
    // Case 3: Optional field without explicit bit arg
    (@encode_mask $self:ident $field:ident opt $shift:ident $bits:ident $expl:literal) => {
        if $self.$field.is_some() { $bits |= (1 << $expl); }
    };

    // --- Pattern 2 Helpers: Encode Field ---

    // Case 1: Required field
    (@encode_field $self:ident $field:ident required $buf:ident) => {
        $crate::codec::HytaleCodec::encode(&$self.$field, $buf);
    };
    // Case 2: Optional with padding
    (@encode_field $self:ident $field:ident opt $buf:ident $pad:literal) => {
        if let Some(v) = &$self.$field {
            $crate::codec::HytaleCodec::encode(v, $buf);
        } else {
            use bytes::BufMut;
            $buf.put_bytes(0, $pad);
        }
    };
    // Case 3: Optional without padding
    (@encode_field $self:ident $field:ident opt $buf:ident) => {
        if let Some(v) = &$self.$field {
            $crate::codec::HytaleCodec::encode(v, $buf);
        }
    };

    // --- Pattern 2 Helpers: Decode Field ---

    // Case 1: Required (eats any trailing args like (0) or [16])
    (@decode_field $buf:ident $bits:ident $shift:ident $field:ident required $t:ty, $( ( $b:literal ) )? $( [ $p:literal ] )?) => {
        <$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?
    };

    // Case 2: Optional Implicit w/ Padding
    (@decode_field $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty, [ $pad:literal ]) => {
        {
            let val = if ($bits & (1 << $shift)) != 0 {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
            } else {
                if $buf.remaining() < $pad { return Err($crate::codec::PacketError::Incomplete); }
                $buf.advance($pad);
                None
            };
            $shift += 1;
            val
        }
    };

    // Case 3: Optional Implicit w/o Padding
    (@decode_field $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty,) => {
        {
            let val = if ($bits & (1 << $shift)) != 0 {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
            } else {
                None
            };
            $shift += 1;
            val
        }
    };

    // Case 4: Optional Explicit w/ Padding
    (@decode_field $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty, ( $expl:literal ) [ $pad:literal ]) => {
        {
            if ($bits & (1 << $expl)) != 0 {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
            } else {
                if $buf.remaining() < $pad { return Err($crate::codec::PacketError::Incomplete); }
                $buf.advance($pad);
                None
            }
        }
    };

    // Case 5: Optional Explicit w/o Padding
    (@decode_field $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty, ( $expl:literal )) => {
        {
            if ($bits & (1 << $expl)) != 0 {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf)?)
            } else {
                None
            }
        }
    };
}

/// Macro to define an enum and automatically implement Enumeration for it.
/// These are enums that are serialized/deserialized as their ordinals.
/// define_enum! {
///     pub enum MyState {
///         Handshake = 0,
///         Play = 1
///     }
/// }
#[macro_export]
macro_rules! define_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $val:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        $vis enum $name {
            $($variant = $val),+
        }

        impl $name {
            pub fn from_u8(v: u8) -> Option<Self> {
                match v {
                    $($val => Some(Self::$variant),)*
                    _ => None,
                }
            }
            pub fn to_u8(&self) -> u8 {
                *self as u8
            }
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) {
                #[allow(unused_imports)]
                use bytes::BufMut;

                buf.put_u8(self.to_u8());
            }

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                #[allow(unused_imports)]
                use $crate::codec::PacketError;

                if !buf.has_remaining() {
                    return Err(PacketError::Incomplete);
                }
                let val = buf.get_u8();
                Self::from_u8(val).ok_or_else(|| PacketError::InvalidEnumVariant(val))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}
