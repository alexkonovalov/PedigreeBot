use teloxide::{dispatching::{update_listeners::{self, StatefulListener}, stop_token::AsyncStopToken}, prelude::*, types::Update };
use teloxide_core::adaptors::AutoSend;
use std::{convert::Infallible, net::SocketAddr, process::{Command as ConsoleCommand, Stdio}};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use reqwest::{StatusCode, Url};

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook(bot: AutoSend<Bot>, server_url: Url, socket_addr: SocketAddr) -> impl update_listeners::UpdateListener<Infallible> {
    
    // You might want to specify a self-signed certificate via .certificate
    // method on SetWebhook.
    bot.set_webhook(server_url)
        .await
        .expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            if let Ok(update) = Update::try_parse(&json) {
                tx.send(Ok(update)).expect("Cannot send an incoming update from the webhook")
            }

            StatusCode::OK
        })
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let server = warp::serve(server);
    let (_addr, fut) = server.bind_with_graceful_shutdown(socket_addr, stop_flag);

    // You might want to use serve.key_path/serve.cert_path methods here to
    // setup a self-signed TLS certificate.

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S { &mut state.0 }
    
    StatefulListener::new((stream, stop_token), streamf, |state: &mut (_, AsyncStopToken)| state.1.clone())
}


pub fn print_graph(dot_graph: String) -> Result<Vec<u8>, String> {
    let mut piped_graph = ConsoleCommand::new("echo")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .arg(dot_graph)
        .spawn().unwrap();

    let _ = piped_graph.wait();
    if let Some(du_output) = piped_graph.stdout.take() {
        let png_img = ConsoleCommand::new("dot")
            .stdin(du_output)
            .arg("-Tpng")
            .output().unwrap();

       return Ok(png_img.stdout);
    }

    Ok(vec![])
}