import json

# Defining the base prime field
q = Integer(21888242871839275222246405745257275088696311157297823662689037894645226208583) # EC group order
Fq = GF(q) 

# r is taken from https://hackmd.io/@jpw/bn254
k = Integer(12) # Embedding degree
t = Integer(4965661367192848881)
r = Integer(21888242871839275222246405745257275088548364400416034343698204186575808495617)
e = (q^(12)-1)/r

# Making sure parameters are correctly defined
# See https://eprint.iacr.org/2010/354.pdf, Equation 1 for details.
assert q == 36*t**4 + 36*t**3 + 24*t**2 + 6*t + 1
assert r == 36*t**4 + 36*t**3 + 18*t**2 + 6*t + 1

# Defining the extensions
# Fq2...
K2.<x> = PolynomialRing(Fq)
Fq2.<u> = Fq.extension(x^2+1)

# Fq6...
K6.<y> = PolynomialRing(Fq2)
Fq6.<v> = Fq2.extension(y^3 - (u+9))

# Defining the Fq12 is a bit more tricky...
p = Fq.characteristic()
Fq12.<G> = GF(p^12)

i = sqrt(Fq12(-1))
R12.<Y> = PolynomialRing(Fq12)

j = (Y^3 - (i+9)).roots(multiplicities=False)[0]
w = sqrt(j)

P = w.minpoly()
Fq12.<W> = GF(p^12, modulus=P)

# Preparing helper debugging lambda functions
fq2_to_dictionary = lambda f : {
    'c0': str(f[0]), 
    'c1': str(f[1])
}
fq6_to_dictionary = lambda f : {
    'c0': {
        'c0': str(f[0][0]), 
        'c1': str(f[0][1])
    }, 
    'c1': {
        'c0': str(f[1][0]), 
        'c1': str(f[1][1])
    },
    'c2': {
        'c0': str(f[2][0]), 
        'c1': str(f[2][1])
    }
}
fq12_to_dictionary = lambda f: {
    'c0': { # Fq6
        'c0': { #Fq2
            'c0': str(f[0]+9*f[6]),
            'c1': str(f[6]),
        },
        'c1': { #Fq2
            'c0': str(f[2]+9*f[8]),
            'c1': str(f[8]),
        },
        'c2': { #Fq2
            'c0': str(f[4]+9*f[10]),
            'c1': str(f[10]),
        }
    }, 
    'c1': { # Fq6
        'c0': { #Fq2
            'c0': str(f[1]+9*f[7]),
            'c1': str(f[7]),
        },
        'c1': { #Fq2
            'c0': str(f[3]+9*f[9]),
            'c1': str(f[9]),
        },
        'c2': { #Fq2
            'c0': str(f[5]+9*f[11]),
            'c1': str(f[11]),
        }
    }
}

# Defining the G1 Curve
G1 = EllipticCurve(Fq, [0, 3])

# Defining the G2 Curve
b = 3 / (u + 9)
G2 = EllipticCurve(Fq2, [0, b])

# Helper debugging functions
g1_point_to_dictionary = lambda point : {
    'x': str(point[0]),
    'y': str(point[1])
}
g2_point_to_dictionary = lambda point : {
    'x': {
        'c0': str(point[0][0]), 
        'c1': str(point[0][1])
    }, 
    'y': {
        'c0': str(point[1][0]), 
        'c1': str(point[1][1])
    }
}

# --- Line functions tests ---

def doubling_step(Q, P):
    X_Q, Y_Q, Z_Q = copy(Q[0]), copy(Q[1]), copy(Q[2])
    x_P, y_P = copy(P[0]), copy(P[1])

    tmp0 = X_Q**2
    tmp1 = Y_Q**2
    tmp2 = tmp1^2
    tmp3 = (tmp1 + X_Q)^2 - tmp0 - tmp2
    tmp3 = 2*tmp3
    tmp4 = 3*tmp0
    tmp6 = X_Q + tmp4
    tmp5 = tmp4^2
    X_T = tmp5 - 2*tmp3
    Z_T = (Y_Q + Z_Q)^2 - tmp1 - Z_Q^2
    Y_T = (tmp3 - X_T)*tmp4 - 8*tmp2
    tmp3 = -2*tmp4*Z_Q^2
    tmp3 = tmp3*x_P
    tmp6 = tmp6^2 - tmp0 - tmp5 - 4*tmp1
    tmp0 = 2*Z_T*Z_Q^2
    tmp0 = tmp0 * y_P
    
    T = G2((X_T / Z_T^2, Y_T / Z_T^3))
    return (tmp0, tmp3, tmp6), T

