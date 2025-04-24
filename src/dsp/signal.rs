//! Types for handling interleaved multi-channel sample buffers.

/// A sample buffer with an associated channel count.
pub trait Signal {
    /// Gets the number of channels of this buffer.
    fn channels(&self) -> usize;

    /// Gets the underlying sample data of this buffer.
    fn samples(&self) -> &[f32];

    /// Gets the number of full samples in this buffer.
    fn length(&self) -> usize {
        self.samples().len() / self.channels()
    }

    /// Provides a mono view of this buffer, downmixing as necessary.
    fn read_mono(&self) -> impl Iterator<Item = f32> {
        let ch = self.channels() as f32;
        self.samples()
            .chunks_exact(self.channels())
            .map(move |slc| slc.iter().sum::<f32>() / ch)
    }

    // TODO: get channel layout from FMOD - panning is potentially heavy?
    /// Provides a stereo view of this buffer, down- or upmixing as necessary.
    fn read_stereo(&self) -> (impl Iterator<Item = f32>, impl Iterator<Item = f32>) {
        (
            self.samples().chunks_exact(self.channels()).map(|slc| {
                if self.channels() <= 2 {
                    slc[0]
                } else {
                    slc.iter().sum::<f32>() / self.channels() as f32
                }
            }),
            self.samples().chunks_exact(self.channels()).map(|slc| {
                if self.channels() == 1 {
                    slc[0]
                } else if self.channels() == 2 {
                    slc[1]
                } else {
                    slc.iter().sum::<f32>() / self.channels() as f32
                }
            }),
        )
    }
}

/// A read-only sample buffer.
/// Use operations from [Signal] to interact with data.
#[derive(Copy, Clone)]
pub struct SignalConst<'a> {
    data: &'a [f32],
    channels: usize,
}

/// A read-write sample buffer.
pub struct SignalMut<'a> {
    data: &'a mut [f32],
    channels: usize,
}

impl<'a> SignalConst<'a> {
    /// Construct a [SignalConst] from its samples and channel count.
    /// Panics if `data.len()` is not a multiple of `channels`, or if `channels` is `0` and
    /// `data.len() != 0`.
    pub fn new(data: &'a [f32], channels: usize) -> Self {
        if channels != 0 {
            assert_eq!(data.len() % channels, 0);
        } else {
            assert_eq!(data.len(), 0);
        }
        SignalConst { data, channels }
    }
}

impl<'a> Signal for SignalConst<'a> {
    fn channels(&self) -> usize {
        self.channels
    }

    fn samples(&self) -> &[f32] {
        self.data
    }
}

impl<'a> SignalMut<'a> {
    /// Construct a [SignalMut] from its samples and channel count.
    /// Panics if `data.len()` is not a multiple of `channels`, or if `channels` is `0` and
    /// `data.len() != 0`.
    pub fn new(data: &'a mut [f32], channels: usize) -> Self {
        if channels != 0 {
            assert_eq!(data.len() % channels, 0);
        } else {
            assert_eq!(data.len(), 0);
        }
        SignalMut { data, channels }
    }

    /// Writes a sample to the buffer across all channels.
    pub fn write_sample(&mut self, n: usize, v: f32) {
        for ch in 0..self.channels {
            self.data[n * self.channels + ch] = v;
        }
    }

    /// Fills the channel with a constant value.
    pub fn fill(&mut self, v: f32) {
        self.data.fill(v);
    }

    /// Gets the underlying sample data of this buffer mutably.
    pub fn samples_mut(&mut self) -> &mut [f32] {
        self.data
    }
}

impl<'a> Signal for SignalMut<'a> {
    fn channels(&self) -> usize {
        self.channels
    }

    fn samples(&self) -> &[f32] {
        self.data
    }
}
