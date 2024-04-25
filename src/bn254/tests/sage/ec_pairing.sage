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

def c0c3c4_to_fq12(c0: Fq2, c3: Fq2, c4: Fq2) -> Fq12:
    return c0[0] + c0[1]*(W^6-9) + (c3[0]+c3[1]*(W^6-9))*W + (c4[0]+c4[1]*(W^6-9))*W^3

# Defining the G1 Curve and its generator
G1 = EllipticCurve(Fq, [0, 3])
G1_GEN = G1(1, 2)

# Defining the G2 Curve
b = 3 / (u + 9)
G2 = EllipticCurve(Fq2, [0, b])
G2_GEN = G2(10857046999023057135944570762232829481370756359578518086990519993285655852781+
            11559732032986387107991004021392285783925812861821192530917403151452391805634*u,
            8495653923123431417604973247489272438418190587263600148770280649306958101930+
            4082367875863433681332203403145435568316851327593401208105741076214120093531*u)

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

# Some coefficients for easier life
SIX_U_PLUS_TWO_WNAF = [
    0, 0, 0, 1, 0, 1, 0, -1, 
    0, 0, 1, -1, 0, 0, 1, 0, 
    0, 1, 1, 0, -1, 0, 0, 1, 
    0, -1, 0, 0, 0, 0, 1, 1, 
    1, 0, 0, -1, 0, 0, 1, 0, 
    0, 0, 0, 0, -1, 0, 0, 1, 
    1, 0, 0, -1, 0, 0, 0, 1, 
    1, 0, -1, 0, 0, 1, 0, 1, 1
]

# Converts the Montomery form represented by 4 64-bit limbs to an integer in Fq
def from_libms(limbs):
    montomery = limbs[0] | (limbs[1] << 64) | (limbs[2] << 128) | (limbs[3] << 192)
    return Fq(montomery) * Fq(2^(-256))

# This is for the last step of Miller loop
FROBENIUS_COEFF_FQ6_C1_1 = from_libms([
    0xb5773b104563ab30,
    0x347f91c8a9aa6454,
    0x7a007127242e0991,
    0x1956bcd8118214ec,
]) + from_libms([
    0x6e849f1ea0aa4757, 
    0xaa1c7b6d89f89141, 
    0xb6e713cdfae0ca3a, 
    0x26694fbb4e82ebc3,
])*u
assert FROBENIUS_COEFF_FQ6_C1_1 == (9+u)**((q-1)/3), 'FROBENIUS_COEFF_FQ6_C1_1 is not correct!'

# (9+u)**((q-1)/2)
XI_TO_Q_MINUS_1_OVER_2 = from_libms([
    0xe4bbdd0c2936b629, 
    0xbb30f162e133bacb, 
    0x31a9d1b6f9645366, 
    0x253570bea500f8dd,
]) + from_libms([
    0xa1d77ce45ffe77c7, 
    0x07affd117826d1db, 
    0x6d16bd27bb7edc6b, 
    0x2c87200285defecc,
])*u
assert XI_TO_Q_MINUS_1_OVER_2 == (9+u)**((q-1)/2), 'Non-XI_TO_Q_MINUS_1_OVER_2 is not correct!'

# (9+u)**((q^2-1)/3)
FROBENIUS_COEFF_FQ6_C1_2 = from_libms([
    0x3350c88e13e80b9c,
    0x7dce557cdb5e56b9,
    0x6001b4b8b615564a,
    0x2682e617020217e0,
]) + from_libms([
    0x0, 
    0x0, 
    0x0, 
    0x0,
])*u
assert FROBENIUS_COEFF_FQ6_C1_2 == (9+u)**((q^2-1)/3), 'FROBENIUS_COEFF_FQ6_C1_2 is not correct!'

# --- Line functions tested ---
# Original implementation from https://eprint.iacr.org/2010/354.pdf

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

