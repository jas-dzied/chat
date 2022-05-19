use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use url::Url;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use rand::{Rng, thread_rng};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ChatApp {
    message: String,
    messages: Arc<RwLock<Vec<String>>>,

    #[serde(skip)]
    socket: WebSocket<MaybeTlsStream<TcpStream>>
}

impl Default for ChatApp {
    fn default() -> Self {

        let uid = {
            let mut rng = thread_rng();
            let mut temp: i64 = 0;
            for i in 1..10 {
                temp += rng.gen_range(0..10)*10i64.pow(i);
            }
            temp
        };
        println!("{}", uid);

        let (mut reader_socket, _) = connect(Url::parse("ws://82.35.235.223:8080").unwrap()).expect("Can't connect");
        let (mut writer_socket, _) = connect(Url::parse("ws://82.35.235.223:8080").unwrap()).expect("Can't connect");

        reader_socket.write_message(Message::Text(format!("{}r", uid))).unwrap();
        writer_socket.write_message(Message::Text(format!("{}w", uid))).unwrap();

        let messages = Arc::new(RwLock::new(vec![]));
        let link = Arc::clone(&messages);

        thread::spawn(move || {
            loop {
                let msg = reader_socket.read_message().expect("Error reading message").into_text().unwrap();
                let mut message_list = link.write().unwrap();
                message_list.push(msg);
                drop(message_list);
                thread::sleep(Duration::from_millis(1000));
            }
        });

        Self {
            message: "".to_owned(),
            messages: Arc::clone(&messages),
            socket: writer_socket
        }

    }
}

impl ChatApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for ChatApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let Self { message, messages, socket } = self;

        egui::CentralPanel::default().show(&ctx, |ui| {
            match messages.try_read() {
                Ok(lock) => {
                    for message in lock.iter() {
                        ui.label(message);
                    }
                },
                Err(_) => {
                    ui.label("no lock?");
                }
            }
        });

        egui::TopBottomPanel::bottom("input_box")
            .min_height(45.0)
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("enter message: ");
                ui.text_edit_singleline(message);
                let button = egui::Button::new("Send");
                if ui.add_sized([45.0, 45.0], button).clicked() {
                    socket.write_message(Message::Text(message.to_string())).unwrap();
                    message.clear();
                }
            })
        });

    }
}
