#[allow(non_camel_case_types)]
mod generated;

pub use generated::*;

#[cfg(test)]
mod tests {
    use crate::lsl_local_clock;

    #[test]
    fn test_properly_linked() {
        unsafe {
            lsl_local_clock();
        }
    }
}