# zksync implementation:
# https://github.com/matter-labs/pairing/tree/master/src/bn256
def doubling_step_zksync(R: tuple[Fq2, Fq2, Fq2]):
    R_X, R_Y, R_Z = copy(R[0]), copy(R[1]), copy(R[2])

    #// Adaptation of Algorithm 26, https://eprint.iacr.org/2010/354.pdf
    #let mut tmp0 = r.x;
    tmp0 = copy(R_X)
    #tmp0.square();
    tmp0 = tmp0^2

    #let mut tmp1 = r.y;
    tmp1 = copy(R_Y)
    #tmp1.square();
    tmp1 = tmp1^2

    #let mut tmp2 = tmp1;
    tmp2 = copy(tmp1)
    #tmp2.square();
    tmp2 = tmp2^2

    #let mut tmp3 = tmp1;
    tmp3 = copy(tmp1)
    #tmp3.add_assign(&r.x);
    tmp3 = tmp3 + R_X
    #tmp3.square();
    tmp3 = tmp3^2
    #tmp3.sub_assign(&tmp0);
    tmp3 = tmp3 - tmp0
    #tmp3.sub_assign(&tmp2);
    tmp3 = tmp3 - tmp2
    #tmp3.double();
    tmp3 = tmp3 * 2

    #let mut tmp4 = tmp0;
    tmp4 = copy(tmp0)
    #tmp4.double();
    tmp4 = tmp4 * 2
    #tmp4.add_assign(&tmp0);
    tmp4 = tmp4 + tmp0

    #let mut tmp6 = r.x;
    tmp6 = copy(R_X)
    #tmp6.add_assign(&tmp4);
    tmp6 = tmp6 + tmp4

    #let mut tmp5 = tmp4;
    tmp5 = copy(tmp4)
    #tmp5.square();
    tmp5 = tmp5^2

    #let mut zsquared = r.z;
    #zsquared.square();
    z_squared = R_Z^2

    #r.x = tmp5;
    T_X = copy(tmp5)
    #r.x.sub_assign(&tmp3);
    T_X = T_X - tmp3
    #r.x.sub_assign(&tmp3);
    T_X = T_X - tmp3

    #r.z.add_assign(&r.y);
    T_Z = R_Z + R_Y
    #r.z.square();
    T_Z = T_Z^2
    #r.z.sub_assign(&tmp1);
    T_Z = T_Z - tmp1
    #r.z.sub_assign(&zsquared);
    T_Z = T_Z - z_squared

    #r.y = tmp3;
    T_Y = copy(tmp3)
    #r.y.sub_assign(&r.x);
    T_Y = T_Y - T_X
    #r.y.mul_assign(&tmp4);
    T_Y = T_Y * tmp4

    #tmp2.double();
    #tmp2.double();
    #tmp2.double();
    tmp2 = tmp2 * 2
    tmp2 = tmp2 * 2
    tmp2 = tmp2 * 2

    #r.y.sub_assign(&tmp2);
    T_Y = T_Y - tmp2

    #// up to here everything was by algorith, line 11
    #// use R instead of new T

    #// tmp3 is the first part of line 12
    #tmp3 = tmp4;
    tmp3 = copy(tmp4)
    #tmp3.mul_assign(&zsquared);
    tmp3 = tmp3 * z_squared
    #tmp3.double();
    tmp3 = tmp3 * 2
    #tmp3.negate();
    tmp3 = -tmp3

    #// tmp6 is from line 14
    #tmp6.square();
    tmp6 = tmp6^2
    #tmp6.sub_assign(&tmp0);
    tmp6 = tmp6 - tmp0
    #tmp6.sub_assign(&tmp5);
    tmp6 = tmp6 - tmp5

    #tmp1.double();
    tmp1 = tmp1 * 2
    #tmp1.double();
    tmp1 = tmp1 * 2

    #tmp6.sub_assign(&tmp1);
    tmp6 = tmp6 - tmp1

    #// tmp0 is the first part of line 16
    #tmp0 = r.z;
    tmp0 = copy(T_Z)
    #tmp0.mul_assign(&zsquared);
    tmp0 = tmp0 * z_squared
    #tmp0.double();
    tmp0 = tmp0 * 2

    return (tmp0, tmp3, tmp6), (T_X, T_Y, T_Z)

