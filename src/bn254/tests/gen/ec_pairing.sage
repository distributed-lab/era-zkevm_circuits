import json

# Defining the base prime field
q = Integer(21888242871839275222246405745257275088696311157297823662689037894645226208583) # EC group order
Fq = GF(q) 

# r is taken from https://hackmd.io/@jpw/bn254
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

LINE_FUNCTIONS_TESTS_NUMBER = 10

print('Preparing the line functions tests...')
tests_dict = {'tests': []}

for _ in range(LINE_FUNCTIONS_TESTS_NUMBER):
    # Generating two random points
    Q = G2.random_point()
    R = G2.random_point()
    P = G1.random_point()

    def doubling(Q, P) -> tuple[Fq2, Fq2, Fq2]:
        X, Y, Z = Q[0], Q[1], Q[2]
        xP, yP = P[0], P[1]

        A = -2*Y*Z*yP
        B = 3*b*Z**2 - Y**2
        C = 3*X**2*xP
        return (A, B, C) # c0, c4, c3
    
    def adding(Q, R, P) -> tuple[Fq2, Fq2, Fq2]:
        X2, Y2, Z2 = Q[0], Q[1], Q[2]
        X, Y, Z = R[0], R[1], R[2]
        xP, yP = P[0], P[1]

        A = (X - Z*X2)*yP
        B = (Y - Z*Y2)*X2 - (X - Z*X2)*Y2
        C = -(Y - Z*Y2)*xP
        return (A, B, C) # c0, c4, c3

    line_add = adding(Q, R, P)
    line_tangent_1 = doubling(Q, P)
    line_tangent_2 = doubling(Q, P)

    def line_evaluation_to_dictionary(result: tuple) -> dict:
        A, B, C = result

        c0: Fq6 = A + 0*v + 0*v**2
        c1: Fq6 = C + B*v + 0*v**2
        return {
            'c0': fq6_to_dictionary(c0),
            'c1': fq6_to_dictionary(c1)
        }

    # Adding the test to the dictionary
    tests_dict['tests'].append({
        'g2_point_1': g2_point_to_dictionary(Q),
        'g2_point_2': g2_point_to_dictionary(R),
        'g1_point': g1_point_to_dictionary(P),
        'expected': {
            'line_add': line_evaluation_to_dictionary(line_add),
            'line_tangent_1': line_evaluation_to_dictionary(line_tangent_1),
            'line_tangent_2': line_evaluation_to_dictionary(line_tangent_2)
        }
    })

print('Line and tangent functions evaluations completed!')

# Saving the json file
FILE_NAME = '../json/ec_pairing/line_functions_tests.json'

print(f'Saving the line function tests to {FILE_NAME}...')

with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

# --- Easy exponentiation tests ---
EXPONENTIATION_TESTS_NUMBER = 10

print('Preparing the final exponentiation tests...')

tests_dict = {'tests': []}

for _ in range(EXPONENTIATION_TESTS_NUMBER):
    # Generating random value
    f = Fq12.random_element()
    f_exp = f^e
    assert f_exp**r == 1

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
    
    print(final_exp(f)**r)

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