use num::{Num, FromPrimitive};
// TODO impl Iterator

macro_rules! create_bump_buf {
    ($name:ident, $size:expr) => {
        #[derive(Clone)]
        pub struct $name<N: Default + Copy>(usize, [N; $size], bool);
        
        impl<N: Default + Copy> $name<N> {
            pub fn new() -> Self {
                $name(0, [N::default(); $size], false)
            }
        }

        impl<N: Default + Copy> BumpBufPrivate<N> for $name<N> {
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

        impl<N: Default + Copy> BumpBuf<N> for $name<N> {}
    };
}
// impl<N: Default + Num + FromPrimitive + Copy> BumpBufNum<N> for $name<N> {}

trait BumpBufPrivate<N: Default + Copy> {
    fn arr(&self) -> &[N];
    fn arr_mut(&mut self) -> &mut [N]; 
    fn idx(&self) -> usize;
    fn reset_idx(&mut self);
    fn increment_idx(&mut self);
    fn past_valid(&self) -> bool;
    fn set_past_valid(&mut self, past_valid: bool);
}

// pub trait BumpBufNum<N: Default + Num + FromPrimitive + Copy>: BumpBuf<N> {
//     fn calc_slope(&self) -> Option<N> {
//         let x2 = if self.past_valid() { 
//             self.arr().len() - 1
//         } else if self.idx() > 0 { 
//             self.idx() - 1
//         } else { 
//             return None // cannot divide by zero
//         };

//         Some((self.recent() - self.last()) / N::from_usize(x2).unwrap()) // x1 always 0 for calc purposes
//     }
// }

pub trait BumpBuf<N: Default + Copy>: BumpBufPrivate<N> {
    fn push(&mut self, val: N) {
        let idx = self.idx();
        self.arr_mut()[idx] = val;
        self.increment_idx();
        if self.idx() == self.arr().len() {
            self.reset_idx();
            self.set_past_valid(true); // Data at idx and above is now valid past data
        }
    }

    /// Returns he most recent element in the buffer
    #[inline]
    fn recent(&self) -> N {
        let idx = if self.idx() == 0 {
            self.arr().len() - 1
        } else {
            self.idx() - 1
        };
        self.arr()[idx]
    }

    /// Returns the oldest element in the buffer
    fn last(&self) -> N {
        let idx = if self.past_valid() {
            self.idx()
        } else {
            0
        };
        self.arr()[idx]
    }

    /// Returns the second most recent element in the buffer
    /// Will return a default value if 0 or 1 elements have been pushed
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

    // TODO needs testing and is wrong
    fn nth(&self, idx: usize) -> Option<N> {
        if idx >= self.arr().len() || (idx >= self.idx() && !self.past_valid()) { 
            None
        } else if self.past_valid() {
            let wrapped_idx = self.idx() as isize - 1 - idx as isize;
            if wrapped_idx >= 0 {
                Some(self.arr()[wrapped_idx as usize - 1])
            } else {
                Some(self.arr()[self.idx() + idx + 1])
            }
        } else {
            Some(self.arr()[idx])
        }
    }

    /// Returns true if the internal buffer was just filled
    fn end_of_internal(&self) -> bool {
        self.idx() == 0 && self.past_valid()
    }

    fn iter(&self) -> BumpBufIterator<N> {
        BumpBufIterator {
            current: if self.past_valid() { self.idx() } else { 0 },
            end: self.idx(),
            looped: false,
            use_past: self.past_valid(), 
            buf: self.arr()
        }
    }
}

pub struct BumpBufIterator<'a, N: Default + Copy> {
    current: usize,
    end: usize,
    looped: bool,
    use_past: bool,
    buf: &'a [N]
}

impl<'a, N: Default + Copy> Iterator for BumpBufIterator<'a, N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end && (!self.use_past || self.looped) {
            None
        } else {
            let item = Some(self.buf[self.current]);
            self.current += 1;
            if self.current == self.buf.len() && self.use_past {
                self.current = 0;
                self.looped = true;
            }
            item
        }
    }
}

create_bump_buf!(BumpBuf8, 8);
create_bump_buf!(BumpBuf16, 16);
create_bump_buf!(BumpBuf32, 32);
create_bump_buf!(BumpBuf50, 50);
create_bump_buf!(BumpBuf64, 64);
create_bump_buf!(BumpBuf128, 128);
create_bump_buf!(BumpBuf250, 250);
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
        // assert_eq!(bump_buf.calc_slope(), 1f32);
    }

    #[test]
    fn test_nth() {
        let mut bump_buf = BumpBuf8::new();
        (0..12).into_iter().for_each(|v| bump_buf.push(v));

        assert_eq!(bump_buf.nth(1), Some(5));
        assert_eq!(bump_buf.nth(8), Some(11));
        assert_eq!(bump_buf.nth(9), None);
    }

    #[test]
    fn test_iterator() {
        let mut bump_buf = BumpBuf8::<u32>::new();
        (0..=200).into_iter().for_each(|n| bump_buf.push(n));
        bump_buf.iter().for_each(|n| println!("{}", n));
        assert_eq!(bump_buf.last(), bump_buf.iter().nth(0).unwrap());
        assert_eq!(bump_buf.recent(), bump_buf.iter().nth(7).unwrap());
        // assert_eq!(bump_buf.nth(4).unwrap(), bump_buf.iter().nth(4).unwrap());
    }
}