def addition_step_zksync(R: tuple[Fq2, Fq2, Fq2], Q: tuple[Fq2, Fq2, Fq2]):
    R_X, R_Y, R_Z = copy(R[0]), copy(R[1]), copy(R[2])
    Q_X, Q_Y, Q_Z = copy(Q[0]), copy(Q[1]), copy(Q[2])
    
    #Adaptation of Algorithm 27, https://eprint.iacr.org/2010/354.pdf
    #let mut zsquared = r.z;
    #zsquared.square();
    z_squared = R_Z^2
    
    #let mut ysquared = q.y;
    #ysquared.square();
    y_squared = Q_Y^2

    #// t0 corresponds to line 1
    #let mut t0 = zsquared;
    #t0.mul_assign(&q.x);

    t0 = z_squared * Q_X

    #// t1 corresponds to lines 2 and 3
    #let mut t1 = q.y;
    t1 = copy(Q_Y)
    #t1.add_assign(&r.z);
    t1 = t1 + R_Z
    #t1.square();
    t1 = t1^2
    #t1.sub_assign(&ysquared);
    t1 = t1 - y_squared
    #t1.sub_assign(&zsquared);
    t1 = t1 - z_squared
    #t1.mul_assign(&zsquared);
    t1 = t1 * z_squared

    #// t2 corresponds to line 4
    #let mut t2 = t0;
    t2 = copy(t0)
    #t2.sub_assign(&r.x);
    t2 = t2 - R_X

    #// t3 corresponds to line 5
    #let mut t3 = t2;
    t3 = copy(t2)
    #t3.square();
    t3 = t3*t3

    #// t4 corresponds to line 6
    #let mut t4 = t3;
    #t4.double();
    #t4.double();
    t4 = copy(t3)
    t4 = t4 * 2
    t4 = t4 * 2

    #// t5 corresponds to line 7
    #let mut t5 = t4;
    #t5.mul_assign(&t2);
    t5 = copy(t4)
    t5 = t5 * t2

    #// t6 corresponds to line 8
    #let mut t6 = t1;
    #t6.sub_assign(&r.y);
    #t6.sub_assign(&r.y);
    t6 = copy(t1)
    t6 = t6 - R_Y
    t6 = t6 - R_Y

    #// t9 corresponds to line 9
    #let mut t9 = t6;
    #t9.mul_assign(&q.x);
    t9 = copy(t6)
    t9 = t9 * Q_X

    #// corresponds to line 10
    #let mut t7 = t4;
    #t7.mul_assign(&r.x);
    t7 = copy(t4)
    t7 = t7 * R_X

    #// corresponds to line 11, but assigns to r.x instead of T.x
    #r.x = t6;
    T_X = copy(t6)

    #r.x.square();
    T_X = T_X^2
    #r.x.sub_assign(&t5);
    T_X = T_X - t5
    #r.x.sub_assign(&t7);
    T_X = T_X - t7
    #r.x.sub_assign(&t7);
    T_X = T_X - t7

    #// corresponds to line 12, but assigns to r.z instead of T.z
    #r.z.add_assign(&t2);
    T_Z = t2 + R_Z
    #r.z.square();
    T_Z = T_Z^2
    #r.z.sub_assign(&zsquared);
    T_Z = T_Z - z_squared
    #r.z.sub_assign(&t3);
    T_Z = T_Z - t3

    #// corresponds to line 13
    #let mut t10 = q.y;
    t10 = copy(Q_Y)
    #t10.add_assign(&r.z);
    t10 = t10 + T_Z

    #// corresponds to line 14
    #let mut t8 = t7;
    t8 = copy(t7)
    #t8.sub_assign(&r.x);
    t8 = t8 - T_X
    #t8.mul_assign(&t6);
    t8 = t8 * t6

    #// corresponds to line 15
    #t0 = r.y;
    t0 = copy(R_Y)
    #t0.mul_assign(&t5);
    t0 = t0 * t5
    #t0.double();
    t0 = t0 * 2

    #// corresponds to line 12, but assigns to r.y instead of T.y
    #r.y = t8;
    T_Y = copy(t8)
    #r.y.sub_assign(&t0);
    T_Y = T_Y - t0

    #// corresponds to line 17
    #t10.square();
    t10 = t10^2
    #t10.sub_assign(&ysquared);
    t10 = t10 - y_squared

    #let mut ztsquared = r.z;
    #ztsquared.square();
    zt_squared = T_Z^2
    
    #t10.sub_assign(&ztsquared);
    t10 = t10 - zt_squared

    #// corresponds to line 18
    #t9.double();
    t9 = t9 * 2
    #t9.sub_assign(&t10);
    t9 = t9 - t10

    #// t10 = 2*Zt from Algo 27, line 19
    #t10 = r.z;
    t10 = copy(T_Z)
    #t10.double();
    t10 = t10 * 2

    #// t1 = first multiplicator of line 21
    #t6.negate();
    #t1 = t6;
    t6 = -t6
    t1 = copy(t6)
    #t1.double();
    t1 = t1 * 2

    #// t9 corresponds to t9 from Algo 27
    return (t10, t1, t9), (T_X, T_Y, T_Z)

