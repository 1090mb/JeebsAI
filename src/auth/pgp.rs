use openpgp::cert::Cert;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use sequoia_openpgp as openpgp;
use std::io::Cursor;

pub fn validate_public_key(public_key: &str) -> Result<(), String> {
    Cert::from_bytes(public_key.as_bytes())
        .map(|_| ())
        .map_err(|e| format!("Invalid PGP public key: {e}"))
}

pub fn verify_signature_with_public_key(
    signed_message: &str,
    public_key: &str,
) -> Result<String, String> {
    let cert = Cert::from_bytes(public_key.as_bytes())
        .map_err(|e| format!("Invalid PGP public key: {e}"))?;
    let policy = StandardPolicy::new();

    let mut message_reader = Cursor::new(signed_message.as_bytes());
    let helper = Helper { cert };
    let mut verifier = openpgp::parse::stream::VerifierBuilder::from_reader(&mut message_reader)
        .map_err(|e| format!("Failed to create verifier: {e}"))?
        .with_policy(&policy, None, helper)
        .map_err(|e| format!("Failed to verify: {e}"))?;

    let mut verified_data = Vec::new();
    std::io::copy(&mut verifier, &mut verified_data)
        .map_err(|e| format!("Failed to read verified data: {e}"))?;

    String::from_utf8(verified_data)
        .map_err(|e| format!("Failed to convert verified data to UTF-8: {e}"))
}

struct Helper {
    cert: Cert,
}

impl openpgp::parse::stream::VerificationHelper for Helper {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> openpgp::Result<Vec<Cert>> {
        Ok(vec![self.cert.clone()])
    }

    fn check(
        &mut self,
        structure: openpgp::parse::stream::MessageStructure,
    ) -> openpgp::Result<()> {
        use std::io::{Error, ErrorKind};

        for layer in structure.iter() {
            if let openpgp::parse::stream::MessageLayer::SignatureGroup { results } = layer {
                for result in results {
                    if result.is_ok() {
                        return Ok(());
                    }
                }
                return Err(Error::new(ErrorKind::InvalidData, "Bad signature").into());
            }
        }

        Err(Error::new(ErrorKind::InvalidData, "No valid signature found").into())
    }
}

impl openpgp::parse::stream::DecryptionHelper for Helper {
    fn decrypt<D>(
        &mut self,
        _: &[openpgp::packet::PKESK],
        _: &[openpgp::packet::SKESK],
        _: Option<openpgp::types::SymmetricAlgorithm>,
        _: D,
    ) -> openpgp::Result<Option<openpgp::Fingerprint>>
    where
        D: FnMut(openpgp::types::SymmetricAlgorithm, &openpgp::crypto::SessionKey) -> bool,
    {
        Ok(None)
    }
}
