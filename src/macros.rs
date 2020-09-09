#![allow(unused_macros)]

macro_rules! forward_val_assign_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res {
            #[inline]
            fn $method(&mut self, other: $res2) {
                self.$method(&other);
            }
        }
    };
}



macro_rules! forward_unop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp for &'macro_lifetime $res {
            type Output = $res;
            #[inline]
            fn $method(self) -> $res {
                self.clone().$method()
            }
        }
    };
}



macro_rules! forward_val_val_binop_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                // forward to val-ref, with the larger capacity as val
                if self.capacity() >= other.capacity() {
                    $imp::$method(self, &other)
                } else {
                    $imp::$method(other, &self)
                }
            }
        }
    };
}


macro_rules! forward_ref_val_binop_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for &'macro_lifetime $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                // reverse, forward to val-ref
                $imp::$method(other, self)
            }
        }
    };
}


macro_rules! forward_ref_ref_binop_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime_b $res> for &'macro_lifetime_a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                // forward to val-ref, choosing the larger to clone
                if self.capacity() >= other.capacity() {
                    $imp::$method(self.clone(), other)
                } else {
                    $imp::$method(other.clone(), self)
                }
            }
        }
    };
}

macro_rules! forward_val_val_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: $res2) -> $res3 {
                // forward to val-ref
                $imp::$method(self, &other)
            }
        }
    };
}

macro_rules! forward_ref_val_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for &'macro_lifetime $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: $res2) -> $res3 {
                // forward to ref-ref
                $imp::$method(self, &other)
            }
        }
    };
}

macro_rules! forward_val_ref_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'a $res> for $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: &$res2) -> $res3 {
                // forward to ref-ref
                $imp::$method(&self, other)
            }
        }
    };
}

macro_rules! forward_ref_ref_binop_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime_b $res2> for &'macro_lifetime_a $res {
            type Output = $res3;

            #[inline]
            fn $method(self, other: &$res2) -> $res3 {
                // forward to val-ref
                $imp::$method(self.clone(), other)
            }
        }
    };
}


macro_rules! forward_all_binop_to_val_ref_commutative_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident for $res:ty, $method:ident) => {
        forward_val_val_binop_commutative_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp for $res, $method);
        forward_ref_val_binop_commutative_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp for $res, $method);
        forward_ref_ref_binop_commutative_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp for $res, $method);
    };
}


macro_rules! forward_all_binop_to_val_ref_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        forward_val_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_ref_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_ref_ref_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
    };
}

macro_rules! forward_all_binop_to_ref_ref_ex {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident -> $res3:ty) => {
        forward_val_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_val_ref_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
        forward_ref_val_binop_ex!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method -> $res3);
    };
}

macro_rules! swap_commutative_val_val {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}

macro_rules! swap_commutative_val_ref {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime $res> for $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}

