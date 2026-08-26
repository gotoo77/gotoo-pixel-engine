use std::io::{Cursor, ErrorKind};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::AudioError;

pub(crate) struct DecodedAudio {
    pub(crate) channels: u16,
    pub(crate) sample_rate: u32,
    pub(crate) samples: Vec<f32>,
}

pub(crate) fn decode_audio(bytes: &[u8]) -> Result<DecodedAudio, AudioError> {
    if bytes.is_empty() {
        return Err(AudioError::new("audio asset is empty"));
    }

    let source = Cursor::new(bytes.to_vec());
    let stream = MediaSourceStream::new(Box::new(source), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|err| AudioError::new(format!("unsupported or invalid audio asset: {err}")))?;
    let mut format = probed.format;

    let (track_id, mut decoder) = {
        let track = format
            .default_track()
            .ok_or_else(|| AudioError::new("audio asset contains no default audio track"))?;
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|err| AudioError::new(format!("unsupported audio codec: {err}")))?;
        (track.id, decoder)
    };

    let mut samples = Vec::new();
    let mut channels = None;
    let mut sample_rate = None;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(err) => {
                return Err(AudioError::new(format!(
                    "failed to read audio packet: {err}"
                )));
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(err) => {
                return Err(AudioError::new(format!(
                    "failed to decode audio packet: {err}"
                )));
            }
        };

        let spec = *decoded.spec();
        let packet_channels = spec.channels.count();
        if packet_channels == 0 || packet_channels > usize::from(u16::MAX) {
            return Err(AudioError::new("audio asset has an unsupported channel count"));
        }
        if spec.rate == 0 {
            return Err(AudioError::new("audio asset has an invalid sample rate"));
        }

        match channels {
            Some(existing) if existing != packet_channels => {
                return Err(AudioError::new(
                    "audio channel layout changed during decoding",
                ));
            }
            None => channels = Some(packet_channels),
            _ => {}
        }
        match sample_rate {
            Some(existing) if existing != spec.rate => {
                return Err(AudioError::new("audio sample rate changed during decoding"));
            }
            None => sample_rate = Some(spec.rate),
            _ => {}
        }

        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buffer.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buffer.samples());
    }

    if samples.is_empty() {
        return Err(AudioError::new("audio asset contains no decodable samples"));
    }

    Ok(DecodedAudio {
        channels: channels.expect("decoded samples require a channel count") as u16,
        sample_rate: sample_rate.expect("decoded samples require a sample rate"),
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::decode_audio;

    const VALID_WAV: &[u8] = &[
        82, 73, 70, 70, 40, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0,
        68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 4, 0, 0, 0, 0, 0, 255, 127,
    ];

    #[test]
    fn decodes_pcm_wav_through_generic_decoder() {
        let sound = decode_audio(VALID_WAV).expect("valid WAV should decode");
        assert_eq!(sound.channels, 1);
        assert_eq!(sound.sample_rate, 44_100);
        assert_eq!(sound.samples.len(), 2);
    }

    #[test]
    fn rejects_empty_asset() {
        assert!(decode_audio(&[]).is_err());
    }

    #[test]
    fn rejects_unknown_asset() {
        assert!(decode_audio(b"not audio").is_err());
    }
}
