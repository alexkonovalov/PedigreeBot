use std::sync::Arc;
use chashmap::CHashMap;

use teloxide::adaptors::AutoSend;
use teloxide::payloads::SendMessageSetters;
use teloxide::{ utils::command::BotCommand, prelude::*};
use tokio_stream::wrappers::UnboundedReceiverStream;
use std::{ net::SocketAddr, env };
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::{task, time}; 

use dotenv::dotenv;
use reqwest::Url;
use teloxide_core::types::InputFile;

use crate::auxillary::{make_inline_keyboard, map_next_action_output, OutputCommand};
use crate::updater::graph_updater::{ GraphUpdater };
use crate::updater::model::{ButtonCommand,InputAction};
mod updater;
mod auxillary;

#[tokio::main]
async fn main() {
    dotenv().ok();
    run().await;
}

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "List all commands")]
    Help,
    #[command(description = "Start/restart tree generation")]
    Start,
    #[command(description = "Print your family tree to the screen")]
    Finish,
}

struct Dialog {
    creation: Instant,
    graph_updater: GraphUpdater,
    last_msg_id: Option<i32>,
}

impl Dialog {
    fn new() -> Self { Self { creation: Instant::now(), graph_updater: GraphUpdater::new(), last_msg_id: None } }
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting bot...");

    let bot = Bot::from_env().auto_send();

    let cloned_bot = bot.clone();
    
    let url = env::var("SERVER_URL").expect("no server url in env");
    let ip = env::var("IP").expect("no IP in env");
    let port = env::var("PORT").expect("no PORT in env");
    let clear_session_interval = env::var("CLEAR_SESSION_MINUTES")
            .expect("no CLEAR_SESSION_MINUTES in env")
            .parse::<u32>()
            .expect("Error parsing CLEAR_SESSION_MINUTES");

    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>().unwrap();
    let url = Url::parse(&url).unwrap();

    let dialogs: CHashMap<String, Dialog> = CHashMap::new();
    let dialogs_rc = Arc::new(dialogs);

    let dialogs_text_message_rc= dialogs_rc.clone();
    let handle_text_message = move |rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
        UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| {
                let dialogs = dialogs_text_message_rc.clone();
                let text = String::from(cx.update.text().unwrap());
                let chat_id = cx.chat_id();

                async move {
                    match BotCommand::parse(&text, "PedigreeBot") {
                        Ok(Command::Help) => {
                            cx.answer(Command::descriptions()).await.log_on_error().await;
                        }
                        Ok(Command::Start) => {
                            dialogs
                                .insert(chat_id.to_string(), Dialog::new());

                            cx.answer("Let's start! Please add some person in your family tree or write your name").await.log_on_error().await;
                        }
                        Ok(Command::Finish) => {
                            let dialog = dialogs.get(&chat_id.to_string());

                            if let Some(dialog) = dialog {
                                let dot_graph = dialog.graph_updater.print_dot();
                                let graph = auxillary::print_graph(dot_graph);
                                let _ = cx.answer_photo(InputFile::Memory {
                                    file_name: "diagram.png".to_string(),
                                    data: std::borrow::Cow::Owned(graph)
                                }).await.log_on_error().await;
                            }
                        }
                        _ => {
                            let dialog = dialogs.get_mut(&chat_id.to_string());

                            if let Some(mut dialog) = dialog {
                                let output_action = dialog.graph_updater.handle_command(InputAction::Text(&text));
                                let output_command = map_next_action_output(&output_action);
                                let msg =  match output_command {
                                    OutputCommand::Prompt(a) => cx.answer(a),
                                    OutputCommand::PromptButtons(commands, promdpt) => cx.answer(promdpt).reply_markup(make_inline_keyboard(&commands)),
                                };
                                let msg_id = msg.await.unwrap().id;
                                dialog.last_msg_id = Some(msg_id);
                            }
                            
                        }
                    }
                }
            }
        )
    };

    let dialogs_query_rc = dialogs_rc.clone();
    let handle_query = move |rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
        UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |cx| {
                let UpdateWithCx { requester: bot, update: query } = cx;
                let dialogs = dialogs_query_rc.clone();
            
                async move {
                    if let Some(version) = query.data {
                        match query.message {
                            Some(Message { chat , id, ..}) => {
                                let input_str = &version;
                                let dialog = dialogs.get_mut(&chat.id.to_string());

                                if let Some(mut dialog) = dialog {
                                    //if user clicked on button of obsolete message
                                    if let Some(last_msg_id) = dialog.last_msg_id {
                                        if id != last_msg_id {
                                            //remove buttons from that message
                                            bot.edit_message_reply_markup(chat.id, id).await.unwrap();
                                        }
                                    }

                                    let button_command = ButtonCommand::from_str(input_str);
                                    let output = match button_command {
                                        Ok(ButtonCommand::No) => {
                                            let output_action = dialog.graph_updater.handle_command(InputAction::No);
                                            map_next_action_output(&output_action)
                                        }
                                        _ => OutputCommand::Prompt("Can't recognise the command".to_string())
                                    };
                                
                                    let _msg =  match output {
                                        OutputCommand::Prompt(a) => {
                                            bot.edit_message_reply_markup(chat.id, id).await.unwrap();
                                            let message_id = bot.send_message(chat.id, a).await.unwrap().id;
                                            dialog.last_msg_id = Some(message_id);
                                        }
                                        OutputCommand::PromptButtons(commands, prompt) => {
                                            bot.edit_message_reply_markup(chat.id, id).await.unwrap();
                                            bot.edit_message_reply_markup_inline(id.to_string());

                                            let message_id = bot.send_message(chat.id, prompt)
                                                .reply_markup(make_inline_keyboard(&commands))
                                                .await.unwrap().id;
                                            dialog.last_msg_id = Some(message_id);
                                        }
                                    };
                                }  
                            }
                            None => {
                            }
                        };
                    }
                    
                }
            }
        )
    };

    let dialogs_session_rc= dialogs_rc.clone();
    let _session_cleaner = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 60));
        loop {
            interval.tick().await;
            dialogs_session_rc.retain(|_, dialog| Instant::now().duration_since(dialog.creation).as_secs() < (60 * 60 * clear_session_interval).into());
        }
    });

    Dispatcher::new(bot)
        .messages_handler(handle_text_message)
        .callback_queries_handler(handle_query)
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            auxillary::webhook(cloned_bot, url, addr).await,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
