#![no_main]
// #![no_std]

use jwt_compact::{alg::{Rsa, RsaSignature}, Algorithm, AlgorithmSignature};
use risc0_zkvm::guest::env;
use rsa::{BigUint, RsaPublicKey};
use x509_parser::public_key::PublicKey;

risc0_zkvm::guest::entry!(main);

static R3_PEM: &[u8] = include_bytes!("lets-encrypt-r3.der");

fn verify_certificate_domain(data: &[u8]) -> String {
    let (_, cert) = x509_parser::parse_x509_certificate(data).unwrap();
    let (_, ca_cert) = x509_parser::parse_x509_certificate(R3_PEM).unwrap();
    
    // adapt to zkvm
    let alg = Rsa::rs256();
    let pub_key = ca_cert.tbs_certificate.subject_pki.parsed().unwrap();
    let rsapub_k = match pub_key {
        PublicKey::RSA(key) => {
            // let modulus = key.modulus;
            // let exponent = key.exponent;
            let e = BigUint::from_bytes_be(key.exponent);
            let n = BigUint::from_bytes_be(key.modulus);
            RsaPublicKey::new(n, e)
        }
        _ => panic!(),
    }
    .unwrap();
    let rsa_signature = RsaSignature::try_from_slice(cert.signature_value.as_ref()).unwrap();
    let res = alg.verify_signature(&rsa_signature, &rsapub_k, cert.tbs_certificate.as_ref());
    if !res {
        panic!("Invalid signature")
    }
    let domain = &cert.subject().to_string()[3..];
    domain.to_string()
}

fn main() {
    let data: Vec<u8> = env::read();
    let domain = verify_certificate_domain(&data);
    env::commit(&domain);
}