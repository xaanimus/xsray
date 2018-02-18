extern crate rand;

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
pub use self::halton_private_module::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum SamplerSpec {
    Pseudorandom,
    Halton {
        base_x: u32,
        base_y: u32
    }
}

impl SamplerSpec {
    pub fn to_sampler(&self) -> Box<Sampler> {
        match self {
            &SamplerSpec::Pseudorandom => Box::new(PseudorandomSampler),
            &SamplerSpec::Halton { base_x, base_y } =>
                Box::new(
                    HaltonSampler::new(
                        HaltonBase::new(base_x).unwrap(),
                        HaltonBase::new(base_y).unwrap()))
        }
    }

    pub fn to_number_sequence(&self, number_of_samples: usize) -> NumberSequenceSampler {
        NumberSequenceSampler::new_from_sampler(
            self.to_sampler().as_mut(), number_of_samples)
    }
}

//each pixel resets its seed based on the pixel location
//each thread has a random number generator

pub trait Sampler: Debug {
    fn get_f32(&mut self) -> f32;

    fn get_2d_f32(&mut self) -> (f32, f32) {
        (self.get_f32(), self.get_f32())
    }

    fn get_usize_from_f32(&mut self, limit: usize) -> usize {
        (self.get_f32() as usize) % limit
    }
}

//A pseudorandom generator without a seed
#[derive(Debug)]
pub struct PseudorandomSampler;
impl Sampler for PseudorandomSampler {
    fn get_f32(&mut self) -> f32 {
        rand::random::<f32>()
    }
}

impl PseudorandomSampler {
    pub fn shared_threadlocal() -> Rc<RefCell<PseudorandomSampler>> {
        PSEUDORANDOM_SAMPLER.with(|s| s.clone())
    }
}

#[derive(Debug)]
pub struct HaltonSampler {
    idx: u32,
    base_x: HaltonBase,
    base_y: HaltonBase
}

impl Sampler for HaltonSampler {
    fn get_f32(&mut self) -> f32 {
        self.idx += 1;
        halton_sequence(self.idx, self.base_x)
    }

    fn get_2d_f32(&mut self) -> (f32, f32) {
        self.idx += 1;
        (halton_sequence(self.idx, self.base_x), halton_sequence(self.idx, self.base_y))
    }
}

impl HaltonSampler {
    fn new(base_x: HaltonBase, base_y: HaltonBase) -> HaltonSampler {
        HaltonSampler {
            idx: 0,
            base_x,
            base_y
        }
    }
}

thread_local! {
    static PSEUDORANDOM_SAMPLER: Rc<RefCell<PseudorandomSampler>> =
        Rc::new(RefCell::new(PseudorandomSampler));
}

mod halton_private_module {
    #[derive(Debug)]
    pub struct HaltonBaseMustBeGreaterThanOne;
    #[derive(Debug, Copy, Clone)]
    pub struct HaltonBase {
        base: u32
    }
    impl HaltonBase {
        pub fn new(base: u32) -> Result<HaltonBase, HaltonBaseMustBeGreaterThanOne> {
            if base <= 1 {
                Err(HaltonBaseMustBeGreaterThanOne)
            } else {
                Ok(HaltonBase {base})
            }
        }

        pub fn value(&self) -> u32 {self.base}
    }
}

// compute halton sequence
// from https://en.wikipedia.org/wiki/Halton_sequence
fn halton_sequence(idx: u32, base: HaltonBase) -> f32 {
    let mut f = 1f32;
    let mut r = 0f32;
    let mut i = idx;
    while i > 0 {
        f = f / base.value() as f32;
        r = r + f * ((i % base.value()) as f32);
        i = i / base.value(); //floor
    }
    r
}

#[derive(Debug, Clone)]
pub struct NumberSequenceSampler {
    sequence: Rc<Vec<(f32, f32)>>,
    idx: usize
}

impl Sampler for NumberSequenceSampler {
    fn get_f32(&mut self) -> f32 {
        self.get_2d_f32().0
    }

    fn get_2d_f32(&mut self) -> (f32, f32) {
        self.idx = (self.idx + 1) % self.sequence.len();
        self.sequence[self.idx]
    }
}

impl NumberSequenceSampler {
    pub fn seed_index(&mut self, seed: usize) {
        self.idx = seed % self.sequence.len()
    }

    pub fn reset(&mut self) {
        self.idx = 0
    }

    pub fn new_from_sampler<TSpl: Sampler + ?Sized>(
        sampler: &mut TSpl, number_of_samples: usize
    ) -> NumberSequenceSampler {
        NumberSequenceSampler {
            sequence: Rc::new((0..number_of_samples)
                .map(|_| sampler.get_2d_f32())
                .collect()),
            idx: 0
        }
    }

    pub fn reset_copy(&self) -> NumberSequenceSampler {
        NumberSequenceSampler {
            sequence: self.sequence.clone(),
            idx: 0
        }
    }
}


