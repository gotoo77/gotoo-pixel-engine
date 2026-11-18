use rodio::{ChannelCount, SampleRate, Source};
use std::{sync::Arc, time::Duration};

/// PCM already validated by the WAV decoder, split into real 1024-frame spans.
#[derive(Clone)]
pub(super) struct LoopBuffer {
    samples: Arc<[f32]>,
    channels: ChannelCount,
    sample_rate: SampleRate,
    position: usize,
}

impl LoopBuffer {
    pub(super) fn new(
        channels: ChannelCount,
        sample_rate: SampleRate,
        samples: Arc<[f32]>,
    ) -> Self {
        debug_assert_eq!(samples.len() % usize::from(channels.get()), 0);
        Self {
            samples,
            channels,
            sample_rate,
            position: 0,
        }
    }
}

impl Iterator for LoopBuffer {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let sample = self.samples.get(self.position).copied()?;
        self.position += 1;
        Some(sample)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.samples.len() - self.position;
        (remaining, Some(remaining))
    }
}

impl Source for LoopBuffer {
    fn current_span_len(&self) -> Option<usize> {
        if self.position == self.samples.len() {
            return Some(0);
        }
        let cap = 1024 * usize::from(self.channels.get());
        let start = self.position / cap * cap;
        // Rodio 0.22.2 specifies total span size, not remaining samples.
        Some(cap.min(self.samples.len() - start))
    }
    fn channels(&self) -> ChannelCount {
        self.channels
    }
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.samples.len() as f64
                / f64::from(self.channels.get())
                / f64::from(self.sample_rate.get()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::{ChannelCount, SampleRate, Source};
    use std::sync::Arc;
    #[test]
    fn real_frame_aligned_blocks_preserve_pcm_and_tail() {
        for channels in [1, 2] {
            for frames in [1, 1023, 1024, 1027, 44100] {
                let count = frames * usize::from(channels);
                let data: Arc<[f32]> = (0..count).map(|i| i as f32).collect::<Vec<_>>().into();
                let mut source = LoopBuffer::new(
                    ChannelCount::new(channels).unwrap(),
                    SampleRate::new(44100).unwrap(),
                    data,
                );
                let mut consumed = 0;
                while let Some(n) = source.current_span_len().filter(|&n| n > 0) {
                    assert!(n <= 1024 * usize::from(channels));
                    assert_eq!(n % usize::from(channels), 0);
                    assert_eq!(n, (1024 * usize::from(channels)).min(count - consumed));
                    for _ in 0..n {
                        assert_eq!(source.current_span_len(), Some(n));
                        assert_eq!(source.next(), Some(consumed as f32));
                        consumed += 1;
                    }
                }
                assert_eq!(consumed, count);
                assert_eq!(source.next(), None);
            }
        }
    }
    #[test]
    fn actual_repeat_keeps_channel_order_across_five_cycles() {
        for channels in [1, 2] {
            let count = 1027 * usize::from(channels);
            let data: Arc<[f32]> = (0..count).map(|i| i as f32).collect::<Vec<_>>().into();
            let source = LoopBuffer::new(
                ChannelCount::new(channels).unwrap(),
                SampleRate::new(44100).unwrap(),
                data,
            );
            let mut repeated = source.repeat_infinite();
            let mut consumed = 0;
            while consumed < count * 5 {
                let n = repeated.current_span_len().unwrap();
                assert!(n > 0 && n <= 1024 * usize::from(channels));
                assert_eq!(n % usize::from(channels), 0);
                for _ in 0..n {
                    assert_eq!(repeated.next(), Some((consumed % count) as f32));
                    consumed += 1;
                }
            }
            assert_eq!(repeated.total_duration(), None);
        }
    }
}
