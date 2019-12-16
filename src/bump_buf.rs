use num::{Num, FromPrimitive};
// TODO impl Iterator

macro_rules! create_bump_buf {
    ($name:ident, $size:expr) => {
        #[derive(Clone)]
        pub struct $name<N: Num + FromPrimitive + Copy>(usize, [N; $size], bool);
        
        impl<N: Num + FromPrimitive + Copy> $name<N> {
            pub fn new() -> Self {
                $name(0, [N::zero(); $size], false)
            }
        }

        impl<N: Num + FromPrimitive + Copy> BumpBufPrivate<N> for $name<N> {
            #[inline]
            fn arr(&self) -> &[N] {
                &self.1[..]
            }
            
            #[inline]
            fn arr_mut(&mut self) -> &mut [N] {
                &mut self.1[..]
            }
            
            #[inline]
            fn set_past_valid(&mut self, use_past: bool) {
                self.2 = use_past
            }

            #[inline]
            fn idx(&self) -> usize {
                self.0
            }

            #[inline]
            fn increment_idx(&mut self) {
                self.0 += 1
            }
        
            #[inline]
            fn reset_idx(&mut self) {
                self.0 = 0
            }
        
            #[inline]
            fn past_valid(&self) -> bool {
                self.2
            }
        }

        impl<N: Num + FromPrimitive + Copy> BumpBuf<N> for $name<N> {}

    };
}

trait BumpBufPrivate<N: Num + FromPrimitive + Copy> {
    fn arr(&self) -> &[N];
    fn arr_mut(&mut self) -> &mut [N]; 
    fn idx(&self) -> usize;
    fn reset_idx(&mut self);
    fn increment_idx(&mut self);
    fn past_valid(&self) -> bool;
    fn set_past_valid(&mut self, past_valid: bool);
}

pub trait BumpBuf<N: Num + FromPrimitive + Copy>: BumpBufPrivate<N> {
    fn push(&mut self, val: N) {
        let idx = self.idx();
        self.arr_mut()[idx] = val;
        self.increment_idx();
        if self.idx() == self.arr().len() {
            self.reset_idx();
            self.set_past_valid(true); // Data at idx and above is now valid past data
        }
    }

    #[inline]
    fn recent(&self) -> N {
        let idx = if self.idx() == 0 {
            self.arr().len() - 1
        } else {
            self.idx() - 1
        };
        self.arr()[idx]
    }

    fn last(&self) -> N {
        let idx = if self.past_valid() {
            self.idx()
        } else {
            0
        };
        self.arr()[idx]
    }

    /// Might return invalid "0.0"
    fn prev(&self) -> N {
        let idx = if self.idx() == 0 {
            self.arr().len() - 2
        } else if self.idx() == 1 {
            self.arr().len() - 1
        } else {
            self.idx() - 2
        };
        self.arr()[idx]
    }

    #[inline]
    fn len(&self) -> usize {
        self.arr().len()
    }


    // TODO needs testing
    fn nth(&self, idx: usize) -> Option<N> {
        if idx >= self.arr().len() || (idx >= self.idx() && !self.past_valid()) { 
            None
        } else if self.past_valid() {
            let wrapped_idx = self.idx() as isize - idx as isize;
            if wrapped_idx >= 0 {
                Some(self.arr()[idx])
            } else {
                Some(self.arr()[self.idx() + idx + 1])
            }
        } else {
            Some(self.arr()[idx])
        }
    }

    fn calc_slope(&self) -> N {
        let x2 = if self.past_valid() { 
            self.arr().len() - 1
        } else if self.idx() > 0 { 
            self.idx() - 1
        } else { 
            0
        };

        (self.recent() - self.last()) / N::from_usize(x2).unwrap() // x1 always 0 for calc purposes
    }
}

create_bump_buf!(BumpBuf8, 8);
create_bump_buf!(BumpBuf16, 16);
create_bump_buf!(BumpBuf32, 32);
create_bump_buf!(BumpBuf64, 64);
create_bump_buf!(BumpBuf128, 128);
create_bump_buf!(BumpBuf256, 256);
create_bump_buf!(BumpBuf512, 512);
create_bump_buf!(BumpBuf1024, 1024);
create_bump_buf!(BumpBuf2056, 2056);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slope() {
        let mut bump_buf = BumpBuf512::new();
        let range = 51209;
        (0..=51209).into_iter().for_each(|v| bump_buf.push(v as f32));

        assert_eq!(bump_buf.recent(), range as f32);
        assert_eq!(bump_buf.prev(), (range - 1) as f32);
        assert_eq!(bump_buf.last(), (range - 511) as f32);
        assert_eq!(bump_buf.calc_slope(), 1f32);
    }
}
