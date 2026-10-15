//! CI-only signature verification using the same library as the actual updater.
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};
use std::{env, fs};
fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 4,
        "Usage: verify PUBLIC_KEY SIGNATURE PACKAGE VERSION"
    );
    let public = String::from_utf8(STANDARD.decode(fs::read_to_string(&args[0])?.trim())?)?;
    let sig = String::from_utf8(STANDARD.decode(fs::read_to_string(&args[1])?.trim())?)?;
    let sig = Signature::decode(&sig)?;
    PublicKey::decode(&public)?.verify(&fs::read(&args[2])?, &sig, true)?;
    let version = sig
        .trusted_comment()
        .split('\t')
        .find_map(|field| field.strip_prefix("version:"))
        .context("No authenticated version in signature")?;
    ensure!(
        semver::Version::parse(version)? == semver::Version::parse(&args[3])?,
        "Signature version does not match the release"
    );
    println!("Update package signature and version verified");
    Ok(())
}
