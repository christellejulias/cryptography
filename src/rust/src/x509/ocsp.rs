// This file is dual licensed under the terms of the Apache License, Version
// 2.0, and the BSD License. See the LICENSE file in the root of this repository
// for complete details.

use std::collections::HashMap;

use cryptography_x509::common;
use cryptography_x509::ocsp_req::CertID;
use once_cell::sync::Lazy;
use pyo3::prelude::PyAnyMethods;

use crate::backend::hashes::Hash;
use crate::error::CryptographyResult;
use crate::x509::certificate::Certificate;

pub(crate) static ALGORITHM_PARAMETERS_TO_HASH: Lazy<
    HashMap<common::AlgorithmParameters<'_>, &str>,
> = Lazy::new(|| {
    let mut h = HashMap::new();
    h.insert(common::AlgorithmParameters::Sha1(None), "SHA1");
    h.insert(common::AlgorithmParameters::Sha1(Some(())), "SHA1");
    h.insert(common::AlgorithmParameters::Sha224(None), "SHA224");
    h.insert(common::AlgorithmParameters::Sha224(Some(())), "SHA224");
    h.insert(common::AlgorithmParameters::Sha256(None), "SHA256");
    h.insert(common::AlgorithmParameters::Sha256(Some(())), "SHA256");
    h.insert(common::AlgorithmParameters::Sha384(None), "SHA384");
    h.insert(common::AlgorithmParameters::Sha384(Some(())), "SHA384");
    h.insert(common::AlgorithmParameters::Sha512(None), "SHA512");
    h.insert(common::AlgorithmParameters::Sha512(Some(())), "SHA512");
    h
});

pub(crate) static HASH_NAME_TO_ALGORITHM_IDENTIFIERS: Lazy<
    HashMap<&str, common::AlgorithmIdentifier<'_>>,
> = Lazy::new(|| {
    let mut h = HashMap::new();
    h.insert(
        "sha1",
        common::AlgorithmIdentifier {
            oid: asn1::DefinedByMarker::marker(),
            params: common::AlgorithmParameters::Sha1(Some(())),
        },
    );
    h.insert(
        "sha224",
        common::AlgorithmIdentifier {
            oid: asn1::DefinedByMarker::marker(),
            params: common::AlgorithmParameters::Sha224(Some(())),
        },
    );
    h.insert(
        "sha256",
        common::AlgorithmIdentifier {
            oid: asn1::DefinedByMarker::marker(),
            params: common::AlgorithmParameters::Sha256(Some(())),
        },
    );
    h.insert(
        "sha384",
        common::AlgorithmIdentifier {
            oid: asn1::DefinedByMarker::marker(),
            params: common::AlgorithmParameters::Sha384(Some(())),
        },
    );
    h.insert(
        "sha512",
        common::AlgorithmIdentifier {
            oid: asn1::DefinedByMarker::marker(),
            params: common::AlgorithmParameters::Sha512(Some(())),
        },
    );
    h
});

pub(crate) fn certid_new<'p>(
    py: pyo3::Python<'p>,
    cert: &'p Certificate,
    issuer: &'p Certificate,
    hash_algorithm: &pyo3::Bound<'p, pyo3::PyAny>,
) -> CryptographyResult<CertID<'p>> {
    let issuer_der = asn1::write_single(&cert.raw.borrow_dependent().tbs_cert.issuer)?;
    let issuer_name_hash = hash_data(py, hash_algorithm, &issuer_der)?;
    let issuer_key_hash = hash_data(
        py,
        hash_algorithm,
        issuer
            .raw
            .borrow_dependent()
            .tbs_cert
            .spki
            .subject_public_key
            .as_bytes(),
    )?;

    Ok(CertID {
        hash_algorithm: HASH_NAME_TO_ALGORITHM_IDENTIFIERS[hash_algorithm
            .getattr(pyo3::intern!(py, "name"))?
            .extract::<&str>()?]
        .clone(),
        issuer_name_hash,
        issuer_key_hash,
        serial_number: cert.raw.borrow_dependent().tbs_cert.serial,
    })
}

pub(crate) fn certid_new_from_hash<'p>(
    py: pyo3::Python<'p>,
    issuer_name_hash: &'p [u8],
    issuer_key_hash: &'p [u8],
    serial_number: asn1::BigInt<'p>,
    hash_algorithm: &'p pyo3::PyAny,
) -> CryptographyResult<CertID<'p>> {
    Ok(CertID {
        hash_algorithm: HASH_NAME_TO_ALGORITHM_IDENTIFIERS[hash_algorithm
            .getattr(pyo3::intern!(py, "name"))?
            .extract::<&str>()?]
        .clone(),
        issuer_name_hash,
        issuer_key_hash,
        serial_number,
    })
}

pub(crate) fn hash_data<'p>(
    py: pyo3::Python<'p>,
    py_hash_alg: &pyo3::Bound<'p, pyo3::PyAny>,
    data: &[u8],
) -> pyo3::PyResult<&'p [u8]> {
    let mut h = Hash::new(py, py_hash_alg, None)?;
    h.update_bytes(data)?;
    Ok(h.finalize(py)?.into_gil_ref().as_bytes())
}