macro_rules! swap_commutative_ref_val {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> $imp<$res> for &'macro_lifetime $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}


macro_rules! swap_commutative_ref_ref {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> $imp<&'macro_lifetime_b $res> for &'macro_lifetime_a $res2 {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}


macro_rules! swap_commutative {
    (impl<$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $imp:ident<$res2:ty> for $res:ty, $method:ident) => {
        swap_commutative_val_val!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
        swap_commutative_val_ref!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
        swap_commutative_ref_val!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
        swap_commutative_ref_ref!(impl<$($imp_l, )*$($imp_i : $imp_p),+> $imp<$res2> for $res, $method);
    };
}

#[macro_export]
macro_rules! num {
    ($x:expr) => {
        $crate::native::num::Num::from($x)
    };
}

#[macro_export]
macro_rules! groth16_ethereum_bindings {
    ($modname:ident, $cir_pub:ty, $ccir_pub:ident, $cir_sec:ty, $ccir_sec:ident, $cir_params:ident, $cir_main:ident, $test_data:ident) => {

        mod $modname {
            use super::*;
            use clap::Clap;
            use std::{fs::File, io::Write};

            use pairing::bn256::{Fr, Bn256, Fq};
            use $crate::{
                native::num::Num,
                core::{signal::Signal, cs::{Circuit, TestCS}},
                helpers::groth16::{
                    prover::{generate_keys, prove, Proof, Parameters},
                    verifier::{truncate_verifying_key, verify, TruncatedVerifyingKeyData, TruncatedVerifyingKey},
                    ethereum::generate_sol_data
                }
            };

            #[derive(Clap)]
            struct Opts {
                #[clap(subcommand)]
                command: SubCommand,
            }

            #[derive(Clap)]
            enum SubCommand {
                /// Generate a SNARK proof
                Prove(ProveOpts),
                /// Verify a SNARK proof
                Verify(VerifyOpts),
                /// Generate trusted setup parameters
                Setup(SetupOpts),
                /// Generate verifier smart contract
                GenerateVerifier(GenerateVerifierOpts),
                /// Generate test object
                GenerateTestData(GenerateTestDataOpts)
            }

            /// A subcommand for generating a SNARK proof
            #[derive(Clap)]
            struct ProveOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String,
                /// Output file for proof JSON
                #[clap(short = "r", long = "proof", default_value = "proof.json")]
                proof: String,
                /// Output file for public inputs JSON
                #[clap(short = "i", long = "inputs", default_value = "inputs.json")]
                inputs: String,
            }

            /// A subcommand for verifying a SNARK proof
            #[derive(Clap)]
            struct VerifyOpts {
                /// Snark verification key
                #[clap(short = "v", long = "vk", default_value = "verification_key.json")]
                vk: String,
                /// Proof JSON file
                #[clap(short = "r", long = "proof", default_value = "proof.json")]
                proof: String,
                /// Public inputs JSON file
                #[clap(short = "i", long = "inputs", default_value = "inputs.json")]
                inputs: String,
            }

            /// A subcommand for generating a trusted setup parameters
            #[derive(Clap)]
            struct SetupOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Snark verifying key file
                #[clap(short = "v", long = "vk", default_value = "verification_key.json")]
                vk: String,
            }

            /// A subcommand for generating a Solidity verifier smart contract
            #[derive(Clap)]
            struct GenerateVerifierOpts {
                /// Snark verification key
                #[clap(short = "v", long = "vk", default_value = "verification_key.json")]
                vk: String,
                /// Output smart contract name
                #[clap(short = "s", long = "solidity", default_value = "verifier.sol")]
                solidity: String,
            }

            #[derive(Clap)]
            struct GenerateTestDataOpts {
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String
            }

                
            
            struct CircuitObject {
                p:Option<$cir_pub>,
                s:Option<$cir_sec>
            }

            impl fawkes_crypto::core::cs::Circuit for CircuitObject  {
                type F = Fr;
                fn synthesize<CS: fawkes_crypto::core::cs::ConstraintSystem<F=pairing::bn256::Fr>>(
                    &self,
                    cs: &CS
                ) {
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    let s = $ccir_sec::alloc(cs, self.s.as_ref());
                    $cir_main(&p, &s, &$cir_params);
                    p.inputize();
                }
            
                fn get_inputs(&self) -> Option<Vec<Num<Self::F>>> {
                    let ref cs = TestCS::new();
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    p.linearize().iter().map(|e|e.get_value()).collect()
                }
            }

            impl Default for CircuitObject {
                fn default() -> Self {
                    Self {
                        p: None,
                        s: None
                    }
                }
            }



            fn cli_setup(o:SetupOpts) {
                let params = generate_keys::<Bn256, CircuitObject>();
                let vk_data_str = serde_json::to_string_pretty(&truncate_verifying_key(&params.vk).into_data()).unwrap();
                params.write(File::create(o.params).unwrap()).unwrap();
                std::fs::write(o.vk, &vk_data_str.into_bytes()).unwrap();
                println!("setup OK");
            }
        
            fn cli_generate_verifier(o: GenerateVerifierOpts) {
                let vk_str = std::fs::read_to_string(o.vk).unwrap();
                let vk :TruncatedVerifyingKeyData<Fq> = serde_json::from_str(&vk_str).unwrap();
                let sol_str = generate_sol_data(&vk);
                File::create(o.solidity).unwrap().write(&sol_str.into_bytes()).unwrap();
                println!("solidity verifier generated")
            }
        
            fn cli_verify(o:VerifyOpts) {
                let vk_str = std::fs::read_to_string(o.vk).unwrap();
                let proof_str = std::fs::read_to_string(o.proof).unwrap();
                let public_inputs_str = std::fs::read_to_string(o.inputs).unwrap();
        
                let vk = TruncatedVerifyingKey::<Bn256>::from_data(&serde_json::from_str(&vk_str).unwrap());
                let proof = Proof::<Bn256>::from_data(&serde_json::from_str(&proof_str).unwrap());
                let public_inputs = serde_json::from_str::<Vec<Num<Fr>>>(&public_inputs_str).unwrap().into_iter().map(|e| e.into_inner()).collect::<Vec<_>>();
        
                println!("Verify result is {}.", verify(&vk, &proof, &public_inputs).unwrap_or(false))
            }
        
            fn cli_generate_test_data(o:GenerateTestDataOpts) {
                let data: ($cir_pub, $cir_sec) = $test_data();
                let data_str = serde_json::to_string_pretty(&data).unwrap();
                std::fs::write(o.object, &data_str.into_bytes()).unwrap();
                println!("Test data generated")
        
            }
        
            fn cli_prove(o:ProveOpts) {
                let params = Parameters::<Bn256>::read(File::open(o.params).unwrap(), false).unwrap();
                let object_str = std::fs::read_to_string(o.object).unwrap();
        
                let (p, s) = serde_json::from_str::<($cir_pub, $cir_sec)>(&object_str).unwrap();
                let c = CircuitObject {p:Some(p), s:Some(s)};
                let proof = prove(&c, &params);
                let inputs = c.get_inputs().unwrap();
        
                let proof_str = serde_json::to_string_pretty(&proof.into_data()).unwrap();
                let inputs_str = serde_json::to_string_pretty(&inputs).unwrap();
        
                std::fs::write(o.proof, &proof_str.into_bytes()).unwrap();
                std::fs::write(o.inputs, &inputs_str.into_bytes()).unwrap();
                
                println!("Proved")
            }
        
        
            pub fn cli_main() {
                let opts: Opts = Opts::parse();
                match opts.command {
                    SubCommand::Prove(o) => cli_prove(o),
                    SubCommand::Verify(o) => cli_verify(o),
                    SubCommand::Setup(o) => cli_setup(o),
                    SubCommand::GenerateVerifier(o) => cli_generate_verifier(o),
                    SubCommand::GenerateTestData(o) => cli_generate_test_data(o)
                }    
            }
        }
    }    
}


