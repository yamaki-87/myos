use x86_64::instructions::interrupts;

pub fn without_interrupts_fn<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    interrupts::without_interrupts(f)
}
