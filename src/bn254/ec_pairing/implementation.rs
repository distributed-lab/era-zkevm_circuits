use std::sync::Arc;

use boojum::{
    gadgets::non_native_field::traits::NonNativeField,
    pairing::bn256::{Fq2, FROBENIUS_COEFF_FQ6_C1, XI_TO_Q_MINUS_1_OVER_2},
};

use super::*;

// Curve parameter for the BN256 curve
const CURVE_U_PARAMETER: u64 = 4965661367192848881;
const SIX_U_PLUS_TWO_WNAF: [i8; 65] = [
    0, 0, 0, 1, 0, 1, 0, -1, 0, 0, 1, -1, 0, 0, 1, 0, 0, 1, 1, 0, -1, 0, 0, 1, 0, -1, 0, 0, 0, 0,
    1, 1, 1, 0, 0, -1, 0, 0, 1, 0, 0, 0, 0, 0, -1, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 1, 1, 0, -1, 0,
    0, 1, 0, 1, 1,
];
pub const U_WNAF: [i8; 63] = [
    1, 0, 0, 0, 1, 0, 1, 0, 0, -1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0,
    0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, -1, 0, 0, 0,
    1,
];

/// Struct for the line function evaluation for the BN256 curve (addition and doubling).
/// The line function is used in the Miller loop of the pairing function.
pub struct LineFunctionEvaluation<F, CS>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    c0: BN256Fq2NNField<F>,
    c3: BN256Fq2NNField<F>,
    c4: BN256Fq2NNField<F>,
    point: BN256SWProjectivePointTwisted<F>,
    _marker: std::marker::PhantomData<CS>,
}