#[macro_export]
macro_rules! groth16_waves_bindings {
    ($modname:ident, $cir_pub:ty, $ccir_pub:ident, $cir_sec:ty, $ccir_sec:ident, $cir_params:ident, $cir_main:ident, $test_data:ident) => {

        mod $modname {
            use super::*;
            use clap::Clap;
            use std::{fs::File, io::Cursor};
            use $crate::base64;
            use $crate::core::field::{Field, PrimeField, PrimeFieldRepr};

            use pairing::bls12_381::{Fr, Bls12};
            use $crate::{
                native::num::Num,
                core::{signal::Signal, cs::{Circuit, TestCS}},
                helpers::groth16::{
                    prover::{generate_keys, prove, Proof, Parameters},
                    verifier::{truncate_verifying_key, verify, TruncatedVerifyingKey}
                }
            };


            fn read_num<Fr:Field>(data: &[u8]) -> Option<Num<Fr>> {
                let mut fr_repr = Fr::zero().into_raw_repr();
            
                match fr_repr.read_be(data) {
                    Err(_) => return None,
                    _ => {}
                }

                Some(Num(Fr::from_repr(fr_repr).ok()?))
            }
            
            fn read_num_vec<Fr:Field>(data: &[u8]) -> Option<Vec<Num<Fr>>> {
                let mut inputs = vec![];
                let fr_repr_sz = std::mem::size_of::<<Fr as PrimeField>::Repr>();
                let data_len = data.len();
                
                if data_len % fr_repr_sz != 0 {
                    None
                } else {
                    for offset in (0..data_len).step_by(fr_repr_sz) {
                        inputs.push(read_num::<Fr>(&data[offset..])?);
                    }
                    Some(inputs)
                }
            }
            
            pub fn num_vec_to_buf<Fr:Field>(nums:&[Num<Fr>]) -> Vec<u8> {
                let fr_repr_sz = std::mem::size_of::<<Fr as PrimeField>::Repr>();
                let mut data = vec![0; fr_repr_sz * nums.len()];

                for (i, e) in nums.iter().enumerate() {
                    e.0.into_repr().write_be(&mut data[fr_repr_sz*i ..]).unwrap();
                }
                data
            }

            #[derive(Clap)]
            struct Opts {
                #[clap(subcommand)]
                command: SubCommand,
            }

            #[derive(Clap)]
            enum SubCommand {
                /// Generate a SNARK proof
                Prove(ProveOpts),
                /// Verify a SNARK proof
                Verify(VerifyOpts),
                /// Generate trusted setup parameters
                Setup(SetupOpts),
                /// Generate test object
                GenerateTestData(GenerateTestDataOpts)
            }

            /// A subcommand for generating a SNARK proof
            #[derive(Clap)]
            struct ProveOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String,
                /// Output file for proof JSON
                #[clap(short = "r", long = "proof", default_value = "proof.txt")]
                proof: String,
                /// Output file for public inputs JSON
                #[clap(short = "i", long = "inputs", default_value = "inputs.txt")]
                inputs: String,
            }

            /// A subcommand for verifying a SNARK proof
            #[derive(Clap)]
            struct VerifyOpts {
                /// Snark verification key
                #[clap(short = "v", long = "vk", default_value = "verification_key.txt")]
                vk: String,
                /// Proof JSON file
                #[clap(short = "r", long = "proof", default_value = "proof.txt")]
                proof: String,
                /// Public inputs JSON file
                #[clap(short = "i", long = "inputs", default_value = "inputs.txt")]
                inputs: String,
            }

            /// A subcommand for generating a trusted setup parameters
            #[derive(Clap)]
            struct SetupOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Snark verifying key file
                #[clap(short = "v", long = "vk", default_value = "verification_key.txt")]
                vk: String,
            }

            #[derive(Clap)]
            struct GenerateTestDataOpts {
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String
            }

                
            
            struct CircuitObject {
                p:Option<$cir_pub>,
                s:Option<$cir_sec>
            }

            impl fawkes_crypto::core::cs::Circuit for CircuitObject  {
                type F = Fr;
                fn synthesize<CS: fawkes_crypto::core::cs::ConstraintSystem<F=pairing::bls12_381::Fr>>(
                    &self,
                    cs: &CS
                ) {
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    let s = $ccir_sec::alloc(cs, self.s.as_ref());
                    $cir_main(&p, &s, &$cir_params);
                    p.inputize();
                }
            
                fn get_inputs(&self) -> Option<Vec<Num<Self::F>>> {
                    let ref cs = TestCS::new();
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    p.linearize().iter().map(|e|e.get_value()).collect()
                }
            }

            impl Default for CircuitObject {
                fn default() -> Self {
                    Self {
                        p: None,
                        s: None
                    }
                }
            }



            fn cli_setup(o:SetupOpts) {
                let params = generate_keys::<Bls12, CircuitObject>();
                let mut tvk_data = Cursor::new(Vec::<u8>::new());
                truncate_verifying_key(&params.vk).write(&mut tvk_data).unwrap();
                std::fs::write(o.vk, &base64::encode(tvk_data.get_ref()).into_bytes()).unwrap();
                params.write(File::create(o.params).unwrap()).unwrap();
                
                println!("setup OK");
            }
        
        
            fn cli_verify(o:VerifyOpts) {
                let vk_str = std::fs::read_to_string(o.vk).unwrap();
                let proof_str = std::fs::read_to_string(o.proof).unwrap();
                let public_inputs_str = std::fs::read_to_string(o.inputs).unwrap();
        
                let vk = TruncatedVerifyingKey::<Bls12>::read(&base64::decode(&vk_str).unwrap()[..]).unwrap();
                let proof = Proof::<Bls12>::read(&base64::decode(&proof_str).unwrap()[..]).unwrap();
                let public_inputs = read_num_vec(&base64::decode(&public_inputs_str).unwrap()[..]).unwrap().iter().map(|n| n.0).collect::<Vec<_>>();
        
                println!("Verify result is {}.", verify(&vk, &proof, &public_inputs).unwrap_or(false));
            }
        
            fn cli_generate_test_data(o:GenerateTestDataOpts) {
                let data: ($cir_pub, $cir_sec) = $test_data();
                let data_str = serde_json::to_string_pretty(&data).unwrap();
                std::fs::write(o.object, &data_str.into_bytes()).unwrap();
                println!("Test data generated");
        
            }
        
            fn cli_prove(o:ProveOpts) {
                let params = Parameters::<Bls12>::read(File::open(o.params).unwrap(), false).unwrap();
                let object_str = std::fs::read_to_string(o.object).unwrap();
        
                let (p, s) = serde_json::from_str::<($cir_pub, $cir_sec)>(&object_str).unwrap();
                let c = CircuitObject {p:Some(p), s:Some(s)};
                let proof = prove(&c, &params);
                let inputs = c.get_inputs().unwrap();
                
                let mut proof_data = Cursor::new(Vec::<u8>::new());
                proof.write(&mut proof_data).unwrap();
                std::fs::write(o.proof, &base64::encode(proof_data.get_ref()).into_bytes()).unwrap();
                std::fs::write(o.inputs, &base64::encode(&num_vec_to_buf(&inputs)).into_bytes()).unwrap();
                println!("Proved");
            }
        
        
            pub fn cli_main() {
                let opts: Opts = Opts::parse();
                match opts.command {
                    SubCommand::Prove(o) => cli_prove(o),
                    SubCommand::Verify(o) => cli_verify(o),
                    SubCommand::Setup(o) => cli_setup(o),
                    SubCommand::GenerateTestData(o) => cli_generate_test_data(o)
                }    
            }
        }
    }    
}

