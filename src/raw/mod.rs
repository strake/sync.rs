pub mod spin;

pub unsafe trait Lock {
    const new: Self;
    fn lock(&self, mut_: bool);
    fn unlock(&self, mut_: bool);
    fn try_lock(&self, mut_: bool) -> bool;

    #[inline]
    fn try_upgrade(&self) -> bool { false }
}
