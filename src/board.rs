use std::fmt;
use std::ptr::NonNull;
use std::slice;

pub struct Board<T> {
    ptr: NonNull<T>,
    width: usize,
    height: usize,
}

impl<T> Board<T> {
    pub fn new_default(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        Self::new_with(width, height, T::default)
    }
    pub fn new_with<F>(width: usize, height: usize, mut f: F) -> Self
    where
        F: FnMut() -> T,
    {
        let vec = (0..width * height).map(|_| f()).collect();
        unsafe { Self::from_vec(vec, width, height) }
    }
    unsafe fn from_vec(vec: Vec<T>, width: usize, height: usize) -> Self {
        let alloc: Box<[T]> = vec.into();
        let raw_ptr = Box::into_raw(alloc).cast::<T>();
        // SAFETY: Box::into_raw always returns a non-null pointer
        let ptr = unsafe { NonNull::new_unchecked(raw_ptr) };
        Self { ptr, width, height }
    }
    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
    pub fn set_row(&mut self, i: usize, vec: Vec<T>) {
        let slice = self.row_checked_mut(i).unwrap();
        assert_eq!(slice.len(), vec.len());
        for (s, v) in slice.iter_mut().zip(vec) {
            *s = v;
        }
    }
    pub fn set_col(&mut self, x: usize, vec: Vec<T>) {
        assert!(x < self.width);
        assert_eq!(vec.len(), self.height);
        for (y, v) in vec.into_iter().enumerate() {
            unsafe {
                *self.pos_unchecked_mut(x, y) = v;
            }
        }
    }
    pub fn row(&self, i: usize) -> &[T] {
        self.row_checked(i).unwrap()
    }
    pub fn row_checked(&self, i: usize) -> Option<&[T]> {
        // SAFETY: We only call `row_unchecked` once we have checked `i < self.height`
        (i < self.height).then(|| unsafe { self.row_unchecked(i) })
    }
    pub fn row_checked_mut(&mut self, i: usize) -> Option<&mut [T]> {
        // SAFETY: We only call `row_unchecked_mut` once we have checked `i < self.width`
        (i < self.width).then(|| unsafe { self.row_unchecked_mut(i) })
    }
    pub unsafe fn row_unchecked(&self, i: usize) -> &[T] {
        let ptr = self.ptr.as_ptr();
        // SAFETY: `self.ptr` is valid by the invariants of the type. The caller must ensure `i < self.width`, meaning this pointer is in bounds. Vec and Box never allocate more than isize::MAX bytes, so this add will not overflow.
        let start = unsafe { ptr.add(self.width * i) };
        unsafe { slice::from_raw_parts(start, self.width) }
    }
    pub unsafe fn row_unchecked_mut(&mut self, i: usize) -> &mut [T] {
        let ptr = self.ptr.as_ptr();
        // SAFETY: `self.ptr` is valid by the invariants of the type. The caller must ensure `i < self.width`, meaning this pointer is in bounds. Vec and Box never allocate more than isize::MAX bytes, so this add will not overflow.
        let start = unsafe { ptr.add(self.width * i) };
        unsafe { slice::from_raw_parts_mut(start, self.width) }
    }
    pub unsafe fn pos_unchecked_mut(&mut self, x: usize, y: usize) -> &mut T {
        let ptr = self.ptr.as_ptr();
        // SAFETY: `self.ptr` is valid by the invariants of the type. The caller must ensure `i < self.width`, meaning this pointer is in bounds. Vec and Box never allocate more than isize::MAX bytes, so this add will not overflow.
        unsafe { &mut *ptr.add(x + self.width * y) }
    }
    pub fn pos_checked_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        (x < self.width && y < self.height).then(|| unsafe { self.pos_unchecked_mut(x, y) })
    }
    pub fn pos_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.pos_checked_mut(x, y).unwrap()
    }
    pub const unsafe fn pos_unchecked(&self, x: usize, y: usize) -> &T {
        // FIXME: cast to `*const T` required to make this function `const`.
        let ptr = self.ptr.as_ptr() as *const T;
        // SAFETY: `self.ptr` is valid by the invariants of the type. The caller must ensure `i < self.width`, meaning this pointer is in bounds. Vec and Box never allocate more than isize::MAX bytes, so this add will not overflow.
        unsafe { &*ptr.add(x + self.width * y) }
    }
    pub fn as_slice(&self) -> &[T] {
        let ptr = self.ptr.as_ptr();
        unsafe { slice::from_raw_parts(ptr, self.width * self.height) }
    }
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let ptr = self.ptr.as_ptr();
        unsafe { slice::from_raw_parts_mut(ptr, self.width * self.height) }
    }
}

impl<T> Board<T>
where
    T: Clone,
{
    pub fn new(width: usize, height: usize, value: T) -> Self {
        let vec = vec![value; width * height];
        unsafe { Self::from_vec(vec, width, height) }
    }
    pub unsafe fn col_unchecked(&self, x: usize) -> Vec<T> {
        (0..self.height)
            .map(|y| unsafe { self.pos_unchecked(x, y) }.clone())
            .collect()
    }
    pub fn col_checked(&self, x: usize) -> Option<Vec<T>> {
        // SAFETY: We only call `col_unchecked` once we have checked `i < self.width`
        (x < self.width).then(|| unsafe { self.col_unchecked(x) })
    }
    pub fn col(&self, x: usize) -> Vec<T> {
        self.col_checked(x).unwrap()
    }
    pub fn set_row_slice(&mut self, i: usize, src: &[T]) {
        let slice = self.row_checked_mut(i).unwrap();
        assert_eq!(slice.len(), src.len());
        for (s, v) in slice.iter_mut().zip(src.iter().cloned()) {
            *s = v;
        }
    }
}

impl<T> Clone for Board<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let vec = self.as_slice().to_vec();
        unsafe { Self::from_vec(vec, self.width, self.height) }
    }
}

impl<T> fmt::Debug for Board<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("data", &self.as_slice())
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl<T> Default for Board<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            width: 0,
            height: 0,
        }
    }
}

impl<T> Drop for Board<T> {
    fn drop(&mut self) {
        let fat_ptr = self.as_slice_mut() as *mut [T];
        let _drop = unsafe { Box::from_raw(fat_ptr) };
    }
}

impl fmt::Display for Board<bool> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            if y > 0 {
                writeln!(f)?;
            }
            for v in self.row(y) {
                let c = display_bool(*v);
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Board<Option<bool>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            if y > 0 {
                writeln!(f)?;
            }
            for v in self.row(y) {
                let c = display_option_bool(*v);
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

const fn display_bool(x: bool) -> char {
    if x {
        'X'
    } else {
        '.'
    }
}

const fn display_option_bool(x: Option<bool>) -> char {
    match x {
        Some(y) => display_bool(y),
        None => '?',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_dangling_drop() {
        let _drop: Board<bool> = Board::default();
    }
}