#[macro_export]
macro_rules! groth16_waves_bindings_bn256 {
    ($modname:ident, $cir_pub:ty, $ccir_pub:ident, $cir_sec:ty, $ccir_sec:ident, $cir_params:ident, $cir_main:ident, $test_data:ident) => {

        mod $modname {
            use super::*;
            use clap::Clap;
            use std::{fs::File, io::Cursor};
            use $crate::base64;
            use $crate::core::field::{Field, PrimeField, PrimeFieldRepr};

            use pairing::bn256::{Fr, Bn256};
            use $crate::{
                native::num::Num,
                core::{signal::Signal, cs::{Circuit, TestCS}},
                helpers::groth16::{
                    prover::{generate_keys, prove, Proof, Parameters},
                    verifier::{truncate_verifying_key, verify, TruncatedVerifyingKey}
                }
            };


            fn read_num<Fr:Field>(data: &[u8]) -> Option<Num<Fr>> {
                let mut fr_repr = Fr::zero().into_raw_repr();
            
                match fr_repr.read_be(data) {
                    Err(_) => return None,
                    _ => {}
                }

                Some(Num(Fr::from_repr(fr_repr).ok()?))
            }
            
            fn read_num_vec<Fr:Field>(data: &[u8]) -> Option<Vec<Num<Fr>>> {
                let mut inputs = vec![];
                let fr_repr_sz = std::mem::size_of::<<Fr as PrimeField>::Repr>();
                let data_len = data.len();
                
                if data_len % fr_repr_sz != 0 {
                    None
                } else {
                    for offset in (0..data_len).step_by(fr_repr_sz) {
                        inputs.push(read_num::<Fr>(&data[offset..])?);
                    }
                    Some(inputs)
                }
            }
            
            pub fn num_vec_to_buf<Fr:Field>(nums:&[Num<Fr>]) -> Vec<u8> {
                let fr_repr_sz = std::mem::size_of::<<Fr as PrimeField>::Repr>();
                let mut data = vec![0; fr_repr_sz * nums.len()];

                for (i, e) in nums.iter().enumerate() {
                    e.0.into_repr().write_be(&mut data[fr_repr_sz*i ..]).unwrap();
                }
                data
            }

            #[derive(Clap)]
            struct Opts {
                #[clap(subcommand)]
                command: SubCommand,
            }

            #[derive(Clap)]
            enum SubCommand {
                /// Generate a SNARK proof
                Prove(ProveOpts),
                /// Verify a SNARK proof
                Verify(VerifyOpts),
                /// Generate trusted setup parameters
                Setup(SetupOpts),
                /// Generate test object
                GenerateTestData(GenerateTestDataOpts)
            }

            /// A subcommand for generating a SNARK proof
            #[derive(Clap)]
            struct ProveOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String,
                /// Output file for proof JSON
                #[clap(short = "r", long = "proof", default_value = "proof.txt")]
                proof: String,
                /// Output file for public inputs JSON
                #[clap(short = "i", long = "inputs", default_value = "inputs.txt")]
                inputs: String,
            }

            /// A subcommand for verifying a SNARK proof
            #[derive(Clap)]
            struct VerifyOpts {
                /// Snark verification key
                #[clap(short = "v", long = "vk", default_value = "verification_key.txt")]
                vk: String,
                /// Proof JSON file
                #[clap(short = "r", long = "proof", default_value = "proof.txt")]
                proof: String,
                /// Public inputs JSON file
                #[clap(short = "i", long = "inputs", default_value = "inputs.txt")]
                inputs: String,
            }

            /// A subcommand for generating a trusted setup parameters
            #[derive(Clap)]
            struct SetupOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Snark verifying key file
                #[clap(short = "v", long = "vk", default_value = "verification_key.txt")]
                vk: String,
            }

            #[derive(Clap)]
            struct GenerateTestDataOpts {
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String
            }

                
            
            struct CircuitObject {
                p:Option<$cir_pub>,
                s:Option<$cir_sec>
            }

            impl fawkes_crypto::core::cs::Circuit for CircuitObject  {
                type F = Fr;
                fn synthesize<CS: fawkes_crypto::core::cs::ConstraintSystem<F=pairing::bn256::Fr>>(
                    &self,
                    cs: &CS
                ) {
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    let s = $ccir_sec::alloc(cs, self.s.as_ref());
                    $cir_main(&p, &s, &$cir_params);
                    p.inputize();
                }
            
                fn get_inputs(&self) -> Option<Vec<Num<Self::F>>> {
                    let ref cs = TestCS::new();
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    p.linearize().iter().map(|e|e.get_value()).collect()
                }
            }

            impl Default for CircuitObject {
                fn default() -> Self {
                    Self {
                        p: None,
                        s: None
                    }
                }
            }



            fn cli_setup(o:SetupOpts) {
                let params = generate_keys::<Bn256, CircuitObject>();
                let mut tvk_data = Cursor::new(Vec::<u8>::new());
                truncate_verifying_key(&params.vk).write(&mut tvk_data).unwrap();
                std::fs::write(o.vk, &base64::encode(tvk_data.get_ref()).into_bytes()).unwrap();
                params.write(File::create(o.params).unwrap()).unwrap();
                
                println!("setup OK");
            }
        
        
            fn cli_verify(o:VerifyOpts) {
                let vk_str = std::fs::read_to_string(o.vk).unwrap();
                let proof_str = std::fs::read_to_string(o.proof).unwrap();
                let public_inputs_str = std::fs::read_to_string(o.inputs).unwrap();
        
                let vk = TruncatedVerifyingKey::<Bn256>::read(&base64::decode(&vk_str).unwrap()[..]).unwrap();
                let proof = Proof::<Bn256>::read(&base64::decode(&proof_str).unwrap()[..]).unwrap();
                let public_inputs = read_num_vec(&base64::decode(&public_inputs_str).unwrap()[..]).unwrap().iter().map(|n| n.0).collect::<Vec<_>>();
        
                println!("Verify result is {}.", verify(&vk, &proof, &public_inputs).unwrap_or(false));
            }
        
            fn cli_generate_test_data(o:GenerateTestDataOpts) {
                let data: ($cir_pub, $cir_sec) = $test_data();
                let data_str = serde_json::to_string_pretty(&data).unwrap();
                std::fs::write(o.object, &data_str.into_bytes()).unwrap();
                println!("Test data generated");
        
            }
        
            fn cli_prove(o:ProveOpts) {
                let params = Parameters::<Bn256>::read(File::open(o.params).unwrap(), false).unwrap();
                let object_str = std::fs::read_to_string(o.object).unwrap();
        
                let (p, s) = serde_json::from_str::<($cir_pub, $cir_sec)>(&object_str).unwrap();
                let c = CircuitObject {p:Some(p), s:Some(s)};
                let proof = prove(&c, &params);
                let inputs = c.get_inputs().unwrap();
                
                let mut proof_data = Cursor::new(Vec::<u8>::new());
                proof.write(&mut proof_data).unwrap();
                std::fs::write(o.proof, &base64::encode(proof_data.get_ref()).into_bytes()).unwrap();
                std::fs::write(o.inputs, &base64::encode(&num_vec_to_buf(&inputs)).into_bytes()).unwrap();
                println!("Proved");
            }
        
        
            pub fn cli_main() {
                let opts: Opts = Opts::parse();
                match opts.command {
                    SubCommand::Prove(o) => cli_prove(o),
                    SubCommand::Verify(o) => cli_verify(o),
                    SubCommand::Setup(o) => cli_setup(o),
                    SubCommand::GenerateTestData(o) => cli_generate_test_data(o)
                }    
            }
        }
    }    
}



