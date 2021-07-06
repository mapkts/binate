//! A list of wellknown MIME types.

/// Default mimetype for encoding metadata and data.
pub const DEFAULT_MIMETYPE: &str = "application/binary";

/// Well-known MIME types.
#[rustfmt::skip]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WellKnownMimeType {
    UNPARSEABLE,
    APPLICATION_AVRO,
    APPLICATION_CBOR,
    APPLICATION_GRAPHQL,
    APPLICATION_GZIP,
    APPLICATION_JAVASCRIPT,
    APPLICATION_JSON,
    APPLICATION_OCTET_STREAM,
    APPLICATION_PDF,
    APPLICATION_VND_APACHE_THRIFT_BINARY,
    APPLICATION_VND_GOOGLE_PROTOBUF,
    APPLICATION_XML,
    APPLICATION_ZIP,
    AUDIO_AAC,
    AUDIO_MP3,
    AUDIO_MP4,
    AUDIO_MPEG3,
    AUDIO_MPEG,
    AUDIO_OGG,
    AUDIO_OPUS,
    AUDIO_VORBIS,
    IMAGE_BMP,
    IMAGE_GIF,
    IMAGE_HEIC_SEQUENCE,
    IMAGE_HEIC,
    IMAGE_HEIF_SEQUENCE,
    IMAGE_HEIF,
    IMAGE_JPEG,
    IMAGE_PNG,
    IMAGE_TIFF,
    MULTIPART_MIXED,
    TEXT_CSS,
    TEXT_CSV,
    TEXT_HTML,
    TEXT_PLAIN,
    TEXT_XML,
    VIDEO_H264,
    VIDEO_H265,
    VIDEO_VP8,
    APPLICATION_X_HESSIAN,
    APPLICATION_X_JAVA_OBJECT,
    APPLICATION_CLOUDEVENTS_JSON,
    MESSAGE_X_RSOCKET_MIME_TYPE_V0,
    MESSAGE_X_RSOCKET_ACCEPT_TIME_TYPES_V0,
    MESSAGE_X_RSOCKET_AUTHENTICATION_V0,
    MESSAGE_X_RSOCKET_TRACING_ZIPKIN_V0,
    MESSAGE_X_RSOCKET_ROUTING_V0,
    MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0,
}

#[rustfmt::skip]
impl From<&str> for WellKnownMimeType {
    fn from(v: &str) -> Self {
        use WellKnownMimeType::*;
        match v {
            "application/avro" => APPLICATION_AVRO,
            "application/cbor" => APPLICATION_CBOR,
            "application/graphql" => APPLICATION_GRAPHQL,
            "application/gzip" => APPLICATION_GZIP,
            "application/javascript" => APPLICATION_JAVASCRIPT,
            "application/json" => APPLICATION_JSON,
            "application/octet-stream" => APPLICATION_OCTET_STREAM,
            "application/pdf" => APPLICATION_PDF,
            "application/vnd.apache.thrift.binary" => APPLICATION_VND_APACHE_THRIFT_BINARY,
            "application/vnd.google.protobuf" => APPLICATION_VND_GOOGLE_PROTOBUF,
            "application/xml" => APPLICATION_XML,
            "application/zip" => APPLICATION_ZIP,
            "audio/aac" => AUDIO_AAC,
            "audio/mp3" => AUDIO_MP3,
            "audio/mp4" => AUDIO_MP4,
            "audio/mpeg3" => AUDIO_MPEG3,
            "audio/mpeg" => AUDIO_MPEG,
            "audio/ogg" => AUDIO_OGG,
            "audio/opus" => AUDIO_OPUS,
            "audio/vorbis" => AUDIO_VORBIS,
            "image/bmp" => IMAGE_BMP,
            "image/gif" => IMAGE_GIF,
            "image/heic-sequence" => IMAGE_HEIC_SEQUENCE,
            "image/heic" => IMAGE_HEIC,
            "image/heif-sequence" => IMAGE_HEIF_SEQUENCE,
            "image/heif" => IMAGE_HEIF,
            "image/jpeg" => IMAGE_JPEG,
            "image/png" => IMAGE_PNG,
            "image/tiff" => IMAGE_TIFF,
            "multipart/mixed" => MULTIPART_MIXED,
            "text/css" => TEXT_CSS,
            "text/csv" => TEXT_CSV,
            "text/html" => TEXT_HTML,
            "text/plain" => TEXT_PLAIN,
            "text/xml" => TEXT_XML,
            "video/H264" => VIDEO_H264,
            "video/H265" => VIDEO_H265,
            "video/VP8" => VIDEO_VP8,
            "application/x-hessian" => APPLICATION_X_HESSIAN,
            "application/x-java-object" => APPLICATION_X_JAVA_OBJECT,
            "application/cloudevents+json" => APPLICATION_CLOUDEVENTS_JSON,
            "message/x.rsocket.mime.type.v0" => MESSAGE_X_RSOCKET_MIME_TYPE_V0,
            "message/x.rsocket.accept.time.types.v0" => MESSAGE_X_RSOCKET_ACCEPT_TIME_TYPES_V0,
            "message/x.rsocket.authentication.v0" => MESSAGE_X_RSOCKET_AUTHENTICATION_V0,
            "message/x.rsocket.tracing.zipkin.v0" => MESSAGE_X_RSOCKET_TRACING_ZIPKIN_V0,
            "message/x.rsocket.routing.v0" => MESSAGE_X_RSOCKET_ROUTING_V0,
            "message/x.rsocket.composite.metadata.v0" => MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0,
            _ => UNPARSEABLE,
        }
    }
}

impl From<WellKnownMimeType> for &'static str {
    fn from(t: WellKnownMimeType) -> &'static str {
        if t == WellKnownMimeType::UNPARSEABLE {
            ""
        } else {
            t.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unparsable_mime_type() {
        let mime: WellKnownMimeType = "unparsable".into();
        let string: &'static str = mime.into();
        assert_eq!(mime, WellKnownMimeType::UNPARSEABLE);
        assert_eq!(string, "");
    }
}
