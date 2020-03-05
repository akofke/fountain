use core::mem::MaybeUninit;
use std::ops::{Index, IndexMut};

pub struct BlockedArray<T, const LOG_BLOCK_SIZE: usize> {
    contents: Vec<MaybeUninit<T>>,
    u_size: usize,
    v_size: usize,
    u_blocks: usize,
    total_elems: usize,
}

impl<T: Copy, const LOG_BLOCK_SIZE: usize> BlockedArray<T, {LOG_BLOCK_SIZE}> {
    /// Size of a block along one axis.
    const BLOCK_SIZE: usize = 1 << LOG_BLOCK_SIZE;

    /// Total number of elements in a block.
    const BLOCK_LEN: usize = Self::BLOCK_SIZE * Self::BLOCK_SIZE;

    /// Round up a number to be a multiple of the block size
    fn round_up(n: usize) -> usize {
        (n + Self::BLOCK_SIZE - 1) & !(Self::BLOCK_SIZE - 1)
    }

    /// Find the block index along a certain dimension given the element index in that direction
    fn block(a: usize) -> usize {
        a >> LOG_BLOCK_SIZE
    }

    /// Find the offset into a block along a dimension for the given element index
    fn offset(a: usize) -> usize {
        a & (Self::BLOCK_SIZE - 1)
    }

    pub fn new(data: &[T], u_size: usize, v_size: usize) -> Self {
        let n_alloc = Self::round_up(u_size) * Self::round_up(v_size);
        let mut contents = vec![MaybeUninit::<T>::uninit(); n_alloc];
        let u_blocks = Self::round_up(u_size) >> LOG_BLOCK_SIZE;
        let total_elems = data.len();
        assert_eq!(u_size * v_size, total_elems);

        for v in 0..v_size {
            for u in 0..u_size {
                let elem = data[v * u_size + u];
                let index = Self::get_index(u, v, u_blocks);
                contents[index] = MaybeUninit::new(elem);
            }
        }
        Self {
            contents,
            u_size,
            v_size,
            u_blocks,
            total_elems,
        }
    }


    /// Get the index
    fn get_index(u: usize, v: usize, u_blocks: usize) -> usize {
        let block_u = Self::block(u);
        let block_v = Self::block(v);
        let offset_u = Self::offset(u);
        let offset_v = Self::offset(v);
        let block = block_v * u_blocks + block_u;
        (Self::BLOCK_LEN * block) + (offset_v * Self::BLOCK_SIZE + offset_u)
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.u_size, self.v_size)
    }

    pub fn u_size(&self) -> usize {
        self.u_size
    }

    pub fn v_size(&self) -> usize {
        self.v_size
    }

    pub fn total_elements(&self) -> usize {
        self.total_elems
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut elems = Vec::with_capacity(self.u_size * self.v_size);
        for u in 0..self.u_size() {
            for v in 0..self.v_size() {
                elems.push(self[(u, v)])
            }
        }
        elems
    }
}

impl<T: Default + Copy, const LOG_BLOCK_SIZE: usize> BlockedArray<T, {LOG_BLOCK_SIZE}> {
    pub fn default(u_size: usize, v_size: usize) -> Self {
        let n_alloc = Self::round_up(u_size) * Self::round_up(v_size);
        let mut contents = vec![MaybeUninit::<T>::uninit(); n_alloc];
        let u_blocks = Self::round_up(u_size) >> LOG_BLOCK_SIZE;
        let total_elems = u_size * v_size;

        for v in 0..v_size {
            for u in 0..u_size {
                let index = Self::get_index(u, v, u_blocks);
                contents[index] = MaybeUninit::new(T::default());
            }
        }

        Self {
            contents,
            u_size,
            v_size,
            u_blocks,
            total_elems,
        }
    }
}

impl<T: Copy> BlockedArray<T, {2}> {
    pub fn with_default_block_size(data: &[T], u_size: usize, v_size: usize) -> Self {
        Self::new(data, u_size, v_size)
    }
}

impl<T: Copy, const LOG_BLOCK_SIZE: usize> Index<(usize, usize)> for BlockedArray<T, {LOG_BLOCK_SIZE}> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (u, v) = index;
        let idx = Self::get_index(u, v, self.u_blocks);
        unsafe { self.contents[idx].get_ref() }
    }
}

impl<T: Copy, const LOG_BLOCK_SIZE: usize> IndexMut<(usize, usize)> for BlockedArray<T, {LOG_BLOCK_SIZE}> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (u, v) = index;
        let idx = Self::get_index(u, v, self.u_blocks);
        unsafe { self.contents[idx].get_mut() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_up() {
        assert_eq!(BlockedArray::<f32, {2}>::round_up(3), 4)
    }

    #[test]
    fn test_blocked_array() {
        let ulen = 8;
        let vlen = 8;
        let elems: Vec<usize> = (0..(ulen * vlen)).collect();
        let blocked_array = BlockedArray::with_default_block_size(elems.as_slice(), ulen, vlen);

        for u in 0..ulen {
            for v in 0..vlen {
                let expected_elem = elems[v * ulen + u];
                assert_eq!(blocked_array[(u, v)], expected_elem, "{:?}", (ulen, vlen));
            }
        }
    }

    #[test]
    fn test_blocked_array_non_multiple_size() {
        let ulen = 7;
        let vlen = 7;
        let elems: Vec<usize> = (0..(ulen * vlen)).collect();
        let blocked_array = BlockedArray::with_default_block_size(elems.as_slice(), ulen, vlen);

        for u in 0..ulen {
            for v in 0..vlen {
                let expected_elem = elems[v * ulen + u];
                assert_eq!(blocked_array[(u, v)], expected_elem, "{:?}", (ulen, vlen));
            }
        }
    }

    #[test]
    fn test_blocked_array_mismatched_dims() {
        let ulen = 8;
        let vlen = 13;
        let elems: Vec<usize> = (0..(ulen * vlen)).collect();
        let blocked_array = BlockedArray::with_default_block_size(elems.as_slice(), ulen, vlen);

        for u in 0..ulen {
            for v in 0..vlen {
                let expected_elem = elems[v * ulen + u];
                assert_eq!(blocked_array[(u, v)], expected_elem, "{:?}", (ulen, vlen));
            }
        }
    }

    #[test]
    fn test_default() {
        let ulen = 8;
        let vlen = 13;
        let blocked_array = BlockedArray::<f32, 2>::default(ulen, vlen);

        for u in 0..ulen {
            for v in 0..vlen {
                assert_eq!(blocked_array[(u, v)], 0.0, "{:?}", (ulen, vlen));
            }
        }
    }
}