use std::fmt;
use std::ptr::NonNull;

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
        let alloc: Box<[T]> = (0..width * height)
            .map(|_| T::default())
            .collect::<Vec<_>>()
            .into();
        // SAFETY: Box::into_raw always returns a non-null pointer
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(alloc).cast::<T>()) };
        Self { ptr, width, height }
    }
    pub fn row(&self, i: usize) -> &[T] {
        self.row_checked(i).unwrap()
    }
    pub fn row_checked(&self, i: usize) -> Option<&[T]> {
        // SAFETY: We only call `row_unchecked` once we have checked `i < self.width`
        (i < self.width).then(|| unsafe { self.row_unchecked(i) })
    }
    pub unsafe fn row_unchecked<'a>(&'a self, i: usize) -> &'a [T] {
        let ptr = self.ptr.as_ptr();
        // SAFETY: `self.ptr` is valid by the invariants of the type. The caller must ensure `i < self.width`, meaning this pointer is in bounds. Vec and Box never allocate more than isize::MAX bytes, so this add will not overflow.
        let start = unsafe { ptr.add(self.width * i) };
        unsafe { std::slice::from_raw_parts(start, self.width) }
    }
    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
}

impl<T> Drop for Board<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.as_ptr();
        let len = self.width * self.height;
        let fat_ptr = unsafe { std::slice::from_raw_parts_mut(ptr, len) } as *mut [T];
        let _ = unsafe{ Box::from_raw(fat_ptr) };
    }
}

impl fmt::Display for Board<Option<bool>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            if y > 0 {
                writeln!(f)?;
            }
            for v in self.row(y) {
                let c = match v {
                    None => '?',
                    Some(false) => '.',
                    Some(true) => 'X',
                };
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}
