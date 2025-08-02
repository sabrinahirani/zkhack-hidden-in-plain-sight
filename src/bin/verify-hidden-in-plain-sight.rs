#![allow(unused, unreachable_code, dead_code)]

use ark_bls12_381::{Fr, G1Affine};
use ark_ff::*;
use ark_poly::{
    univariate::DensePolynomial, EvaluationDomain, GeneralEvaluationDomain, Polynomial,
    UVPolynomial,
};
use ark_serialize::CanonicalDeserialize;
use hidden_in_plain_sight::{generate::kzg_commit, PUZZLE_DESCRIPTION};
use prompt::{puzzle, welcome};

fn read_cha_from_file() -> (Vec<G1Affine>, Vec<Vec<Fr>>, Fr, Fr, G1Affine, Fr, Fr) {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open("challenge_data").unwrap();
    let mut bytes: Vec<u8> = vec![];
    file.read_to_end(&mut bytes).unwrap();

    let setup_bytes: Vec<u8> = bytes[0..98312].to_vec();
    let accts_bytes: Vec<u8> = bytes[98312..1130320].to_vec();
    let cha_1_bytes: Vec<u8> = bytes[1130320..1130352].to_vec();
    let cha_2_bytes: Vec<u8> = bytes[1130352..1130384].to_vec();
    let commt_bytes: Vec<u8> = bytes[1130384..1130480].to_vec();
    let opn_1_bytes: Vec<u8> = bytes[1130480..1130512].to_vec();
    let opn_2_bytes: Vec<u8> = bytes[1130512..1130544].to_vec();

    let setup = Vec::<G1Affine>::deserialize_unchecked(&setup_bytes[..]).unwrap();
    let accts = Vec::<Vec<Fr>>::deserialize_unchecked(&accts_bytes[..]).unwrap();
    let cha_1 = Fr::deserialize_unchecked(&cha_1_bytes[..]).unwrap();
    let cha_2 = Fr::deserialize_unchecked(&cha_2_bytes[..]).unwrap();
    let commt = G1Affine::deserialize_unchecked(&commt_bytes[..]).unwrap();
    let opn_1 = Fr::deserialize_unchecked(&opn_1_bytes[..]).unwrap();
    let opn_2 = Fr::deserialize_unchecked(&opn_2_bytes[..]).unwrap();

    (setup, accts, cha_1, cha_2, commt, opn_1, opn_2)
}

fn main() {
    welcome();
    puzzle(PUZZLE_DESCRIPTION);

    let (setup, accts, cha_1, cha_2, commt, opn_1, opn_2) = read_cha_from_file();

    let mut solution_commitment = G1Affine::zero();
    let number_of_accts = 1000usize;
    let domain: GeneralEvaluationDomain<Fr> =
        GeneralEvaluationDomain::new(number_of_accts + 2).unwrap();

    // Solution:
    // To find the right account we iterated over all accts, we then saw that accts[535] solves it!
    for (i, evals_p) in accts.iter().enumerate() {
        // We compute P_a(x)
        let p = DensePolynomial::from_coefficients_vec(domain.ifft(evals_p));
        
        // Evaluate it at z_1,z_2
        let p_cha_1: Fr = p.evaluate(&cha_1);
        let p_cha_2: Fr = p.evaluate(&cha_2);
        
        // Here we just set up the 2 equations with 2 unknowns
        let e_cha_1 = opn_1 - p_cha_1;
        let e_cha_2 = opn_2 - p_cha_2;
        const N: u64 = 1024u64;
        
        // We compute b_0, b_1
        let b_1 = ((e_cha_1 / (cha_1.pow(&[N]) - Fr::from(1)))
            - (e_cha_2 / (cha_2.pow(&[N]) - Fr::from(1))))
            / (cha_1 - cha_2);
        let b_0 = (e_cha_1 / (cha_1.pow(&[N]) - Fr::from(1))) - (b_1 * cha_1);

        // We set Q(x) = P(x) since most of the coeffients are the same
        let mut q = vec![Fr::from(0); 1026];
        p.coeffs
            .iter()
            .enumerate()
            .for_each(|(idx, f)| q[idx] = f.clone());
        
        // We compute q_0,q_1,q_1024,q_1025
        q[0] -= b_0;           // q[0] = p[0] - b_0
        q[1] -= b_1;           // q[1] = p[1] - b_1
        q[1024] = b_0.clone();
        q[1025] = b_1.clone();
        
        // We compute comm(Q(x)) and check if it's equal to the given commitment
        let dp = DensePolynomial::from_coefficients_vec(q);
        solution_commitment = kzg_commit(&dp, &setup);
        if (solution_commitment == commt) {
            println!("Found matching account at index {i}");
            break;
        }
    }

    assert_eq!(solution_commitment, commt);
}

