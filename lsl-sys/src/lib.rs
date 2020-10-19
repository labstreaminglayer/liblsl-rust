#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod generated;

pub use generated::*;

#[cfg(test)]
mod tests {
    use crate::lsl_local_clock;

    #[test]
    // bare-bones test to confirm that liblsl is linked at all
    fn test_properly_linked() {
        unsafe {
            lsl_local_clock();
        }
    }
}
