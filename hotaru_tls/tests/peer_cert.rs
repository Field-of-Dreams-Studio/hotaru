//! Smoke test for issue #5 — server captures the client certificate chain
//! and exposes it through `TlsMeta::peer_certificates()`.
//!


use std::io::Write;
use std::path::PathBuf;

use hotaru_core::connection::{ConnStream, Inbound, Outbound};
use hotaru_tls::{
    TlsClientConfig, TlsConfig, TlsInbound, TlsInboundTarget, TlsOutbound, TlsOutboundTarget,
};


fn temp_file(name: &str, contents: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("hotaru_tls_test_{}_{}", std::process::id(), name));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    path
}

#[tokio::test]
async fn server_captures_client_leaf_cert() {
    use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};

    // Create certificates for a mini PKI: a CA, a client cert signed by that CA, and a self-signed server cert.
    let ca_key = KeyPair::generate().unwrap();
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();

    //Client certificate: signed by our CA, with CN=client (CN is ignored by our code but good to have for realism)
    let client_key = KeyPair::generate().unwrap();
    let client_params = CertificateParams::new(vec!["client".to_string()]).unwrap();
    let client_cert = client_params
        .signed_by(&client_key, &ca_cert, &ca_key)
        .unwrap();

    // Server certificate: self-signed, for use with localhost (client will skip verification)
    let server_key = KeyPair::generate().unwrap();
    let server_params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    let server_cert = server_params.self_signed(&server_key).unwrap();

    // ── 2. Write the certificates to temporary files ─────────────────────────
    let ca_file = temp_file("ca.pem", &ca_cert.pem());
    let client_cert_file = temp_file("client.pem", &client_cert.pem());
    let client_key_file = temp_file("client.key", &client_key.serialize_pem());

    // ── 3. server config: require client auth, trusting our CA ───────────────────────
    let server_config = TlsConfig::builder()
        .cert_chain_pem(server_cert.pem().as_bytes())
        .unwrap()
        .private_key_pem(server_key.serialize_pem().as_bytes())
        .unwrap()
        .require_client_auth(&ca_file) // ← ClientAuth::Required
        .unwrap()
        .build()
        .unwrap();

    let addr = "127.0.0.1:34743";
    let server = TlsInbound::bind(TlsInboundTarget::new(addr, server_config))
        .await
        .unwrap();

    // ── 4. In the background task, accept + split, and extract the peer certificate chain length ──────
    let server_task = tokio::spawn(async move {
        let wire = server.accept().await.unwrap();
        let (_r, _w, meta) = wire.split();
        meta.peer_certificates().map(|chain| chain.len())
    });

    // ── 5. client config: present the client cert, but skip verification since we're using a self-signed server cert ───────
    let client_config = TlsClientConfig::builder()
        .client_auth(&client_cert_file, &client_key_file)
        .unwrap()
        .danger_disable_verification()
        .build()
        .unwrap();

    let client = TlsOutbound::build(TlsOutboundTarget::new("localhost", 34743, client_config))
        .await
        .unwrap();
    let _client_wire = client.connect().await.unwrap();

    // ── 6. Assert: the server indeed captured the client's leaf certificate ─────────────
    let chain_len = server_task.await.unwrap();
    assert_eq!(
        chain_len,
        Some(1),
        "server should have captured the client's leaf certificate, but got chain length {:?}",
    );
}
