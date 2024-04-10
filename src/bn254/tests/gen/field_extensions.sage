#! File for validating field extension arithmetic
import json

# --- Fq2 tests ---

# Defining the base prime field
q = 21888242871839275222246405745257275088696311157297823662689037894645226208583 # EC group order
Fq = GF(q) 

# Defining the Fq2 extension
R.<x> = PolynomialRing(Fq)
Fq2.<u> = GF(q**2, modulus=x^2+1)

f = Fq2.random_element()
print(f)

# Generating tests
print('Preparing the fq2 tests...')
tests_dict = {'tests': []}

FQ2_TESTS_NUMBER = 100

for _ in range(FQ2_TESTS_NUMBER):
    f = Fq2.random_element()
    g = Fq2.random_element()
    sum = f + g
    diff = f - g
    prod = f * g
    quot = f / g

    tests_dict['tests'].append({
        'scalar_1': {
            'c0': str(f[0]), 
            'c1': str(f[1])
        },
        'scalar_2': {
            'c0': str(g[0]), 
            'c1': str(g[1])
        },
        'expected': {
            'sum': {
                'c0': str(sum[0]), 
                'c1': str(sum[1])
            },
            'diff': {
                'c0': str(diff[0]), 
                'c1': str(diff[1])
            },
            'prod': {
                'c0': str(prod[0]), 
                'c1': str(prod[1])
            },
            'quot': {
                'c0': str(quot[0]), 
                'c1': str(quot[1])
            }
        }
    })

print('Fq2 tests formed successfully!')

# Saving the json file
FILE_NAME = '../json/fq2_tests.json'

print(f'Saving the fq2 tests to {FILE_NAME}...')
with open(FILE_NAME, 'w') as f:
    json.dump(tests_dict, f, indent=4)

print('Successfully saved the fq2 tests!')

#_.<Y> = PolynomialRing(R2)
#xi = -X + 1
#R6.<Y> = R2.extension(Y^3-xi, 'Y')
#R6.is_field = lambda : True
#_.<Z> = PolynomialRing(R6)
#R12.<Z> = R6.extension(Z^2-(Y), 'Z')
#R12 