#[macro_export]
macro_rules! groth16_near_bindings {
    ($modname:ident, $cir_pub:ty, $ccir_pub:ident, $cir_sec:ty, $ccir_sec:ident, $cir_params:ident, $cir_main:ident, $test_data:ident) => {

        mod $modname {
            use super::*;
            use clap::Clap;
            use std::{fs::File, io::Cursor};
            use $crate::base64;
            use $crate::core::field::{Field, PrimeField, PrimeFieldRepr};
            use $crate::borsh::{BorshSerialize, BorshDeserialize};

            use pairing::bn256::{Fr, Bn256};
            use $crate::{
                native::num::Num,
                core::{signal::Signal, cs::{Circuit, TestCS}},
                helpers::groth16::{
                    prover::{generate_keys, prove, Proof, Parameters},
                    verifier::{truncate_verifying_key, verify, TruncatedVerifyingKey},
                    near
                }
            };


            fn read_num<Fr:Field>(data: &[u8]) -> Option<Num<Fr>> {
                let mut fr_repr = Fr::zero().into_raw_repr();
            
                match fr_repr.read_be(data) {
                    Err(_) => return None,
                    _ => {}
                }

                Some(Num(Fr::from_repr(fr_repr).ok()?))
            }
            
            fn read_num_vec<Fr:Field>(data: &[u8]) -> Option<Vec<Num<Fr>>> {
                let mut inputs = vec![];
                let fr_repr_sz = std::mem::size_of::<<Fr as PrimeField>::Repr>();
                let data_len = data.len();
                
                if data_len % fr_repr_sz != 0 {
                    None
                } else {
                    for offset in (0..data_len).step_by(fr_repr_sz) {
                        inputs.push(read_num::<Fr>(&data[offset..])?);
                    }
                    Some(inputs)
                }
            }
            
            pub fn num_vec_to_buf<Fr:Field>(nums:&[Num<Fr>]) -> Vec<u8> {
                let fr_repr_sz = std::mem::size_of::<<Fr as PrimeField>::Repr>();
                let mut data = vec![0; fr_repr_sz * nums.len()];

                for (i, e) in nums.iter().enumerate() {
                    e.0.into_repr().write_be(&mut data[fr_repr_sz*i ..]).unwrap();
                }
                data
            }

            #[derive(Clap)]
            struct Opts {
                #[clap(subcommand)]
                command: SubCommand,
            }

            #[derive(Clap)]
            enum SubCommand {
                /// Generate a SNARK proof
                Prove(ProveOpts),
                /// Verify a SNARK proof
                Verify(VerifyOpts),
                /// Generate trusted setup parameters
                Setup(SetupOpts),
                /// Generate test object
                GenerateTestData(GenerateTestDataOpts)
            }

            /// A subcommand for generating a SNARK proof
            #[derive(Clap)]
            struct ProveOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String,
                /// Output file for proof JSON
                #[clap(short = "r", long = "proof", default_value = "proof.txt")]
                proof: String,
                /// Output file for public inputs JSON
                #[clap(short = "i", long = "inputs", default_value = "inputs.txt")]
                inputs: String,
            }

            /// A subcommand for verifying a SNARK proof
            #[derive(Clap)]
            struct VerifyOpts {
                /// Snark verification key
                #[clap(short = "v", long = "vk", default_value = "verification_key.txt")]
                vk: String,
                /// Proof JSON file
                #[clap(short = "r", long = "proof", default_value = "proof.txt")]
                proof: String,
                /// Public inputs JSON file
                #[clap(short = "i", long = "inputs", default_value = "inputs.txt")]
                inputs: String,
            }

            /// A subcommand for generating a trusted setup parameters
            #[derive(Clap)]
            struct SetupOpts {
                /// Snark trusted setup parameters file
                #[clap(short = "p", long = "params", default_value = "params.bin")]
                params: String,
                /// Snark verifying key file
                #[clap(short = "v", long = "vk", default_value = "verification_key.txt")]
                vk: String,
            }

            #[derive(Clap)]
            struct GenerateTestDataOpts {
                /// Input object JSON file
                #[clap(short = "o", long = "object", default_value = "object.json")]
                object: String
            }

                
            
            struct CircuitObject {
                p:Option<$cir_pub>,
                s:Option<$cir_sec>
            }

            impl fawkes_crypto::core::cs::Circuit for CircuitObject  {
                type F = Fr;
                fn synthesize<CS: fawkes_crypto::core::cs::ConstraintSystem<F=pairing::bn256::Fr>>(
                    &self,
                    cs: &CS
                ) {
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    let s = $ccir_sec::alloc(cs, self.s.as_ref());
                    $cir_main(&p, &s, &$cir_params);
                    p.inputize();
                }
            
                fn get_inputs(&self) -> Option<Vec<Num<Self::F>>> {
                    let ref cs = TestCS::new();
                    let p = $ccir_pub::alloc(cs, self.p.as_ref());
                    p.linearize().iter().map(|e|e.get_value()).collect()
                }
            }

            impl Default for CircuitObject {
                fn default() -> Self {
                    Self {
                        p: None,
                        s: None
                    }
                }
            }



            fn cli_setup(o:SetupOpts) {
                let params = generate_keys::<Bn256, CircuitObject>();
                let tvk = truncate_verifying_key(&params.vk);
                let mut tvk_data = tvk.into_data().try_to_vec().unwrap();
                std::fs::write(o.vk, &base64::encode(&tvk_data).into_bytes()).unwrap();
                params.write(File::create(o.params).unwrap()).unwrap();
                
                println!("setup OK");
            }
        
        
            fn cli_verify(o:VerifyOpts) {
                let vk_str = std::fs::read_to_string(o.vk).unwrap();
                let proof_str = std::fs::read_to_string(o.proof).unwrap();
                let public_inputs_str = std::fs::read_to_string(o.inputs).unwrap();
        
                let vk = TruncatedVerifyingKey::<Bn256>::from_data(&<_>::try_from_slice(&base64::decode(&vk_str).unwrap()[..]).unwrap());
                let proof = Proof::<Bn256>::from_data(&<_>::try_from_slice(&base64::decode(&proof_str).unwrap()[..]).unwrap());
                let public_inputs = <Vec<Num<Fr>>>::try_from_slice(&base64::decode(&public_inputs_str).unwrap()[..]).unwrap().iter().map(|n| n.0).collect::<Vec<_>>();
        
                println!("Verify result is {}.", verify(&vk, &proof, &public_inputs).unwrap_or(false));
            }
        
            fn cli_generate_test_data(o:GenerateTestDataOpts) {
                let data: ($cir_pub, $cir_sec) = $test_data();
                let data_str = serde_json::to_string_pretty(&data).unwrap();
                std::fs::write(o.object, &data_str.into_bytes()).unwrap();
                println!("Test data generated");
        
            }
        
            fn cli_prove(o:ProveOpts) {
                let params = Parameters::<Bn256>::read(File::open(o.params).unwrap(), false).unwrap();
                let object_str = std::fs::read_to_string(o.object).unwrap();
        
                let (p, s) = serde_json::from_str::<($cir_pub, $cir_sec)>(&object_str).unwrap();
                let c = CircuitObject {p:Some(p), s:Some(s)};
                let proof = prove(&c, &params);
                let inputs = c.get_inputs().unwrap();
                
                let proof_data = proof.into_data().try_to_vec().unwrap();
                let inputs_data = inputs.try_to_vec().unwrap();
                std::fs::write(o.proof, &base64::encode(&proof_data).into_bytes()).unwrap();
                std::fs::write(o.inputs, &base64::encode(&inputs_data).into_bytes()).unwrap();
                println!("Proved");
            }
        
        
            pub fn cli_main() {
                let opts: Opts = Opts::parse();
                match opts.command {
                    SubCommand::Prove(o) => cli_prove(o),
                    SubCommand::Verify(o) => cli_verify(o),
                    SubCommand::Setup(o) => cli_setup(o),
                    SubCommand::GenerateTestData(o) => cli_generate_test_data(o)
                }    
            }
        }
    }    
}