impl<F, CS> LineFunctionEvaluation<F, CS>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    /// Creates a zero instance of the line function evaluation for the BN256 curve.
    pub fn zero(cs: &mut CS, params: &Arc<BN256BaseNNFieldParams>) -> Self {
        Self {
            c0: BN256Fq2NNField::zero(cs, params),
            c3: BN256Fq2NNField::zero(cs, params),
            c4: BN256Fq2NNField::zero(cs, params),
            point: BN256SWProjectivePointTwisted::zero(cs, params),
            _marker: std::marker::PhantomData::<CS>,
        }
    }

    /// Returns the point of the line function evaluation.
    pub fn point(&self) -> BN256SWProjectivePointTwisted<F> {
        self.point.clone()
    }

    /// Returns the coefficients of the line function evaluation.
    pub fn c0c3c4(&self) -> (BN256Fq2NNField<F>, BN256Fq2NNField<F>, BN256Fq2NNField<F>) {
        (self.c0.clone(), self.c3.clone(), self.c4.clone())
    }

    /// This function conducts the doubling step in the Miller loop for the BN256 curve.
    /// Namely, given `Q` in `E'(Fp2)` and `P` in `E(Fp)`, it computes the line function
    /// together with the resultant point `T=2*Q`. The implementation is based
    /// on the _Algorithm 26_ from https://eprint.iacr.org/2010/354.pdf.
    pub fn doubling_step(
        cs: &mut CS,
        q: &mut BN256SWProjectivePointTwisted<F>,
        p: &mut BN256SWProjectivePoint<F>,
    ) -> Self
    where
        CS: ConstraintSystem<F>,
    {
        // 1. tmp0 <- X_Q^2; 2. tmp1 <- Y_Q^2; 3. tmp2 <- tmp1^2;
        let mut tmp0 = q.x.square(cs);
        tmp0.normalize(cs);
        let mut tmp1 = q.y.square(cs);
        tmp1.normalize(cs);
        let mut tmp2 = tmp1.square(cs);
        tmp2.normalize(cs);

        // 4. tmp3 <- (tmp1 + X_Q)^2 - tmp0 - tmp2; 5. tmp3 <- 2*tmp3;
        let mut tmp3 = tmp1.add(cs, &mut q.x);
        tmp3.normalize(cs);
        let mut tmp3 = tmp3.square(cs);
        tmp3.normalize(cs);
        let mut tmp3 = tmp3.sub(cs, &mut tmp0);
        let mut tmp3 = tmp3.sub(cs, &mut tmp2);
        tmp3.normalize(cs);
        let mut tmp3 = tmp3.double(cs);
        tmp3.normalize(cs);

        // 6. tmp4 <- 3*tmp0; 7. tmp6 <- X_Q + tmp4;
        let mut tmp4 = tmp0.double(cs);
        tmp4.normalize(cs);
        let mut tmp4 = tmp4.add(cs, &mut tmp0);
        tmp4.normalize(cs);
        let mut tmp6 = q.x.add(cs, &mut tmp4);
        tmp6.normalize(cs);

        // 8. tmp5 <- tmp4^2; 9. X_T <- tmp5 - 2*tmp3;
        tmp4.normalize(cs);
        let mut tmp5 = tmp4.square(cs);
        tmp5.normalize(cs);
        let mut tmp3_double = tmp3.double(cs);
        tmp3_double.normalize(cs);
        let mut x_t = tmp5.sub(cs, &mut tmp3_double);
        x_t.normalize(cs);

        // Saving Z_Q^2 for later use
        let mut z_q_square = q.z.square(cs);
        z_q_square.normalize(cs);

        // 10. Z_T <- (Y_Q + Z_Q)^2 - tmp1 - Z_Q^2;
        let mut z_t = q.y.add(cs, &mut q.z);
        z_t.normalize(cs);
        let mut z_t = z_t.square(cs);
        z_t.normalize(cs);
        let mut z_t = z_t.sub(cs, &mut tmp1);
        let mut z_t = z_t.sub(cs, &mut z_q_square);
        z_t.normalize(cs);

        // 11. Y_T <- (tmp3 - X_T)*tmp4 - 8*tmp2;
        let mut y_t = tmp3.sub(cs, &mut x_t);
        tmp4.normalize(cs);
        let mut y_t = y_t.mul(cs, &mut tmp4);
        y_t.normalize(cs);
        let mut tmp2_8 = tmp2.double(cs);
        let mut tmp2_8 = tmp2_8.double(cs);
        tmp2_8.normalize(cs);
        let mut tmp2_8 = tmp2_8.double(cs);
        tmp2_8.normalize(cs);
        let mut y_t = y_t.sub(cs, &mut tmp2_8);
        y_t.normalize(cs);

        // 12. tmp3 <- -2*(tmp4 * Z_Q^2); 13. tmp3 <- tmp3 * xP;
        z_q_square.normalize(cs);
        let mut tmp3 = tmp4.mul(cs, &mut z_q_square);
        tmp3.normalize(cs);
        let mut tmp3 = tmp3.double(cs);
        tmp3.normalize(cs);
        let mut tmp3 = tmp3.negated(cs);
        tmp3.normalize(cs);
        let mut tmp3 = tmp3.mul_c0(cs, &mut p.x);
        tmp3.normalize(cs);

        // 14. tmp6 <- tmp6^2 - tmp0 - tmp5 - 4*tmp1; 15. tmp0 <- 2*Z_T*Z_Q^2
        let mut tmp6 = tmp6.square(cs);
        tmp6.normalize(cs);
        let mut tmp6 = tmp6.sub(cs, &mut tmp0);
        let mut tmp6 = tmp6.sub(cs, &mut tmp5);
        tmp6.normalize(cs);
        let mut tmp1_4 = tmp1.double(cs);
        tmp1_4.normalize(cs);
        let mut tmp1_4 = tmp1_4.double(cs);
        tmp1_4.normalize(cs);
        let mut tmp6 = tmp6.sub(cs, &mut tmp1_4);
        tmp6.normalize(cs);

        z_t.normalize(cs);
        let mut tmp0 = z_t.mul(cs, &mut z_q_square);
        tmp0.normalize(cs);
        let mut tmp0 = tmp0.double(cs);

        // 16. tmp0 <- tmp0 * y_P
        let mut tmp0 = tmp0.mul_c0(cs, &mut p.y);
        tmp0.normalize(cs);

        // Result: T = (X_T, Y_T, Z_T); Line function is a0 + a1*w
        // where a0 = tmp0; a1 = tmp3 + tmp6*v;

        Self {
            c0: tmp0,
            c3: tmp3,
            c4: tmp6,
            point: BN256SWProjectivePointTwisted {
                x: x_t,
                y: y_t,
                z: z_t,
                _marker: std::marker::PhantomData,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// This function conducts the addition step in the Miller loop for the BN256 curve.
    /// Namely, given `Q` and `R` in `E'(Fp2)` and `P` in `E(Fp)`, it computes the line function
    /// together with the resultant point `T=Q+R`. The implementation is based
    /// on the _Algorithm 27_ from https://eprint.iacr.org/2010/354.pdf.
    pub fn addition_step(
        cs: &mut CS,
        q: &mut BN256SWProjectivePointTwisted<F>,
        r: &mut BN256SWProjectivePointTwisted<F>,
        p: &mut BN256SWProjectivePoint<F>,
    ) -> Self
    where
        CS: ConstraintSystem<F>,
    {
        // Preparing some temporary variables
        let mut z_r_square = r.z.square(cs);
        z_r_square.normalize(cs);
        let mut y_q_square = q.y.square(cs);
        y_q_square.normalize(cs);

        // 1. t0 <- X_Q*Z_R^2; 2. t1 <- (Y_Q + Z_R)^2 - Y_Q^2 - Z_R^2;
        let mut t0 = q.x.mul(cs, &mut z_r_square);
        t0.normalize(cs);

        let mut t1 = q.y.add(cs, &mut r.z);
        t1.normalize(cs);
        let mut t1 = t1.square(cs);
        t1.normalize(cs);
        let mut t1 = t1.sub(cs, &mut y_q_square);
        let mut t1 = t1.sub(cs, &mut z_r_square);
        t1.normalize(cs);

        // 3. t1 <- t1 * Z_R^2; 4. t2 <- t0 - X_R; 5. t3 <- t2^2;
        let mut t1 = t1.mul(cs, &mut z_r_square);
        t1.normalize(cs);
        let mut t2 = t0.sub(cs, &mut r.x);
        let mut t3 = t2.square(cs);
        t3.normalize(cs);

        // 6. t4 <- 4*t3; 7. t5 <- t4*t2; 8. t6 <- t1 - 2*Y_R;
        let mut t4 = t3.double(cs);
        t4.normalize(cs);
        let mut t4 = t4.double(cs);
        t4.normalize(cs);
        let mut t5 = t4.mul(cs, &mut t2);
        t5.normalize(cs);
        let mut y_r_2 = r.y.double(cs);
        let mut t6 = t1.sub(cs, &mut y_r_2);
        t6.normalize(cs);

        // 9. t9 <- t6 * X_Q; 10. t7 <- X_R * t4; 11. X_T <- t6^2 - t5 - 2t7
        let mut t9 = t6.mul(cs, &mut q.x);
        t6.normalize(cs);
        let mut t7 = r.x.mul(cs, &mut t4);
        t7.normalize(cs);
        let mut x_t = t6.square(cs);
        x_t.normalize(cs);
        let mut x_t = x_t.sub(cs, &mut t5);
        let mut t7_2 = t7.double(cs);
        let mut x_t = x_t.sub(cs, &mut t7_2);
        x_t.normalize(cs);

        // 12. Z_T <- (Z_R + t2)^2 - Z_R^2 - t3;
        let mut z_t = r.z.add(cs, &mut t2);
        let mut z_t = z_t.square(cs);
        z_t.normalize(cs);
        let mut z_t = z_t.sub(cs, &mut z_r_square);
        let mut z_t = z_t.sub(cs, &mut t3);
        z_t.normalize(cs);

        // 13. t10 <- Y_Q + Z_T; 14. t8 <- (t7 - X_T)*t6;
        let mut t10 = q.y.add(cs, &mut z_t);
        let mut t8 = t7.sub(cs, &mut x_t);
        t8.normalize(cs);
        let mut t8 = t8.mul(cs, &mut t6);
        t8.normalize(cs);

        // 15. t0 <- 2*Y_R*t5; 16. Y_T <- t8 - t0; 17. t10 <- t10^2 - Y_Q^2 - Z_T^2;
        let mut t0 = y_r_2.mul(cs, &mut t5);
        t0.normalize(cs);
        let y_t = t8.sub(cs, &mut t0);
        let mut t10 = t10.square(cs);
        t10.normalize(cs);
        let mut t10 = t10.sub(cs, &mut y_q_square);
        let mut z_t_square = z_t.square(cs);
        z_t_square.normalize(cs);
        let mut t10 = t10.sub(cs, &mut z_t_square);

        // 18. t9 <- 2*t9 - t10; 19. t10 <- 2*Z_T*y_P;
        let mut t9 = t9.double(cs);
        t9.normalize(cs);
        let t9 = t9.sub(cs, &mut t10);
        let mut t10 = z_t.mul_c0(cs, &mut p.y);
        t10.normalize(cs);
        let mut t10 = t10.double(cs);
        t10.normalize(cs);

        // 20. t6 <- -t6; 21. t1 <- 2*t6*x_P;
        let mut t6 = t6.negated(cs);
        t6.normalize(cs);
        let mut t1 = t6.mul_c0(cs, &mut p.x);
        t1.normalize(cs);
        let mut t1 = t1.double(cs);
        t1.normalize(cs);

        // Result: T = (X_T, Y_T, Z_T); Line function is l0 + l1*w
        // where l0 = t10; l1 = t1 + t9*v;

        Self {
            c0: t10,
            c3: t1,
            c4: t9,
            point: BN256SWProjectivePointTwisted {
                x: x_t,
                y: y_t,
                z: z_t,
                _marker: std::marker::PhantomData,
            },
            _marker: std::marker::PhantomData,
        }
    }
}

/// Struct for the miller loop evaluation for the BN256 curve.
/// Here, the Miller loop returns the accumulated f value after the loop
/// without the final exponentiation.
pub struct MillerLoopEvaluation<F, CS>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    accumulated_f: BN256Fq12NNField<F>,
    _marker: std::marker::PhantomData<CS>,
}

impl<F, CS> MillerLoopEvaluation<F, CS>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    pub fn get_accumulated_f(&self) -> BN256Fq12NNField<F> {
        self.accumulated_f.clone()
    }

    /// This function computes the Miller loop for the BN256 curve, using
    /// _Algorithm 1_ from https://eprint.iacr.org/2010/354.pdf. Frobenius
    /// map is taken from https://hackmd.io/@Wimet/ry7z1Xj-2.
    pub fn evaluate(
        cs: &mut CS,
        p: &mut BN256SWProjectivePoint<F>,
        q: &mut BN256SWProjectivePointTwisted<F>,
    ) -> Self {
        // Verifying that q is normalized
        let q_is_normalized = q.is_normalized(cs);
        let boolean_true = Boolean::allocated_constant(cs, true);
        Boolean::enforce_equal(cs, &q_is_normalized, &boolean_true);

        // Setting evaluation parameters
        let mut t = q.clone();
        let params = p.x.params.clone();
        let mut f = BN256Fq12NNField::one(cs, &params);

        // Saving Q negative to avoid doing that in the loop
        let mut q_negated = q.negated(cs);

        // Main loop
        for i in (1..SIX_U_PLUS_TWO_WNAF.len()).rev() {
            // Doubling step: f <- f^2 * L_{R,R}(P), T <- 2*T
            // Evaluation of L_{R,R} and 2R is done in the same step
            if i != SIX_U_PLUS_TWO_WNAF.len() - 1 {
                f = f.square(cs);
                f.normalize(cs);
            }

            let mut doubling = LineFunctionEvaluation::doubling_step(cs, &mut t, p);
            doubling.point.x.normalize(cs);
            doubling.point.y.normalize(cs);
            doubling.point.z.normalize(cs);
            f = Self::mul_f12_by_line_fn(cs, &mut f, &mut doubling);
            f.normalize(cs);
            t = doubling.point;

            let x = SIX_U_PLUS_TWO_WNAF[i - 1];
            match x {
                1 => {
                    // Addition step: f <- f * L_{T,Q}(P), T <- T + Q
                    let mut addition = LineFunctionEvaluation::addition_step(cs, q, &mut t, p);
                    addition.point.x.normalize(cs);
                    addition.point.y.normalize(cs);
                    addition.point.z.normalize(cs);
                    f = Self::mul_f12_by_line_fn(cs, &mut f, &mut addition);
                    f.normalize(cs);
                    t = addition.point;
                }
                -1 => {
                    // Addition step: f <- f * L_{T,-Q}(P), T <- T - Q
                    let mut addition =
                        LineFunctionEvaluation::addition_step(cs, &mut q_negated, &mut t, p);
                    addition.point.x.normalize(cs);
                    addition.point.y.normalize(cs);
                    addition.point.z.normalize(cs);
                    f = Self::mul_f12_by_line_fn(cs, &mut f, &mut addition);
                    f.normalize(cs);
                    t = addition.point;
                }
                _ => continue,
            }
        }

        // Some additional steps to finalize the Miller loop...
        // Preparing some constants for the Frobenius operator
        let mut q1_mul_factor = Self::allocate_fq2_constant(cs, FROBENIUS_COEFF_FQ6_C1[1], &params);
        let mut q2_mul_factor = Self::allocate_fq2_constant(cs, FROBENIUS_COEFF_FQ6_C1[2], &params);
        let mut xi_to_q_minus_1_over_2 =
            Self::allocate_fq2_constant(cs, XI_TO_Q_MINUS_1_OVER_2, &params);

        // Calculating Frobenius operator Q1 = pi_p(Q)
        let mut q1 = q.clone();
        q1.x = q1.x.conjugate(cs);
        q1.x = q1.x.mul(cs, &mut q1_mul_factor);
        q1.x.normalize(cs);

        q1.y = q1.y.conjugate(cs);
        q1.y = q1.y.mul(cs, &mut xi_to_q_minus_1_over_2);
        q1.y.normalize(cs);

        // Calculating Frobenius operator Q2 = -pi_p^2(Q)
        let mut q2 = q.clone();
        q2.x = q2.x.mul(cs, &mut q2_mul_factor);
        q2.x.normalize(cs);

        // Calculating addition step for T, Q1, f <- f * (line function), T <- T + Q1
        let mut addition = LineFunctionEvaluation::addition_step(cs, &mut q1, &mut t, p);
        f = Self::mul_f12_by_line_fn(cs, &mut f, &mut addition);
        f.normalize(cs);
        t = addition.point;

        // Calculating addition step for T, -Q2, f <- f * (line function), T <- T - Q2
        let mut addition = LineFunctionEvaluation::addition_step(cs, &mut q2, &mut t, p);
        f = Self::mul_f12_by_line_fn(cs, &mut f, &mut addition);
        f.normalize(cs);

        Self {
            accumulated_f: f,
            _marker: std::marker::PhantomData::<CS>,
        }
    }

    fn mul_f12_by_line_fn(
        cs: &mut CS,
        f: &mut BN256Fq12NNField<F>,
        line_fn: &mut LineFunctionEvaluation<F, CS>,
    ) -> BN256Fq12NNField<F> {
        let mut f = f.mul_by_c0c3c4(cs, &mut line_fn.c0, &mut line_fn.c3, &mut line_fn.c4);
        f.normalize(cs);
        f
    }

    /// Allocates the constant from `Fq2` constant
    pub fn allocate_fq2_constant(
        cs: &mut CS,
        value: Fq2,
        params: &Arc<BN256BaseNNFieldParams>,
    ) -> BN256Fq2NNField<F> {
        let c0 = BN256BaseNNField::allocated_constant(cs, value.c0, params);
        let c1 = BN256BaseNNField::allocated_constant(cs, value.c1, params);

        BN256Fq2NNField::new(c0, c1)
    }
}

