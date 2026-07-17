use flaredeck_lib::application::webhook_service;

#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["--version"] {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.len() != 4 {
        eprintln!("usage: flaredeck-webhook-proxy <listen> <route-id> <origin> <event-store>");
        std::process::exit(2);
    }
    if let Err(error) = webhook_service::serve(
        &args[0],
        args[1].clone(),
        args[2].clone(),
        std::path::PathBuf::from(&args[3]),
    )
    .await
    {
        eprintln!("webhook proxy failed: {error}");
        std::process::exit(1);
    }
}