def addition_step(Q, R, P):
    X_Q, Y_Q, Z_Q = copy(Q[0]), copy(Q[1]), copy(Q[2])
    X_R, Y_R, Z_R = copy(R[0]), copy(R[1]), copy(R[2])
    x_P, y_P = copy(P[0]), copy(P[1])

    t0 = X_Q * Z_R^2
    t1 = (Y_Q + Z_R)^2 - Y_Q^2 - Z_R^2
    t1 = t1 * Z_R^2
    t2 = t0 - X_R
    t3 = t2^2 
    t4 = 4*t3
    t5 = t4 * t2
    t6 = t1 - 2*Y_R
    t9 = t6 * X_Q
    t7 = X_R*t4
    X_T = t6^2 - t5 - 2*t7
    Z_T = (Z_R + t2)^2 - Z_R^2 - t3
    t10 = Y_Q + Z_T
    t8 = (t7 - X_T)*t6
    t0 = 2*Y_R*t5
    Y_T = t8 - t0
    t10 = t10^2 - Y_Q^2 - Z_T^2
    t9 = 2*t9 - t10
    t10 = 2*Z_T*y_P
    t6 = -t6
    t1 = 2*t6*x_P

    T = G2((X_T / Z_T^2, Y_T / Z_T^3))
    return (t10, t1, t9), T

LINE_FUNCTIONS_TESTS_NUMBER = 2

print('Preparing the line functions tests...')
tests_dict = {'tests': []}

for _ in range(LINE_FUNCTIONS_TESTS_NUMBER):
    # Generating two random points
    Q = G2.random_point()
    R = G2.random_point()
    P = G1.random_point()
    
    (c0_1, c3_1, c4_1), T1 = doubling_step(Q, P)
    (c0_2, c3_2, c4_2), T2 = doubling_step(R, P)
    (c0_3, c3_3, c4_3), T3 = addition_step(Q, R, P)

    assert T1 == 2*Q, 'Doubling step 1 failed!'
    assert T2 == 2*R, 'Doubling step 2 failed!'
    assert T3 == Q+R, 'Addition step failed!'

    # Adding the test to the dictionary
    tests_dict['tests'].append({
        'g2_point_1': g2_point_to_dictionary(Q),
        'g2_point_2': g2_point_to_dictionary(R),
        'g1_point': g1_point_to_dictionary(P),
        'expected': {
            'doubling_1': {
                'point': g2_point_to_dictionary(2*Q),
                'c0': fq2_to_dictionary(c0_1),
                'c3': fq2_to_dictionary(c3_1),
                'c4': fq2_to_dictionary(c4_1)
            },
            'doubling_2': {
                'point': g2_point_to_dictionary(2*R),
                'c0': fq2_to_dictionary(c0_2),
                'c3': fq2_to_dictionary(c3_2),
                'c4': fq2_to_dictionary(c4_2)
            },
            'addition': {
                'point': g2_point_to_dictionary(Q+R),
                'c0': fq2_to_dictionary(c0_3),
                'c3': fq2_to_dictionary(c3_3),
                'c4': fq2_to_dictionary(c4_3)
            }
        }
    })

print('Line and tangent functions evaluations completed!')

# Saving the json file
FILE_NAME = '../json/ec_pairing/line_functions_tests.json'

print(f'Saving the line function tests to {FILE_NAME}...')

with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

# --- Easy exponentiation tests ---
EXPONENTIATION_TESTS_NUMBER = 2

