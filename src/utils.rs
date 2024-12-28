pub(crate) unsafe fn ref_cast_mut<T: ?Sized, U>(ptr: &mut T) -> &mut U {
    unsafe { &mut *(ptr as *mut T as *mut U) }
}
