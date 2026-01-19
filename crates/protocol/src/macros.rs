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
            #[allow(clippy::wrong_self_convention)]
            pub fn from_u8(v: u8) -> Option<Self> {
                match v {
                    $($val => Some(Self::$variant),)*
                    _ => None,
                }
            }
            #[allow(clippy::wrong_self_convention)]
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
                #[allow(unused_imports)]
                use $crate::codec::PacketContext;

                if !buf.has_remaining() {
                    return Err(PacketError::Incomplete).context(stringify!($name));
                }
                let val = buf.get_u8();
                Self::from_u8(val).ok_or_else(|| PacketError::InvalidEnumVariant(val)).context(stringify!($name))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

#[macro_export]
macro_rules! define_packet {
    // Simple sequential packet layout that does not use bitmasks or offsets. All fields are required to exist.
    // Usage:
    // PacketName {
    //   a: i32,
    //   b: Vec3,
    //   c: String,
    // }
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
    // Unified masked packet
    // Usage:
    // PacketName {
    //   mask_size: 4, // The number of bytes used for the null bitmasks, optional & defaults to 1
    //   fixed { // For fixed-length fields
    //     required a: i32,
    //     opt b: Vector3f [pad=12] // Padding to preserve the space if the field is absent
    //     opt(2) c: Vector3f // The `opt(2)` syntax is for specifying which bit of the bitmask this field is checked against.
    //     // When the number is omitted, all omitted fields use a shared counter that starts from 0 and is increased by 1 per field
    //     // Explicitly-defined null bit does not increment this shared counter.
    //   }
    //   variable { // For variable-length fields
    //     opt(0) s: String
    //   }
    // }
    (
        $name:ident {
            $( mask_size: $mask_bytes:literal $(,)? )?
            fixed { $($fix_content:tt)* }
            variable { $($var_content:tt)* }
        }
    ) => {
        $crate::define_packet!(@impl_masked $name, ($($mask_bytes)?), (fixed { $($fix_content)* }), (variable { $($var_content)* }));
    };
    // Fixed-only masked packet
    (
        $name:ident {
            $( mask_size: $mask_bytes:literal $(,)? )?
            fixed { $($fix_content:tt)* }
        }
    ) => {
        $crate::define_packet!(@impl_masked $name, ($($mask_bytes)?), (fixed { $($fix_content)* }), (variable {}));
    };
    // Variable-only masked packet
    (
        $name:ident {
            $( mask_size: $mask_bytes:literal $(,)? )?
            variable { $($var_content:tt)* }
        }
    ) => {
        $crate::define_packet!(@impl_masked $name, ($($mask_bytes)?), (fixed {}), (variable { $($var_content)* }));
    };

    // The body of the 2nd pattern, abstracted away to reduce duplication.
    (@impl_masked
        $name:ident,
        ($($mask_bytes:literal)?),
        (fixed { $($fix_mode:ident $( ( $fix_bit:literal ) )? $fix_field:ident : $fix_type:ty $( [ $pad_lbl:ident = $pad:literal ] )? ),* $(,)? }),
        (variable { $($var_mode:ident $( ( $var_bit:literal ) )? $var_field:ident : $var_type:ty),* $(,)? })
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            // Fixed fields
            $( pub $fix_field: $crate::define_packet!(@field_type $fix_mode $fix_type), )*
            // Variable fields
            $( pub $var_field: $crate::define_packet!(@field_type $var_mode $var_type), )*
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) {
                #[allow(unused_imports)]
                use bytes::BufMut;

                // Calculate mask size. Default 1 if not provided.
                const MASK_SIZE: usize = 0 $( + $mask_bytes )? + (1 * (0 $( + $mask_bytes )? == 0) as usize);
                let mut null_bits = [0u8; MASK_SIZE];
                let mut _shift = 0;

                // Encode masks for fixed fields
                $(
                    $crate::define_packet!(@encode_mask_multi self $fix_field $fix_mode _shift null_bits $( $fix_bit )?);
                )*
                // Encode masks for variable fields
                $(
                    $crate::define_packet!(@encode_mask_multi self $var_field $var_mode _shift null_bits $( $var_bit )?);
                )*

                buf.put_slice(&null_bits);

                // Write fixed fields
                $(
                    $crate::define_packet!(@encode_field self $fix_field $fix_mode buf $( $pad )?);
                )*

                // Write offsets placeholders
                let mut _offset_indices: Vec<usize> = vec![];
                $(
                    _offset_indices.push(buf.len());
                    buf.put_i32_le(0);
                    let _ = stringify!($var_field);
                )*

                // Write variable bodies
                let var_block_start = buf.len();
                let mut _var_idx = 0;
                $(
                    $crate::define_packet!(@offset_encode_body self $var_field $var_mode _offset_indices _var_idx buf var_block_start);
                    _var_idx += 1;
                )*
            }

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                #[allow(unused_imports)]
                use bytes::Buf;
                #[allow(unused_imports)]
                use $crate::codec::PacketContext;

                if !buf.has_remaining() { return Err($crate::codec::PacketError::Incomplete).context("the whole buf"); }

                const MASK_SIZE: usize = 0 $( + $mask_bytes )? + (1 * (0 $( + $mask_bytes )? == 0) as usize);
                let remaining = buf.remaining();
                if remaining < MASK_SIZE { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: MASK_SIZE }).context("mask bytes"); }

                let mut null_bits = [0u8; MASK_SIZE];
                buf.copy_to_slice(&mut null_bits);

                let mut _shift = 0;

                // Decode fixed fields
                $(
                    let $fix_field = $crate::define_packet!(@decode_field_multi buf null_bits _shift $fix_field $fix_mode $fix_type, $( ( $fix_bit ) )?  $( [$pad] )?);
                )*

                // Read offsets
                let mut _offsets: Vec<i32> = vec![];
                $(
                    _offsets.push(buf.get_i32_le());
                    let _ = stringify!($var_field);
                )*

                let mut _var_read_bytes = 0i32;
                let mut _var_offset_idx = 0;
                $(
                    let $var_field = $crate::define_packet!(@stream_decode_body buf null_bits _shift _offsets _var_offset_idx _var_read_bytes $var_mode $var_type, $var_field $( ( $var_bit ) )? );
                    $crate::define_packet!(@inc_bit_idx $var_mode _shift $( $var_bit )?);
                    _var_offset_idx += 1;
                )*

                Ok(Self { $( $fix_field, )* $( $var_field, )* })
            }
        }
    };

    (@encode_mask_multi $self:ident $field:ident required $shift:ident $bits:ident) => {};
    (@encode_mask_multi $self:ident $field:ident opt $shift:ident $bits:ident) => {
        if $self.$field.is_some() {
            $bits[$shift / 8] |= (1 << ($shift % 8));
        }
        $shift += 1;
    };
    (@encode_mask_multi $self:ident $field:ident opt $shift:ident $bits:ident $expl:literal) => {
        if $self.$field.is_some() {
            $bits[$expl / 8] |= (1 << ($expl % 8));
        }
    };

    // Offset Encode Mask
    (@offset_encode_mask_multi $self:ident $field:ident required $shift:ident $bits:ident) => {};
    (@offset_encode_mask_multi $self:ident $field:ident opt $shift:ident $bits:ident) => {
        if $self.$field.is_some() {
             $bits[$shift / 8] |= (1 << ($shift % 8));
        }
        $shift += 1;
    };
    (@offset_encode_mask_multi $self:ident $field:ident opt $shift:ident $bits:ident $expl:literal) => {
         if $self.$field.is_some() {
             $bits[$expl / 8] |= (1 << ($expl % 8));
         }
    };

    // Decode Field Multi
    (@decode_field_multi $buf:ident $bits:ident $shift:ident $field:ident required $t:ty, $( ( $b:literal ) )? $( [ $p:literal ] )?) => {
        <$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?
    };

    (@decode_field_multi $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty, [ $pad:literal ]) => {
    {
        let is_set = ($bits[$shift / 8] & (1 << ($shift % 8))) != 0;
        let start = $buf.remaining();
        let val = if is_set {
            Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
        } else {
            None
        };

        let consumed = start - $buf.remaining();

        // TODO: Maybe consider a different error here later
        if consumed > $pad { return Err($crate::codec::PacketError::DecodedMoreThanPadding { actual: consumed, pad: $pad }).context(stringify!($field)); }
        let padding = $pad - consumed;
        let remaining = $buf.remaining();
        if remaining < padding { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: padding }).context(stringify!($field)); }
        $buf.advance(padding);

        $shift += 1;
        val
    }
};

    (@decode_field_multi $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty,) => {
        {
            let is_set = ($bits[$shift / 8] & (1 << ($shift % 8))) != 0;
            let val = if is_set {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
            } else {
                None
            };
            $shift += 1;
            val
        }
    };

    (@decode_field_multi $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty, ( $expl:literal ) [ $pad:literal ]) => {
        {
            let is_set = ($bits[$expl / 8] & (1 << ($expl % 8))) != 0;
            if is_set {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
            } else {
                let remaining = $buf.remaining();
                if remaining < $pad { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: $pad }).context(stringify!($field)); }
                $buf.advance($pad);
                None
            }
        }
    };

    (@decode_field_multi $buf:ident $bits:ident $shift:ident $field:ident opt $t:ty, ( $expl:literal )) => {
        {
            let is_set = ($bits[$expl / 8] & (1 << ($expl % 8))) != 0;
            if is_set {
                Some(<$t as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?)
            } else {
                None
            }
        }
    };

    (@stream_decode_body $buf:ident $bits:ident $bit_idx:ident $offsets:ident $off_idx:ident $read_bytes:ident opt $type:ty, $field:ident ( $expl:literal )) => {
        if ($bits[$expl / 8] & (1 << ($expl % 8))) != 0 {
             let target_offset = $offsets[$off_idx];
             if target_offset < $read_bytes { return Err($crate::codec::PacketError::IncompleteBytes { found: $read_bytes as usize, expected: target_offset as usize }).context(stringify!($field)); }

             let skip = (target_offset - $read_bytes) as usize;
             let remaining = $buf.remaining();
             if remaining < skip { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: skip } ).context(stringify!($field)); }
             $buf.advance(skip);
             $read_bytes += skip as i32;

             let start = $buf.remaining();
             let val = Some(<$type as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?);
             $read_bytes += (start - $buf.remaining()) as i32;
             val
        } else {
             None
        }
    };

    (@stream_decode_body $buf:ident $bits:ident $bit_idx:ident $offsets:ident $off_idx:ident $read_bytes:ident opt $type:ty, $field:ident) => {
        if ($bits[$bit_idx / 8] & (1 << ($bit_idx % 8))) != 0 {
             let target_offset = $offsets[$off_idx];
             if target_offset < $read_bytes { return Err($crate::codec::PacketError::IncompleteBytes { found: $read_bytes as usize, expected: target_offset as usize }).context(stringify!($field)); }

             let skip = (target_offset - $read_bytes) as usize;
             let remaining = $buf.remaining();
             if remaining < skip { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: skip }).context(stringify!($field)); }
             $buf.advance(skip);
             $read_bytes += skip as i32;

             let start = $buf.remaining();
             let val = Some(<$type as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?);
             $read_bytes += (start - $buf.remaining()) as i32;
             val
        } else {
             None
        }
    };

    (@stream_decode_body $buf:ident $bits:ident $bit_idx:ident $offsets:ident $off_idx:ident $read_bytes:ident required $type:ty, $field:ident ( $ign:literal )) => {
         {
             let target_offset = $offsets[$off_idx];
             if target_offset < $read_bytes { return Err($crate::codec::PacketError::IncompleteBytes { found: $read_bytes as usize, expected: target_offset as usize }).context(stringify!($field)); }

             let skip = (target_offset - $read_bytes) as usize;
             let remaining = $buf.remaining();
             if remaining < skip { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: skip }).context(stringify!($field)); }
             $buf.advance(skip);
             $read_bytes += skip as i32;

             let start = $buf.remaining();
             let val = <$type as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?;
             $read_bytes += (start - $buf.remaining()) as i32;
             val
        }
    };

    (@stream_decode_body $buf:ident $bits:ident $bit_idx:ident $offsets:ident $off_idx:ident $read_bytes:ident required $type:ty, $field:ident) => {
         {
             let target_offset = $offsets[$off_idx];
             if target_offset < $read_bytes { return Err($crate::codec::PacketError::IncompleteBytes { found: $read_bytes as usize, expected: target_offset as usize }).context(stringify!($field)); }

             let skip = (target_offset - $read_bytes) as usize;
             let remaining = $buf.remaining();
             if remaining < skip { return Err($crate::codec::PacketError::IncompleteBytes { found: remaining, expected: skip }).context(stringify!($field)); }
             $buf.advance(skip);
             $read_bytes += skip as i32;

             let start = $buf.remaining();
             let val = <$type as $crate::codec::HytaleCodec>::decode($buf).context(stringify!($field))?;
             $read_bytes += (start - $buf.remaining()) as i32;
             val
        }
    };

    (@encode_field $self:ident $field:ident required $buf:ident) => {
        $crate::codec::HytaleCodec::encode(&$self.$field, $buf);
    };
    (@encode_field $self:ident $field:ident opt $buf:ident $pad:literal) => {
        if let Some(v) = &$self.$field {
            $crate::codec::HytaleCodec::encode(v, $buf);
        } else {
            use bytes::BufMut;
            $buf.put_bytes(0, $pad);
        }
    };
    (@encode_field $self:ident $field:ident opt $buf:ident) => {
        if let Some(v) = &$self.$field {
            $crate::codec::HytaleCodec::encode(v, $buf);
        }
    };

    (@field_type required $t:ty) => { $t };
    (@field_type opt $t:ty) => { Option<$t> };

    (@inc_bit_idx required $idx:ident) => {};
    (@inc_bit_idx opt $idx:ident) => { $idx += 1; };
    (@inc_bit_idx $mode:ident $idx:ident $expl:literal) => {}; // Not incremented for explicit null bit

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
}