print('Preparing the final exponentiation tests...')

tests_dict = {'tests': []}

for _ in range(EXPONENTIATION_TESTS_NUMBER):
    # Generating random value
    f = Fq12.random_element()
    f_exp = f^e
    assert f_exp**r == 1

    # Reference implementation of the final exponentiation
    def final_exp(r: Fq12) -> Fq12:
        f1 = copy(r)
        f1 = f1.conjugate()
        f2 = r.inverse()
        r = copy(f1)
        r = r * f2 
        f2 = copy(r)
        r = r**(q**2)
        r = r * f2
        x = copy(t)
        fp = copy(r)
        fp = fp**(q)
        fp2 = copy(r)
        fp2 = fp2**(q**2)
        fp3 = copy(fp2)
        fp3 = fp3**(q)
        fu = copy(r)
        fu = fu**x
        fu2 = copy(fu)
        fu2 = fu2**x
        fu3 = copy(fu2)
        fu3 = fu3**x
        y3 = copy(fu)
        y3 = y3**q
        fu2p = copy(fu2)
        fu2p = fu2p**q
        fu3p = copy(fu3)
        fu3p = fu3p**q
        y2 = copy(fu2)
        y2 = y2**(q**2)
        y0 = copy(fp)
        y0 = y0 * fp2
        y0 = y0 * fp3
        y1 = copy(r)
        y1 = y1.conjugate()
        y5 = copy(fu2)
        y5 = y5.conjugate()
        y3 = y3.conjugate()
        y4 = copy(fu)
        y4 = y4 * fu2p
        y4 = y4.conjugate()
        y6 = copy(fu3)
        y6 = y6*fu3p
        y6 = y6.conjugate()
        y6 = y6**2
        y6 = y6 * y4
        y6 = y6 * y5
        t1 = copy(y3)
        t1 = t1 * y5
        t1 = t1 * y6
        y6 = y6 * y2
        t1 = t1**2
        t1 = t1 * y6
        t1 = t1**2 
        t0 = copy(t1)
        t0 = t0 * y1
        t1 = t1 * y0
        t0 = t0**2
        t0 = t0 * t1
        return t0
    
    assert final_exp(f) == f_exp

    tests_dict['tests'].append({
        'scalar': fq12_to_dictionary(f),
        'expected': fq12_to_dictionary(f_exp)
    })

print('Final exponentiation tests formed successfully!')

# Saving the json file
FILE_NAME = '../json/ec_pairing/final_exp_tests.json'

print(f'Saving the final exponentiation tests to {FILE_NAME}...')

with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

# --- Pairing tests ---
PAIRING_TESTS_NUMBER = 2

print('Preparing the pairing tests...')

tests_dict = {'tests': []}

