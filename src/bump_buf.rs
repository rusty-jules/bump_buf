macro_rules! create_bump_buf {
    ($name:ident, $size:expr) => {
        #[derive(Clone)]
        pub struct $name<N: Default + Copy>(usize, [N; $size], bool);
        
        impl<N: Default + Copy> $name<N> {
            pub fn new() -> Self {
                $name(0, [N::default(); $size], false)
            }
        }

        impl<N: Default + Copy> $name<N> {
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

            /// Add an element on the front of the buffer, evicting the last element
            pub fn push(&mut self, val: N) {
                let idx = self.idx();
                self.arr_mut()[idx] = val;
                self.increment_idx();
                if self.idx() == self.arr().len() {
                    self.reset_idx();
                    self.set_past_valid(true); // Data at idx and above is now valid past data
                }
            }
        
            /// Returns the most recently pushed element in the buffer
            #[inline]
            pub fn recent(&self) -> N {
                let idx = if self.idx() == 0 {
                    self.arr().len() - 1
                } else {
                    self.idx() - 1
                };
                self.arr()[idx]
            }
        
            /// Returns the oldest element in the buffer
            pub fn last(&self) -> N {
                let idx = if self.past_valid() {
                    self.idx()
                } else {
                    0
                };
                self.arr()[idx]
            }
        
            /// Returns the second most recent element in the buffer
            /// Will return a default value if 0 or 1 elements have been pushed
            pub fn prev(&self) -> N {
                let idx = if self.idx() == 0 {
                    self.arr().len() - 2
                } else if self.idx() == 1 {
                    self.arr().len() - 1
                } else {
                    self.idx() - 2
                };
                self.arr()[idx]
            }
            
            /// Returns the length of the underlying array
            #[inline]
            pub fn len(&self) -> usize {
                self.arr().len()
            }
        
            /// Returns true if the most recent item was written to the last index of the internal array
            pub fn end_of_internal(&self) -> bool {
                self.idx() == 0 && self.past_valid()
            }
        
            pub fn iter(&self) -> BumpBufIterator<N> {
                BumpBufIterator {
                    current: if self.past_valid() { self.idx() } else { 0 },
                    end: self.idx(),
                    looped: false,
                    use_past: self.past_valid(), 
                    buf: self.arr()
                }
            }
        }
    };
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
create_bump_buf!(BumpBuf25, 25);
create_bump_buf!(BumpBuf32, 32);
create_bump_buf!(BumpBuf50, 50);
create_bump_buf!(BumpBuf64, 64);
create_bump_buf!(BumpBuf100, 100);
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
    fn test_push() {
        let mut bump_buf = BumpBuf512::new();
        let range = 51209;
        (0..=51209).into_iter().for_each(|v| bump_buf.push(v as f32));

        assert_eq!(bump_buf.recent(), range as f32);
        assert_eq!(bump_buf.prev(), (range - 1) as f32);
        assert_eq!(bump_buf.last(), (range - 511) as f32);
    }

    #[test]
    fn test_iterator() {
        let mut bump_buf = BumpBuf8::<u32>::new();
        (0..=200).into_iter().for_each(|n| bump_buf.push(n));
        bump_buf.iter().for_each(|n| println!("{}", n));
        assert_eq!(bump_buf.last(), bump_buf.iter().nth(0).unwrap());
        assert_eq!(bump_buf.recent(), bump_buf.iter().nth(7).unwrap());
    }
}
