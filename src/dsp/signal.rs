/// A sample buffer with an associated channel count.
pub trait Signal {
    fn channels(&self) -> usize;

    fn samples(&self) -> &[f32];

    fn length(&self) -> usize {
        self.samples().len() / self.channels()
    }

    fn read_mono(&self) -> impl Iterator<Item = f32> {
        let ch = self.channels() as f32;
        self.samples()
            .chunks_exact(self.channels())
            .map(move |slc| slc.iter().sum::<f32>() / ch)
    }

    // can't automatically decide channel layout... pick sensible default & provide override?
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
    /// Panics if `data.len()` is not a multiple of `channels`.
    pub fn new(data: &'a [f32], channels: usize) -> Self {
        assert_eq!(data.len() % channels, 0);
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
    /// Panics if `data.len()` is not a multiple of `channels`.
    pub fn new(data: &'a mut [f32], channels: usize) -> Self {
        assert_eq!(data.len() % channels, 0);
        SignalMut { data, channels }
    }

    pub fn write_sample(&mut self, n: usize, v: f32) {
        for ch in 0..self.channels {
            self.data[n * self.channels + ch] = v;
        }
    }

    pub fn fill(&mut self, v: f32) {
        self.data.fill(v);
    }

    pub(crate) fn samples_mut(&mut self) -> &mut [f32] {
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
