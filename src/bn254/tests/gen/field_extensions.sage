#! File for validating field extension arithmetic
import json

# --- Fq2 tests ---

# Defining the base prime field
q = 21888242871839275222246405745257275088696311157297823662689037894645226208583 # EC group order
Fq = GF(q) 

# Defining the extensions
K2.<x> = PolynomialRing(Fq)
Fq2.<u> = Fq.extension(x^2+1)
K6.<y> = PolynomialRing(Fq2)
Fq6.<v> = Fq2.extension(y^3 - (u+9))
# K12.<z> = PolynomialRing(Fq6)
# Fq12.<w> = Fq6.extension(z^2-(v))

# Defining the Fq12 is a bit more tricky...
p = Fq.characteristic()
Fq12.<G> = GF(p^12)

i = sqrt(Fq12(-1))
R12.<Y> = PolynomialRing(Fq12)

j = (Y^3 - (i+9)).roots(multiplicities=False)[0]
w = sqrt(j)

P = w.minpoly()
Fq12.<W> = GF(p^12, modulus=P)

k = Fq12.random_element()
print(k)
print(k[0])


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

# Generating Fq2 tests
print('Preparing the Fq2 tests...')
tests_dict = {'tests': []}

FQ2_TESTS_NUMBER = 30

for _ in range(FQ2_TESTS_NUMBER):
    f = Fq2.random_element()
    g = Fq2.random_element()
    sum = f + g
    diff = f - g
    prod = f * g
    quot = f / g
    f_non_residue = f * (u + 9)

    tests_dict['tests'].append({
        'scalar_1': fq2_to_dictionary(f),
        'scalar_2': fq2_to_dictionary(g),
        'expected': {
            'sum': fq2_to_dictionary(sum),
            'difference': fq2_to_dictionary(diff),
            'product': fq2_to_dictionary(prod),
            'quotient': fq2_to_dictionary(quot),
            'scalar_1_non_residue': fq2_to_dictionary(f_non_residue),
        }
    })

print('Fq2 tests formed successfully!')

# Saving the json file
FILE_NAME = '../json/fq2_tests.json'

print(f'Saving the Fq6 tests to {FILE_NAME}...')
with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

print('Successfully saved the Fq6 tests!')

# Generating Fq6 tests
print('Preparing the Fq6 tests...')
tests_dict = {'tests': []}

FQ6_TESTS_NUMBER = 30

for _ in range(FQ6_TESTS_NUMBER):
    # Defining inputs
    f = Fq6.random_element()
    g = Fq6.random_element()
    c0 = Fq2.random_element()
    c1 = Fq2.random_element()
    h_c0c1 = c0 + c1*v
    h_c1 = c1*v

    # Defining the operations tested
    sum = f + g
    diff = f - g
    prod = f * g
    prod_c1 = f * h_c1
    prod_c0c1 = f * h_c0c1
    f_inv = f.inverse()
    g_inv = g.inverse()
    quot = f / g
    f_square = f^2
    f_non_residue = f * v

    tests_dict['tests'].append({
        'scalar_1': fq6_to_dictionary(f),
        'scalar_2': fq6_to_dictionary(g),
        'c0': fq2_to_dictionary(c0),
        'c1': fq2_to_dictionary(c1),
        'expected': {
            'sum': fq6_to_dictionary(sum),
            'difference': fq6_to_dictionary(diff),
            'product': fq6_to_dictionary(prod),
            'quotient': fq6_to_dictionary(quot),
            'product_c1': fq6_to_dictionary(prod_c1),
            'product_c0c1': fq6_to_dictionary(prod_c0c1),
            'scalar_1_inverse': fq6_to_dictionary(f_inv),
            'scalar_1_square': fq6_to_dictionary(f_square),
            'scalar_1_non_residue': fq6_to_dictionary(f_non_residue),
        }
    })

print('Fq6 tests formed successfully!')

# Saving the json file
FILE_NAME = '../json/fq6_tests.json'

print(f'Saving the Fq6 tests to {FILE_NAME}...')
with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

print('Successfully saved the Fq6 tests!')

# --- Generating Fq12 tests ---
print('Preparing the Fq12 tests...')
tests_dict = {'tests': []}

FQ12_TESTS_NUMBER = 30
for _ in range(FQ12_TESTS_NUMBER):
    # Defining inputs
    f = Fq12.random_element()
    g = Fq12.random_element()

    # Defining sparse elements
    c0 = Fq2.random_element()
    c1 = Fq2.random_element()
    c3 = Fq2.random_element() 
    c4 = Fq2.random_element()
    c0c1c4 = c0[0] + c0[1]*(W^6-9) + (c1[0]+c1[1]*(W^6-9))*W^2 + (c4[0]+c4[1]*(W^6-9))*W^3
    c0c3c4 = c0[0] + c0[1]*(W^6-9) + (c3[0]+c3[1]*(W^6-9))*W + (c4[0]+c4[1]*(W^6-9))*W^3

    # Defining the operations tested
    sum = f + g
    diff = f - g
    prod = f * g
    quot = f / g
    f_inv = f.inverse()
    f_square = f^2
    prod_c0c3c4 = f * c0c3c4
    prod_c0c1c4 = f * c0c1c4

    tests_dict['tests'].append({
        'scalar_1': fq12_to_dictionary(f),
        'scalar_2': fq12_to_dictionary(g),
        'c0': fq2_to_dictionary(c0),
        'c1': fq2_to_dictionary(c1),
        'c3': fq2_to_dictionary(c3),
        'c4': fq2_to_dictionary(c4),
        'expected': {
            'sum': fq12_to_dictionary(sum),
            'difference': fq12_to_dictionary(diff),
            'product': fq12_to_dictionary(prod),
            'product_c0c3c4': fq12_to_dictionary(prod_c0c3c4),
            'product_c0c1c4': fq12_to_dictionary(prod_c0c1c4),
            'quotient': fq12_to_dictionary(quot),
            'scalar_1_inverse': fq12_to_dictionary(f_inv),
            'scalar_1_square': fq12_to_dictionary(f_square),
        }
    })

# Saving the fq12 tests
FILE_NAME = '../json/fq12_tests.json'
print(f'Saving the Fq12 tests to {FILE_NAME}...')
with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

print('Successfully saved the Fq12 tests!')