LINE_FUNCTIONS_TESTS_NUMBER = 2

print('Preparing the line functions tests...')
tests_dict = {'tests': []}

for _ in range(LINE_FUNCTIONS_TESTS_NUMBER):
    # Generating two random points
    Q = G2.random_point()
    R = G2.random_point()
    P = G1.random_point()
    
    # Testing the line functions
    (c0_1, c3_1, c4_1), T1 = doubling_step_zksync(Q)
    (c0_2, c3_2, c4_2), T2 = doubling_step_zksync(R)
    (c0_3, c3_3, c4_3), T3 = addition_step_zksync(R, Q)

    def ell(c0c3c4, P: G1):
        c0, c3, c4 = c0c3c4
        x, y = P[0], P[1]
        return (c0 * y, c3 * x, c4)

    # Checking point correctness
    assert G2(T1[0]/T1[2]^2, T1[1]/T1[2]^3) == 2*Q, 'Doubling step 1 point is wrong!'
    assert G2(T2[0]/T2[2]^2, T2[1]/T2[2]^3) == 2*R, 'Doubling step 2 point is wrong!'
    assert G2(T3[0]/T3[2]^2, T3[1]/T3[2]^3) == Q+R, 'Addition step point is wrong!'

    # Testing two algorithms correspondence
    doubling_regular_1, T1_regular = doubling_step(Q, P)
    assert ell((c0_1, c3_1, c4_1), P) == doubling_regular_1, 'Doubling step 1 coefficients are wrong!'
    
    doubling_regular_2, T2_regular = doubling_step(R, P)
    assert ell((c0_2, c3_2, c4_2), P) == doubling_regular_2, 'Doubling step 2 coefficients are wrong!'
    
    addition_regular, T3_regular = addition_step(Q, R, P)
    assert ell((c0_3, c3_3, c4_3), P) == addition_regular, 'Addition step coefficients are wrong!'

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

# --- Final exponentiation tests ---
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

EXPONENTIATION_TESTS_NUMBER = 2

print('Preparing the final exponentiation tests...')

tests_dict = {'tests': []}

for _ in range(EXPONENTIATION_TESTS_NUMBER):
    # Generating random value
    f = Fq12.random_element()
    f_exp = f^e

    assert f_exp**r == 1, 'final exponentiation must be in the r-th power unit subfield'
    assert final_exp(f) == f_exp, 'final exponentiation is wrong!'

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
PAIRING_TESTS_NUMBER = 1

print('Preparing the pairing tests...')

tests_dict = {'tests': []}

def tuple_to_g2(t: tuple[Fq2, Fq2, Fq2]) -> G2:
    return G2(t[0]/t[2]^2, t[1]/t[2]^3)

