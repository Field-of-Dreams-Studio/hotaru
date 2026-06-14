//! Smoke test for issue #5 — server captures the client certificate chain
//! and exposes it through `TlsMeta::peer_certificates()`.
//!
//!放到 `hotaru_tls/tests/peer_cert.rs`。
//! 运行： `cargo test -p hotaru_tls --test peer_cert`

use std::io::Write;
use std::path::PathBuf;

use hotaru_core::connection::{ConnStream, Inbound, Outbound};
use hotaru_tls::{
    TlsClientConfig, TlsConfig, TlsInbound, TlsInboundTarget, TlsOutbound, TlsOutboundTarget,
};

/// 把一段 PEM 文本写进一个临时文件，返回路径。
/// （服务器的 require_client_auth / 客户端的 client_auth 只接受“文件路径”，
/// 所以测试里要先把证书落盘。）
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

    // ── 1. 造证书 ──────────────────────────────────────────────
    // CA：用来签发客户端证书，也是服务器“信任的根”
    let ca_key = KeyPair::generate().unwrap();
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();

    // 客户端证书：由上面的 CA 签名
    let client_key = KeyPair::generate().unwrap();
    let client_params = CertificateParams::new(vec!["client".to_string()]).unwrap();
    let client_cert = client_params
        .signed_by(&client_key, &ca_cert, &ca_key)
        .unwrap();

    // 服务器证书：自签，给 localhost 用即可（客户端会跳过校验）
    let server_key = KeyPair::generate().unwrap();
    let server_params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    let server_cert = server_params.self_signed(&server_key).unwrap();

    // ── 2. 把需要“按路径”读取的证书落盘 ─────────────────────────
    let ca_file = temp_file("ca.pem", &ca_cert.pem());
    let client_cert_file = temp_file("client.pem", &client_cert.pem());
    let client_key_file = temp_file("client.key", &client_key.serialize_pem());

    // ── 3. 服务器配置：要求客户端证书，信任我们的 CA ───────────
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

    // ── 4. 在后台任务里 accept + split，取出对端证书链长度 ──────
    let server_task = tokio::spawn(async move {
        let wire = server.accept().await.unwrap();
        let (_r, _w, meta) = wire.split();
        meta.peer_certificates().map(|chain| chain.len())
    });

    // ── 5. 客户端：出示自己的证书，跳过对服务器证书的校验 ───────
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

    // ── 6. 断言：服务器确实捕获到了客户端的叶子证书 ─────────────
    let chain_len = server_task.await.unwrap();
    assert_eq!(
        chain_len,
        Some(1),
        "服务器应通过 TlsMeta 捕获到客户端的叶子证书"
    );
}