for _ in range(PAIRING_TESTS_NUMBER):
    # Generating random value
    
    P = G1.random_point()
    Q = G2.random_point()
    a = 10
    b = 40

    def miller_loop(p: G1, q: G2):
        SIX_U_PLUS_TWO_WNAF = [
            0, 0, 0, 1, 0, 1, 0, -1,
            0, 0, 1, -1, 0, 0, 1, 0,
            0, 1, 1, 0, -1, 0, 0, 1, 
            0, -1, 0, 0, 0, 0, 1, 1,
            1, 0, 0, -1, 0, 0, 1, 0, 
            0, 0, 0, 0, -1, 0, 0, 1,
            1, 0, 0, -1, 0, 0, 0, 1, 
            1, 0, -1, 0, 0, 1, 0, 1, 
            1
        ]
        assert len(SIX_U_PLUS_TWO_WNAF) == 65, 'SIX_U_PLUS_TWO_WNAF is probably wrong'
        assert sum([SIX_U_PLUS_TWO_WNAF[i]*2**i for i in range(len(SIX_U_PLUS_TWO_WNAF))]) == 6*4965661367192848881+2, 'SIX_U_PLUS_TWO_WNAF is probably wrong'

        f = Fq12.one()
        t = copy(q)
        q_neg = -copy(q)

        def c0c3c4_to_fq12(c0: Fq2, c3: Fq2, c4: Fq2) -> Fq12:
            return c0[0] + c0[1]*(W^6-9) + (c3[0]+c3[1]*(W^6-9))*W + (c4[0]+c4[1]*(W^6-9))*W^3
        
        def ell(f: Fq12, coeffs: tuple[Fq2, Fq2, Fq2], p: G1) -> Fq12:
            px, py = copy(p[0]), copy(p[1])
            
            c0 = coeffs[0]
            c1 = coeffs[1]

            c0_c0, c0_c1 = copy(c0[0]), copy(c0[1])
            c0_c0 = c0_c0 * py
            c0_c1 = c0_c1 * py
            c0 = c0_c0 + c0_c1*u

            c1_c0, c1_c1 = copy(c1[0]), copy(c1[1])
            c1_c0 = c1_c0 * px
            c1_c1 = c1_c1 * px
            c1 = c1_c0 + c1_c1*u

            return f * c0c3c4_to_fq12(c0, c1, coeffs[2])

        # For i = L-2 down to 0...
        for i in reversed(range(1, len(SIX_U_PLUS_TWO_WNAF))):
            if i != len(SIX_U_PLUS_TWO_WNAF) - 1:
                f = f*f
            
            (c0, c3, c4), t2 = doubling_step(t, p)
            assert t2 == 2*t
            c0c3c4 = c0c3c4_to_fq12(c0, c3, c4)
            f = ell(f, (c0, c3, c4), p)
            t = t2

            x = SIX_U_PLUS_TWO_WNAF[i-1]

            if x == 0:
                continue
            
            q1 = copy(q)
            if x == -1:
                q1 = q_neg
            
            (c0, c3, c4), tq1 = addition_step(t, q1, p)
            assert tq1 == t + q1
            c0c3c4 = c0c3c4_to_fq12(c0, c3, c4)
            f = ell(f, (c0, c3, c4), p)
            t = tq1

        # Some additional steps to finalize the Miller loop...
        qq = 21888242871839275222246405745257275088696311157297823662689037894645226208583
        
        frobenius_coeff_fq6_c1_c0 = 0xb5773b104563ab30 | 0x347f91c8a9aa6454 << 64 | 0x7a007127242e0991 << 128 | 0x1956bcd8118214ec << 192
        frobenius_coeff_fq6_c1_c0 = Fq(frobenius_coeff_fq6_c1_c0) * Fq(2^(-256))
        frobenius_coeff_fq6_c1_c1 = 0x6e849f1ea0aa4757 | 0xaa1c7b6d89f89141 << 64 | 0xb6e713cdfae0ca3a << 128 | 0x26694fbb4e82ebc3 << 192
        frobenius_coeff_fq6_c1_c1 = Fq(frobenius_coeff_fq6_c1_c1) * Fq(2^(-256))
        q1_mul_factor = frobenius_coeff_fq6_c1_c0 + frobenius_coeff_fq6_c1_c1*u
        expected = (u + 9)**(((qq^1) - 1) / 3)
        assert q1_mul_factor == expected, 'q1_mul_factor is wrong!'

        # Some additional steps to finalize the Miller loop...
        frobenius_coeff_fq6_c1_c0 = 0x3350c88e13e80b9c | 0x7dce557cdb5e56b9 << 64 | 0x6001b4b8b615564a << 128 | 0x2682e617020217e0 << 192
        frobenius_coeff_fq6_c1_c0 = Fq(frobenius_coeff_fq6_c1_c0) * Fq(2^(-256))
        frobenius_coeff_fq6_c1_c1 = 0x0 | 0x0 << 64 | 0x0 << 128 | 0x0 << 192
        frobenius_coeff_fq6_c1_c1 = Fq(frobenius_coeff_fq6_c1_c1) * Fq(2^(-256))
        frobenius_coeff_fq6_c1 = frobenius_coeff_fq6_c1_c0 + frobenius_coeff_fq6_c1_c1*u
        q2_mul_factor = frobenius_coeff_fq6_c1_c0 + frobenius_coeff_fq6_c1_c1*u
        expected = (u + 9)**(((qq^2) - 1) / 3)
        assert q2_mul_factor == expected, 'q2_mul_factor is wrong!'

        # Writing xi to the power of q-1 over 2
        xi_to_q_minus_1_over_2_c0 = 0xe4bbdd0c2936b629 | (0xbb30f162e133bacb << 64) | (0x31a9d1b6f9645366 << 128) | (0x253570bea500f8dd << 192)
        xi_to_q_minus_1_over_2_c0 = Fq(xi_to_q_minus_1_over_2_c0) * Fq(2^(-256))
        xi_to_q_minus_1_over_2_c1 = 0xa1d77ce45ffe77c7 | (0x07affd117826d1db << 64) | (0x6d16bd27bb7edc6b << 128) | (0x2c87200285defecc << 192)
        xi_to_q_minus_1_over_2_c1 = Fq(xi_to_q_minus_1_over_2_c1) * Fq(2^(-256))
        xi_to_q_minus_1_over_2 = xi_to_q_minus_1_over_2_c0 + xi_to_q_minus_1_over_2_c1*u
        expected = (u + 9)**(((qq^1) - 1) / 2)
        assert xi_to_q_minus_1_over_2 == expected, 'xi_to_q_minus_1_over_2 is wrong!'

        # Copying q1
        q1 = copy(q)
        # Copying q elementwise
        q1X, q1Y = copy(q1[0]), copy(q1[1])

        # --- Negating q1.x.c1 ---
        # Copying c0 and c1 of q1X
        q1X_c0, q1X_c1 = copy(q1X[0]), copy(q1X[1])
        # Inverting c1
        q1X_c1 = -q1X_c1
        # Copying back to q1X
        q1X = q1X_c0 + q1X_c1*u

        # Multiplying by frobenius coefficient
        q1X = q1X * q1_mul_factor
        # Copying c0 and c1 of q1Y
        q1Y_c0, q1Y_c1 = copy(q1Y[0]), copy(q1Y[1])
        # Inverting c1
        q1Y_c1 = -q1Y_c1
        # Copying back to q1Y
        q1Y = q1Y_c0 + q1Y_c1*u
        # Multiplying by xi to the power of q-1 over 2
        q1Y = q1Y * xi_to_q_minus_1_over_2
        # Copying back to q1
        q1 = G2((q1X, q1Y))

        (c0, c3, c4), tq1 = addition_step(t, q1, p)
        assert tq1 == t + q1
        c0c3c4 = c0c3c4_to_fq12(c0, c3, c4)
        f = ell(f, (c0, c3, c4), p)
        t = tq1

        minus_q2 = copy(q)
        minus_q2X, minus_q2Y = copy(minus_q2[0]), copy(minus_q2[1])
        minus_q2X = minus_q2X * q2_mul_factor
        minus_q2 = G2((minus_q2X, minus_q2Y))

        (c0, c3, c4), tq2 = addition_step(t, minus_q2, p)
        assert tq2 == t + minus_q2
        c0c3c4 = c0c3c4_to_fq12(c0, c3, c4)
        f = ell(f, (c0, c3, c4), p)
        return f

    def pairing(P, Q):
        return miller_loop(P, Q)^e

    e1 = pairing(a*P, b*Q)
    e2 = pairing(P, Q)
    assert e1**r == 1 and e2**r == 1, "Pairing result is not in the rth roots of unity subgroup!"
    assert e2**(a*b) == e1, "Pairing result is not correct!"

    tests_dict['tests'].append({
        'g1_point': g1_point_to_dictionary(P),
        'g2_point': g2_point_to_dictionary(Q),
        'g1_point_scaled': g1_point_to_dictionary(k*P),
        'g2_point_scaled': g2_point_to_dictionary(k*Q),
        'scalar': str(k),
    })

print('Pairing tests formed successfully!')

# Saving the json file
FILE_NAME = '../json/ec_pairing/pairing_tests.json'

print(f'Saving the pairing tests to {FILE_NAME}...')

with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)