pub struct FinalExpEvaluation<F, CS>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    resultant_f: BN256Fq12NNField<F>,
    _marker: std::marker::PhantomData<CS>,
}

impl<F, CS> FinalExpEvaluation<F, CS>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    /// This function computes the final exponentiation for the BN256 curve
    /// without using the Torus (`T2`) compression technique.
    ///
    /// The final exponentiation is partially based on _Algorithm 31_ from
    /// https://eprint.iacr.org/2010/354.pdf, but mainly based on implementation
    /// from pairing repository https://github.com/matter-labs/pairing.
    pub fn evaluate_without_torus(cs: &mut CS, r: &mut BN256Fq12NNField<F>) -> Self {
        // TODO: Avoid too many normalizations
        // Preparing a curve parameter
        let u = CURVE_U_PARAMETER;

        // 1-6. Calculate the easy part.
        let mut r = Self::easy_part(cs, r);

        // 7-9. fpk <- f^p^k, k = 1, 2, 3
        let mut fp = r.frobenius_map(cs, 1);
        fp.normalize(cs);
        let mut fp2 = r.frobenius_map(cs, 2);
        fp2.normalize(cs);
        let mut fp3 = fp2.frobenius_map(cs, 1);
        fp3.normalize(cs);

        // 10-12. fuk <- f^u^k, k = 1, 2, 3
        let mut fu = r.pow_u32(cs, &[u]);
        fu.normalize(cs);
        let mut fu2 = fu.pow_u32(cs, &[u]);
        fu2.normalize(cs);
        let mut fu3 = fu2.pow_u32(cs, &[u]);
        fu3.normalize(cs);

        // 13. y3 <- fu^p; 14. fu2p <- fu2^p; 15. fu3p <- fu3^p; 16. y2 <- fu2^p
        let mut y3 = fu.frobenius_map(cs, 1);
        y3.normalize(cs);
        let mut fu2p = fu2.frobenius_map(cs, 1);
        fu2p.normalize(cs);
        let mut fu3p = fu3.frobenius_map(cs, 1);
        fu3p.normalize(cs);
        let mut y2 = fu2.frobenius_map(cs, 2);
        y2.normalize(cs);

        // 17. y0 <- fp*fp2*fp3; 18. y1 <- r^*; 19. y5 <- fu2^*;
        let mut y0 = fp.mul(cs, &mut fp2);
        y0.normalize(cs);
        let mut y0 = y0.mul(cs, &mut fp3);
        y0.normalize(cs);
        let mut y1 = r.conjugate(cs);
        let mut y5 = fu2.conjugate(cs);

        // 20. y3 <- y3^*; 21. y4 <- fu*fu2p; 22. y4 <- y4^*;
        let mut y3 = y3.conjugate(cs);
        let mut y4 = fu.mul(cs, &mut fu2p);
        y4.normalize(cs);
        let mut y4 = y4.conjugate(cs);

        // 23. y6 <- fu3*fu3p; 24. y6 <- y6^*; 25. y6 <- y6^2;
        let mut y6 = fu3.mul(cs, &mut fu3p);
        y6.normalize(cs);
        let mut y6 = y6.conjugate(cs);
        let mut y6 = y6.square(cs);
        y6.normalize(cs);

        // 26. y6 <- y6*y4; 27. y6 <- y6*y5; 28. t1 <- y3*y5;
        let mut y6 = y6.mul(cs, &mut y4);
        y6.normalize(cs);
        let mut y6 = y6.mul(cs, &mut y5);
        y6.normalize(cs);
        let mut t1 = y3.mul(cs, &mut y5);
        t1.normalize(cs);

        // 29. t1 <- t1*y6; 30. y6 <- y6*y2; 31. t1 <- t1^2; 32. t1 <- t1*y6;
        let mut t1 = t1.mul(cs, &mut y6);
        t1.normalize(cs);
        let mut y6 = y6.mul(cs, &mut y2);
        y6.normalize(cs);
        let mut t1 = t1.square(cs);
        t1.normalize(cs);
        let mut t1 = t1.mul(cs, &mut y6);
        t1.normalize(cs);

        // 33. t1 <- t1^2; 34. t1 <- t1*y1; 35. t1 <- t1*y0;
        let mut t1 = t1.square(cs);
        t1.normalize(cs);
        let mut t0 = t1.mul(cs, &mut y1);
        t0.normalize(cs);
        let mut t1 = t1.mul(cs, &mut y0);
        t1.normalize(cs);

        // 36. t0 <- t0^2; 37. t0 <- t0*t1; Return t0
        let mut t0 = t0.square(cs);
        t0.normalize(cs);
        let mut t0 = t0.mul(cs, &mut t1);
        t0.normalize(cs);

        Self {
            resultant_f: t0,
            _marker: std::marker::PhantomData::<CS>,
        }
    }

    /// Calculates the easy part of the exponentiation, that is
    /// `r^((p^(k) - 1) / Phi_k(p))` where
    /// `Phi_{12}(p) = p^4 - p^2 + 1` is a 12th cyclotomic polynomial.
    fn easy_part(cs: &mut CS, r: &mut BN256Fq12NNField<F>) -> BN256Fq12NNField<F> {
        // 1. f1 <- f1^*; 2. f2 <- f^{-1}; 3. f <- f1*f2; 4. f2 <- f
        let mut f1 = r.conjugate(cs);
        let mut f2 = r.inverse(cs);
        f2.normalize(cs);
        let mut r = f1.mul(cs, &mut f2);
        r.normalize(cs);
        let mut f2 = r.clone();

        // 5. f <- f^q^2; 6. f <- f*f2;
        let mut r = r.frobenius_map(cs, 2);
        r.normalize(cs);
        let mut r = r.mul(cs, &mut f2);
        r.normalize(cs);

        r
    }

    /// Calculates the hard part of the exponentiation using torus compression.
    /// In a nutshell, this function conducts the following steps:
    /// 1. Compresses the `Fq12` element after the easy part into the `T2` torus.
    /// 2. Computes the hard part of the exponentiation in the `T2` torus in the same
    /// way as it was done before.
    /// 3. Decompresses the result from the `T2` torus back to the `Fq12` element.
    ///
    /// NOTE: The last step is actually not needed for checks in a form
    /// `e(P1,Q1)e(P2,Q2)...e(Pn,Qn) = 1` later (that is, the ecpairing precompile),
    /// but for now we stick to the easier-to-implement version.
    fn hard_part_torus(cs: &mut CS, r: &mut BN256Fq12NNField<F>) -> BN256Fq12NNField<F> {
        // TODO: Avoid too many normalizations
        // Preparing a curve parameter
        let u = U_WNAF;

        // Creating a wrapper around the r
        let mut torus = TorusWrapper::compress::<_, true>(cs, r);

        // 7-9. fpk <- f^p^k, k = 1, 2, 3
        let mut fp = torus.frobenius_map(cs, 1);
        fp.normalize(cs);
        let mut fp2 = torus.frobenius_map(cs, 2);
        fp2.normalize(cs);
        let mut fp3 = fp2.frobenius_map(cs, 1);
        fp3.normalize(cs);

        // 10-12. fuk <- f^u^k, k = 1, 2, 3
        let mut fu = torus.pow_naf_decomposition::<_, _, true>(cs, &u);
        fu.normalize(cs);
        let mut fu2 = fu.pow_naf_decomposition::<_, _, true>(cs, &u);
        fu2.normalize(cs);
        let mut fu3 = fu2.pow_naf_decomposition::<_, _, true>(cs, &u);
        fu3.normalize(cs);

        // 13. y3 <- fu^p; 14. fu2p <- fu2^p; 15. fu3p <- fu3^p; 16. y2 <- fu2^p
        let mut y3 = fu.frobenius_map(cs, 1);
        y3.normalize(cs);
        let mut fu2p = fu2.frobenius_map(cs, 1);
        fu2p.normalize(cs);
        let mut fu3p = fu3.frobenius_map(cs, 1);
        fu3p.normalize(cs);
        let mut y2 = fu2.frobenius_map(cs, 2);
        y2.normalize(cs);

        // 17. y0 <- fp*fp2*fp3; 18. y1 <- r^*; 19. y5 <- fu2^*;
        let mut y0 = fp.mul::<_, true>(cs, &mut fp2);
        y0.normalize(cs);
        let mut y0 = y0.mul::<_, true>(cs, &mut fp3);
        y0.normalize(cs);
        let mut y1 = torus.conjugate(cs);
        let mut y5 = fu2.conjugate(cs);

        // 20. y3 <- y3^*; 21. y4 <- fu*fu2p; 22. y4 <- y4^*;
        let mut y3 = y3.conjugate(cs);
        let mut y4 = fu.mul::<_, true>(cs, &mut fu2p);
        y4.normalize(cs);
        let mut y4 = y4.conjugate(cs);

        // 23. y6 <- fu3*fu3p; 24. y6 <- y6^*; 25. y6 <- y6^2;
        let mut y6 = fu3.mul::<_, true>(cs, &mut fu3p);
        y6.normalize(cs);
        let mut y6 = y6.conjugate(cs);
        let mut y6 = y6.square::<_, true>(cs);
        y6.normalize(cs);

        // 26. y6 <- y6*y4; 27. y6 <- y6*y5; 28. t1 <- y3*y5;
        let mut y6 = y6.mul::<_, true>(cs, &mut y4);
        y6.normalize(cs);
        let mut y6 = y6.mul::<_, true>(cs, &mut y5);
        y6.normalize(cs);
        let mut t1 = y3.mul::<_, true>(cs, &mut y5);
        t1.normalize(cs);

        // 29. t1 <- t1*y6; 30. y6 <- y6*y2; 31. t1 <- t1^2; 32. t1 <- t1*y6;
        let mut t1 = t1.mul::<_, true>(cs, &mut y6);
        t1.normalize(cs);
        let mut y6 = y6.mul::<_, true>(cs, &mut y2);
        y6.normalize(cs);
        let mut t1 = t1.square::<_, true>(cs);
        t1.normalize(cs);
        let mut t1 = t1.mul::<_, true>(cs, &mut y6);
        t1.normalize(cs);

        // 33. t1 <- t1^2; 34. t1 <- t1*y1; 35. t1 <- t1*y0;
        let mut t1 = t1.square::<_, true>(cs);
        t1.normalize(cs);
        let mut t0 = t1.mul::<_, true>(cs, &mut y1);
        t0.normalize(cs);
        let mut t1 = t1.mul::<_, true>(cs, &mut y0);
        t1.normalize(cs);

        // 36. t0 <- t0^2; 37. t0 <- t0*t1; Return t0
        let mut t0 = t0.square::<_, true>(cs);
        t0.normalize(cs);
        let mut t0 = t0.mul::<_, true>(cs, &mut t1);
        t0.normalize(cs);

        t0.decompress(cs)
    }

    /// This function computes the final exponentiation for the BN256 curve using the Torus
    /// compression technique. It firstly computes the easy part as usual, then compresses
    /// the result into the `T2` torus, computes the hard part in the `T2` torus, and finally
    /// decompresses the result back to the `Fq12` element.
    pub fn evaluate_torus(cs: &mut CS, r: &mut BN256Fq12NNField<F>) -> Self {
        let mut easy = Self::easy_part(cs, r);
        let hard = Self::hard_part_torus(cs, &mut easy);

        Self {
            resultant_f: hard,
            _marker: std::marker::PhantomData::<CS>,
        }
    }

    /// Returns the accumulated `f` value after the final exponentiation.
    pub fn get(&self) -> BN256Fq12NNField<F> {
        self.resultant_f.clone()
    }
}

