use crate::fixed::Fix64;

const SIN_PI_OVER_2_POLY_COEFFS: &[Fix64] = &[
    Fix64::from_rat(26353589, 16777216),  // x^1
    Fix64::from_rat(-10837479, 16777216), // x^3
    Fix64::from_rat(334255, 4194304),     // x^5
    Fix64::from_rat(-78547, 16777216),    // x^7
    Fix64::from_rat(673, 4194304),        // x^9
];

fn sin_pi_over_2_poly(x: Fix64) -> Fix64 {
    let x_sq = x * x;
    let mut retval = Fix64::from(0);
    for coeff in SIN_PI_OVER_2_POLY_COEFFS.iter().rev() {
        retval = retval.mul_add(x_sq, *coeff);
    }
    retval * x
}

const COS_PI_OVER_2_POLY_COEFFS: &[Fix64] = &[
    Fix64::from_rat(1, 1),                // x^0
    Fix64::from_rat(-20698061, 16777216), // x^2
    Fix64::from_rat(1063967, 4194304),    // x^4
    Fix64::from_rat(-350031, 16777216),   // x^6
    Fix64::from_rat(15423, 16777216),     // x^8
];

fn cos_pi_over_2_poly(x: Fix64) -> Fix64 {
    let x_sq = x * x;
    let mut retval = Fix64::from(0);
    for coeff in COS_PI_OVER_2_POLY_COEFFS.iter().rev() {
        retval = retval.mul_add(x_sq, *coeff);
    }
    retval
}

pub fn sin_cos_pi(mut x: Fix64) -> (Fix64, Fix64) {
    x >>= 1;
    x = x.floor_fract();
    x <<= 2;
    let xi = x.round();
    x -= Fix64::from(xi);
    match xi & 3 {
        0 => (sin_pi_over_2_poly(x), cos_pi_over_2_poly(x)),
        1 => (cos_pi_over_2_poly(x), -sin_pi_over_2_poly(x)),
        2 => (-sin_pi_over_2_poly(x), -cos_pi_over_2_poly(x)),
        3 => (-cos_pi_over_2_poly(x), sin_pi_over_2_poly(x)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sincospi() {
        #[derive(Debug, Copy, Clone)]
        #[allow(dead_code)]
        struct Error {
            v: Fix64,
            fv: f64,
            fsin: f64,
            fcos: f64,
            sin: Fix64,
            cos: Fix64,
            eps: f64,
            sin_dist: f64,
            cos_dist: f64,
            max_dist: f64,
        }
        let mut worst_error = None;
        for i in (Fix64::from(-4i64).as_bits()..=Fix64::from(4i64).as_bits()).step_by(12345) {
            let v = Fix64::from_bits(i);
            let fv = v.to_f64();
            let (fsin, fcos) = (fv * std::f64::consts::PI).sin_cos();
            let (sin, cos) = sin_cos_pi(v);
            let eps = Fix64::from_bits(5).to_f64();
            let sin_dist = (sin.to_f64() - fsin).abs();
            let cos_dist = (cos.to_f64() - fcos).abs();
            let max_dist = sin_dist.max(cos_dist);
            match worst_error {
                Some(Error { max_dist: d, .. }) if d > max_dist => {}
                _ => {
                    worst_error = Some(Error {
                        v,
                        fv,
                        fsin,
                        fcos,
                        sin,
                        cos,
                        eps,
                        sin_dist,
                        cos_dist,
                        max_dist,
                    })
                }
            }
        }
        let Some(worst_error @ Error { eps, max_dist, .. }) = worst_error else {
            return;
        };
        assert!(max_dist < eps, "{worst_error:?}");
    }
}
