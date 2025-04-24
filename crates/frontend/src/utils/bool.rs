#[inline]
pub fn fmt_if<T>(b: bool, yes: T, no: T) -> T {
    if b {
        yes
    } else {
        no
    }
}
