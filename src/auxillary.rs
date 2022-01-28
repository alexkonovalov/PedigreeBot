use teloxide::{dispatching::{update_listeners::{self, StatefulListener}, stop_token::AsyncStopToken}, prelude::*, types::{Update, InlineKeyboardMarkup, InlineKeyboardButton} };
use teloxide_core::adaptors::AutoSend;
use std::{convert::Infallible, net::SocketAddr, process::{Command as ConsoleCommand, Stdio}};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use reqwest::{StatusCode, Url};
use crate::updater::model::{ButtonCommand, OutputAction};

#[derive(Debug)]
pub enum OutputCommand {
    Prompt(String),
    PromptButtons(Vec<(ButtonCommand, String)>, String)
}

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Webhook stateful listener
/// Copied from https://github.com/teloxide/teloxide/blob/85ef14867fb9b23d1a221ebc911dca458bec3291/examples/ngrok_ping_pong_bot/src/main.rs#L23-L58
pub async fn webhook(bot: AutoSend<Bot>, server_url: Url, socket_addr: SocketAddr) -> impl update_listeners::UpdateListener<Infallible> {
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

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S { &mut state.0 }
    
    StatefulListener::new((stream, stop_token), streamf, |state: &mut (_, AsyncStopToken)| state.1.clone())
}

pub fn print_graph(dot_graph: String) -> Vec<u8> {
    let mut piped_graph = ConsoleCommand::new("echo")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .arg(dot_graph)
        .spawn().unwrap();

    let _ = piped_graph.wait();
    let du_output = match piped_graph.stdout.take() {
        Some(it) => it,
        None => panic!("Error printing graph")
    };
    let png_img = ConsoleCommand::new("dot")
        .stdin(du_output)
        .arg("-Tpng")
        .output().unwrap();

    png_img.stdout
}

pub fn make_inline_keyboard(commands: &Vec<(ButtonCommand, String)>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for versions in commands.chunks(3) {
        let row = versions
            .iter()
            .map(|command| InlineKeyboardButton::callback(String::from(&command.1), command.0.to_string()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub fn map_next_action_output(action: &OutputAction) -> OutputCommand {
    return match action {
        OutputAction::AskFirstParent(description) => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "Don't know".to_string())
                ],
                format!("Write then name of the 1st parent of {}. If you don't know the name, press the button.", description)
            ),
        OutputAction::AskSecondParent(description) => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "Don't know".to_string())
                ],
                format!("Write then name of the 2nd parent of {}. If you don't know the name, press the button.", description)
            ),
        OutputAction::AskIfSiblings(description) => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No siblings".to_string())
                ],
                format!("Maybe {} has some siblings? Write the name of the first one that you know or press the button.", description)
            ),
        OutputAction::AskIfMoreSiblings(description) => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No more siblings".to_string())
                ],
                format!("Tell me the name of one more sibling of {} or press the button.", description)
            ),
        OutputAction::AskIfChildren(description) => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No children".to_string())
                ],
                format!("Tell me if {} has any children. If so, tell me the name. If none or you don't know, press the button.", description)
            ),
        OutputAction::AskIfMoreChildren(description) => 
            OutputCommand::PromptButtons(
                vec![
                    (ButtonCommand::No, "No".to_string())
                ],
                format!("Maybe {} has any other kids? If there's none, press the button. If you know someone, write the name.", description)
            ),
        OutputAction::NotifyError =>
            OutputCommand::Prompt(
                "Some error occured :( Please restart the bot!".to_string()
            ),
        OutputAction::NotifyComplete =>
            OutputCommand::Prompt(
                "We asked enough! you can get your pedigree chart by performing /finish command".to_string()
            ),
    }
}