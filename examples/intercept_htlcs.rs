// This example connects to LND and uses router rpc to intercept htlcs, inspect the details, then resume forwarding them
//
// The program accepts three arguments: address, cert file, macaroon file
// The address must start with `https://`!
//
// Example run: `cargo run --features=routerrpc --example intercept_htlcs <address> <tls.cert> <file.macaroon>`

#[tokio::main]
#[cfg(feature = "routerrpc")]
async fn main() {
    use std::collections::HashMap;

    let mut args = std::env::args_os();
    args.next().expect("not even zeroth arg given");
    let address: String = args
        .next()
        .expect("missing arguments: address, macaroon file, payment hash")
        .into_string()
        .expect("address is not UTF-8");
    let cert_file: String = args
        .next()
        .expect("missing arguments: cert file, macaroon file")
        .into_string()
        .expect("cert_file is not UTF-8");
    let macaroon_file: String = args
        .next()
        .expect("missing argument: macaroon file")
        .into_string()
        .expect("macaroon_file is not UTF-8");

    // Connecting to LND requires only address, cert file, and macaroon file
    let mut client = voltage_tonic_lnd::Client::builder()
        .address(address)
        .cert_path(cert_file)
        .macaroon_path(macaroon_file)
        .build()
        .await
        .expect("failed to connect");

    let (tx, rx) = tokio::sync::mpsc::channel::<
        voltage_tonic_lnd::routerrpc::ForwardHtlcInterceptResponse,
    >(1024);
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    let mut htlc_stream = client
        .router()
        .htlc_interceptor(stream)
        .await
        .expect("Failed to call subscribe_invoices")
        .into_inner();

    while let Some(htlc) = htlc_stream.message().await.expect("Failed to receive invoices") {
        println!("HTLC Intercepted------------");
        println!(
            "incoming_circuit_key: {:?}\nincoming_amount_msat: {}\noutgoing_amount_msat: {}\npayment_hash: {:?}\n",
            htlc.incoming_circuit_key, htlc.incoming_amount_msat, htlc.outgoing_amount_msat, htlc.payment_hash
        );

        let response = voltage_tonic_lnd::routerrpc::ForwardHtlcInterceptResponse {
            incoming_circuit_key: htlc.incoming_circuit_key,
            action: 2, // Resume fordwarding of intercepted HTLC
            preimage: vec![],
            failure_message: vec![],
            failure_code: 0,
            in_amount_msat: htlc.incoming_amount_msat,
            out_amount_msat: htlc.outgoing_amount_msat,
            out_wire_custom_records: HashMap::new(),
        };
        tx.send(response).await.unwrap();
    }
}
