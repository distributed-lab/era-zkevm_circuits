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

def fq2_to_fq12(f: Fq2) -> Fq12:
    return f[0] + f[1]*(W^6 - 9)

def c0c3c4_to_fq12(c0: Fq2, c3: Fq2, c4: Fq2) -> Fq12:
    return fq2_to_fq12(c0) + fq2_to_fq12(c3)*W + fq2_to_fq12(c4)*W^3    

# Defining the G1 Curve
G1 = EllipticCurve(Fq, [0, 3])

# Defining the G2 Curve
b = 3 / (u + 9)
G2 = EllipticCurve(Fq2, [0, b])
JG2 = G2.jacobian()

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