/// This function computes the pairing function for the BN256 curve.
pub fn ec_pairing<F, CS>(
    cs: &mut CS,
    p: &mut BN256SWProjectivePoint<F>,
    q: &mut BN256SWProjectivePointTwisted<F>,
) -> BN256Fq12NNField<F>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    p.enforce_reduced(cs);
    q.enforce_reduced(cs);

    let mut miller_loop = MillerLoopEvaluation::evaluate(cs, p, q);
    let final_exp = FinalExpEvaluation::evaluate_without_torus(cs, &mut miller_loop.accumulated_f);
    final_exp.resultant_f
}

/// This function computes the pairing function for the BN256 curve using
/// the Torus compression technique. This implementation is faster than
/// the regular one and requires less constraints.
pub fn ec_pairing_torus<F, CS>(
    cs: &mut CS,
    p: &mut BN256SWProjectivePoint<F>,
    q: &mut BN256SWProjectivePointTwisted<F>,
) -> BN256Fq12NNField<F>
where
    F: SmallField,
    CS: ConstraintSystem<F>,
{
    p.enforce_reduced(cs);
    q.enforce_reduced(cs);

    let mut miller_loop = MillerLoopEvaluation::evaluate(cs, p, q);
    let final_exp = FinalExpEvaluation::evaluate_torus(cs, &mut miller_loop.accumulated_f);
    final_exp.resultant_f
}
