# Fawkes-Crypto - zkSNARKs framework


## Abstract

Fawkes-Crypto is a lightweight framework for building circuits in bellman, using groth16 proving system and BN254 curve.

The framework is targeted to use best practices for circuit building from circom and sapling-crypto. 

Final fields and circuit math are wrapped and operators are implemented, so, in most cases if you want to type `a+b`, you can do it.

## Example

Here is an example, how Merkle tree implementation is working.
Also you may check the rollup [here](https://github.com/snjax/fawkes-rollup).

```rust
#[derive(Clone, Signal)]
#[Value="MerkleProof<CS::F, L>"]
pub struct CMerkleProof<'a, CS:ConstraintSystem, L:Unsigned> {
    pub sibling: SizedVec<CNum<'a, CS>, L>,
    pub path: SizedVec<CBool<'a, CS>, L>
}


pub fn c_poseidon_merkle_proof_root<'a, CS:ConstraintSystem, L:Unsigned>(
    leaf:&CNum<'a, CS>, 
    proof:&CMerkleProof<'a, CS, L>,
    params:&PoseidonParams<CS::F>
) -> CNum<'a, CS> {
    let mut root = leaf.clone();
    for (p, s) in proof.path.iter().zip(proof.sibling.iter()) {
        let first = s.switch(p, &root); 
        let second = &root + s - &first;
        root = c_poseidon( [first, second].as_ref(), params);
    }
    root
}

```

`Signal` is a sparse linear combination of inputs, based on ordered linked list, so we perform arithmetics with `Signal` with `U(N)` complexity. With `Signal` bellman will allocate additional inputs only when you really need it (for example, in the case when you multiply two nonconstant `Signal`). If you perform multiplication with constant or zero `Signal`, no additional inputs will be allocated.

## Benchmarks

| Circuit | Constraints | Per bit | 
| - | - | - |
| poseidon hash (4, 8, 54) | 255 | 0.33 |
| jubjub oncurve+subgroup check | 19 | |
| ecmul_const 254 bits | 513 | 2.02 |
| ecmul 254 bits | 2296 | 9.04 |
| poseidon merkle proof 32| 7328 | |
| poseidon eddsa | 3860 | |
| rollup 1024 txs, 2^32 set | 35695616 |

At i9-9900K rollup is proved for 628 seconds. 

Source code of the rollup is available at [https://github.com/snjax/fawkes-rollup](https://github.com/snjax/fawkes-rollup).

## Circuit improvements

* We are using indeterministic subgroup checks, performing most part of computations as witness-only and perform cofactor multiplication at the circuit.
* ecmul and ecmul_cost operations are working assuming that the base point is in the subgroup. This allows us to use Montgomery (0, 0) point as adder initial state. Then the adder never reaches zero point and subgroup point, because (0, 0) is not in subgroup and we can use cheap montgomery_add circuit safely.
* improved compconstant circuit. The same PR into circomlib available [here](https://github.com/iden3/circomlib/pull/40)

See more as ethresear.ch [here](https://ethresear.ch/t/fawkes-crypto-zksnarks-framework-from-zeropool/7201).

## Authors

Igor Gulamov

## Disclaimer

Fawkes-Crypto has not been audited and is provided as is, use at your own risk.

## License

Fawkes-Crypto is available under Apache License 2.0 license or MIT as your choice.
