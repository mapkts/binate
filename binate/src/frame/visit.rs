use super::*;
use bytes::{Buf, Bytes};

pub(super) type Result<T> = std::result::Result<T, DecodeError>;

macro_rules! incomplete_if_less_than {
    ($buf:ident, $len:expr) => {
        if $buf.remaining() < $len {
            return Err(DecodeError::InComplete);
        }
    };
}

pub(super) fn eat_stream_id<B: Buf>(buf: &mut B) -> Result<u32> {
    incomplete_if_less_than!(buf, 4);

    let stream_id = buf.get_u32();
    Ok(stream_id & MAX_U31)
}

pub(super) fn eat_flags<B: Buf>(buf: &mut B) -> Result<(FrameType, Flags)> {
    incomplete_if_less_than!(buf, 2);

    let flags = buf.get_u16();
    let ft = flags >> 10;
    let frame_type = match FrameType::from_bits(flags) {
        Some(frame_type) => frame_type,
        None => return Err(DecodeError::UnrecognizedFrameType(ft)),
    };
    let flags = Flags::from_bits_truncate(flags);

    Ok((frame_type, flags))
}

pub(super) fn eat_version<B: Buf>(buf: &mut B) -> Result<Version> {
    incomplete_if_less_than!(buf, 4);

    let major = buf.get_u16();
    let minor = buf.get_u16();
    Ok(Version::new(major, minor))
}

pub(super) fn eat_u8<B: Buf>(buf: &mut B) -> Result<u8> {
    incomplete_if_less_than!(buf, 2);

    Ok(buf.get_u8())
}

pub(super) fn eat_u16<B: Buf>(buf: &mut B) -> Result<u16> {
    incomplete_if_less_than!(buf, 2);

    Ok(buf.get_u16())
}

pub(super) fn eat_u24<B: Buf>(buf: &mut B) -> Result<U24> {
    let high = eat_u8(buf)?;
    let low = eat_u16(buf)?;
    Ok(U24::new(high, low))
}

pub(super) fn eat_u31<B: Buf>(buf: &mut B) -> Result<u32> {
    incomplete_if_less_than!(buf, 4);

    Ok(buf.get_u32() & MAX_U31)
}

pub(super) fn eat_u32<B: Buf>(buf: &mut B) -> Result<u32> {
    incomplete_if_less_than!(buf, 4);

    Ok(buf.get_u32())
}

pub(super) fn eat_u63<B: Buf>(buf: &mut B) -> Result<u64> {
    incomplete_if_less_than!(buf, 8);

    Ok(buf.get_u64() & MAX_U63)
}

pub(super) fn eat_bytes<B: Buf>(buf: &mut B, len: usize) -> Result<Bytes> {
    incomplete_if_less_than!(buf, len);

    Ok(buf.copy_to_bytes(len))
}

pub(super) fn eat_payload<B: Buf>(
    buf: &mut B,
    can_have_data: bool,
) -> Result<Payload> {
    let metadata_len =
        if can_have_data { eat_u24(buf)?.into_usize() } else { 0 };
    let metadata = if metadata_len > 0 {
        Some(eat_bytes(buf, metadata_len)?)
    } else {
        None
    };
    let data = match buf.remaining() {
        0 => None,
        len => Some(eat_bytes(buf, len)?),
    };
    Ok(Payload::new(metadata, data))
}

pub(super) fn eat_resume_token<B: Buf>(
    buf: &mut B,
    flags: Flags,
) -> Result<Option<Bytes>> {
    let resume_token = if flags.contains(Flags::RESUME) {
        let token_len = eat_u16(buf)?;
        Some(eat_bytes(buf, token_len as usize)?)
    } else {
        None
    };
    Ok(resume_token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_eat_flags() {
        let mut invalid_flags = BytesMut::new();
        invalid_flags.put_u16(
            (0x2F << 10)
                | Flags::METADATA.bits()
                | Flags::IGNORE.bits()
                | Flags::RESUME.bits(),
        );

        let mut valid_flags = BytesMut::new();
        valid_flags.put_u16(
            FrameType::SETUP.bits()
                | Flags::METADATA.bits()
                | Flags::IGNORE.bits()
                | Flags::RESUME.bits()
                | 0b01
                | 0b10,
        );

        assert_eq!(
            eat_flags(&mut invalid_flags),
            Err(DecodeError::UnrecognizedFrameType(0x2F))
        );
        assert_eq!(
            eat_flags(&mut valid_flags),
            Ok((
                FrameType::SETUP,
                Flags::METADATA | Flags::IGNORE | Flags::RESUME
            ))
        );
    }
}
