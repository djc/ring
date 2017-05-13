// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

extern crate ring;
extern crate untrusted;

use ring::{signature, test};

#[test]
fn signature_ecdsa_verify_asn1_test() {
    test::from_file("tests/ecdsa_verify_asn1_tests.txt", |section, test_case| {
        assert_eq!(section, "");

        let curve_name = test_case.consume_string("Curve");
        let digest_name = test_case.consume_string("Digest");

        let msg = test_case.consume_bytes("Msg");
        let msg = untrusted::Input::from(&msg);

        let public_key = test_case.consume_bytes("Q");
        let public_key = untrusted::Input::from(&public_key);

        let sig = test_case.consume_bytes("Sig");
        let sig = untrusted::Input::from(&sig);

        let expected_result = test_case.consume_string("Result");

        let alg = match (curve_name.as_str(), digest_name.as_str()) {
            ("P-256", "SHA256") => &signature::ECDSA_P256_SHA256_ASN1,
            ("P-256", "SHA384") => &signature::ECDSA_P256_SHA384_ASN1,
            ("P-384", "SHA256") => &signature::ECDSA_P384_SHA256_ASN1,
            ("P-384", "SHA384") => &signature::ECDSA_P384_SHA384_ASN1,
            _ => {
                panic!("Unsupported curve+digest: {}+{}", curve_name,
                       digest_name);
            }
        };

        let actual_result = signature::verify(alg, public_key, msg, sig);
        assert_eq!(actual_result.is_ok(), expected_result == "P (0 )");

        Ok(())
    });
}

#[test]
fn signature_ecdsa_verify_fixed_test() {
    test::from_file("tests/ecdsa_verify_fixed_tests.txt", |section, test_case| {
        assert_eq!(section, "");

        let curve_name = test_case.consume_string("Curve");
        let digest_name = test_case.consume_string("Digest");

        let msg = test_case.consume_bytes("Msg");
        let msg = untrusted::Input::from(&msg);

        let public_key = test_case.consume_bytes("Q");
        let public_key = untrusted::Input::from(&public_key);

        let sig = test_case.consume_bytes("Sig");
        let sig = untrusted::Input::from(&sig);

        let expected_result = test_case.consume_string("Result");

        let alg = match (curve_name.as_str(), digest_name.as_str()) {
            ("P-256", "SHA256") => &signature::ECDSA_P256_SHA256_FIXED,
            ("P-384", "SHA384") => &signature::ECDSA_P384_SHA384_FIXED,
            _ => {
                panic!("Unsupported curve+digest: {}+{}", curve_name, digest_name);
            }
        };

        let actual_result = signature::verify(alg, public_key, msg, sig);
        assert_eq!(actual_result.is_ok(), expected_result == "P (0 )");

        Ok(())
    });
}