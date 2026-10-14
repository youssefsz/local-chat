fn main() -> anyhow::Result<()> {
    let evidence = local_chat::platform::network_evidence()?;
    println!(
        "interface={} gateway_hardware_present={} context_recognizable={} generation={}",
        evidence.interface,
        evidence.gateway_mac.is_some(),
        evidence.fingerprint().is_some(),
        local_chat::platform::network_generation()
    );
    Ok(())
}