def miller_loop(P: G1, Q: G2):
    # --- Gathering coefficients step ---
    coeffs = []
    R = copy(Q)
    neg_q = -copy(Q)
    
    for i in reversed(range(1, len(SIX_U_PLUS_TWO_WNAF))):
        c0c3c4, R2 = doubling_step_zksync(R)
        assert tuple_to_g2(R2) == 2*tuple_to_g2(R), 'Doubling step is wrong!'
        coeffs.append(c0c3c4)
        R = R2
        x = SIX_U_PLUS_TWO_WNAF[i-1]
        if x == 1:
            c0c3c4, RQ = addition_step_zksync(R, Q)
            assert tuple_to_g2(RQ) == tuple_to_g2(R) + tuple_to_g2(Q), 'Addition step is wrong!'
            coeffs.append(c0c3c4)
            R = RQ
        elif x == -1:
            c0c3c4, RQ = addition_step_zksync(R, neg_q)
            assert tuple_to_g2(RQ) == tuple_to_g2(R) - tuple_to_g2(Q), 'Addition step is wrong!'
            coeffs.append(c0c3c4)
            R = RQ
    
    # Some additional steps to finalize the Miller loop...
    # Q1 <- pi_p(Q)
    Q1 = [Q[0], Q[1], Q[2]]
    Q1[0] = Q1[0].conjugate() * FROBENIUS_COEFF_FQ6_C1_1
    Q1[1] = Q1[1].conjugate() * XI_TO_Q_MINUS_1_OVER_2
    
    c0c3c4, RQ1 = addition_step_zksync(R, Q1)
    assert tuple_to_g2(RQ1) == tuple_to_g2(R) + tuple_to_g2(Q1), 'Addition step is wrong!'
    coeffs.append(c0c3c4)
    R = RQ1

    # Q2 <- -pi_{p^2}(Q)
    Q2 = [Q[0], Q[1], Q[2]]
    Q2[0] = Q2[0] * FROBENIUS_COEFF_FQ6_C1_2

    c0c3c4, RQ2 = addition_step_zksync(R, Q2)
    assert tuple_to_g2(RQ2) == tuple_to_g2(R) + tuple_to_g2(Q2), 'Addition step is wrong!'
    coeffs.append(c0c3c4)

    coeffs = iter(coeffs)

    # --- Calculating f step ---
    def ell(f: Fq12, c0c3c4, P: G1):
        c0, c3, c4 = c0c3c4
        x, y = P[0], P[1]
        return f * c0c3c4_to_fq12(c0 * y, c3 * x, c4)

    f = Fq12.one()
    for i in reversed(range(1, len(SIX_U_PLUS_TWO_WNAF))):
        if i != len(SIX_U_PLUS_TWO_WNAF) - 1:
            f = f*f

        f = ell(f, next(coeffs), P)
        x = SIX_U_PLUS_TWO_WNAF[i-1]
        if x == 1:
            f = ell(f, next(coeffs), P)
        elif x == -1:
            f = ell(f, next(coeffs), P)

    f = ell(f, next(coeffs), P)
    f = ell(f, next(coeffs), P)
    
    assert all(False for _ in coeffs), 'coeffs must be empty up to this point'
    return f

def pairing(P, Q):
    f = miller_loop(P, Q)
    return f^e

for _ in range(PAIRING_TESTS_NUMBER):
    # Defining random elements
    a = Fq.random_element()
    A = a * G1_GEN

    b = Fq.random_element()
    B = b * G2_GEN

    pair = pairing(A, B)
    pair_AB = pairing(A, 2*B)
    pair_BA = pairing(2*A, B)

    assert pair_AB**r == pair_BA**r == pair**r == 1, "Pairing result is not in the rth roots of unity subgroup!"
    assert pair_BA == pair_AB == pair**2, "Pairing result is not correct!"

    tests_dict['tests'].append({
        'g1_point': g1_point_to_dictionary(A),
        'g2_point': g2_point_to_dictionary(B),
        'pairing': fq12_to_dictionary(pairing(A, B))
    })

print('Pairing tests formed successfully!')

# Saving the json file
FILE_NAME = '../json/ec_pairing/pairing_tests.json'

print(f'Saving the pairing tests to {FILE_NAME}...')

with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)