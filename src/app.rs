use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use url::Url;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use rand::{Rng, thread_rng};
use egui::RichText;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ChatApp {
    message: String,
    username: String,
    messages: Arc<RwLock<Vec<String>>>,

    #[serde(skip)]
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    #[serde(skip)]
    mode: i32
}

impl Default for ChatApp {
    fn default() -> Self {

        let uid = {
            let mut rng = thread_rng();
            let mut temp: i64 = 0;
            for i in 1..10 {
                temp += rng.gen_range(1..10)*10i64.pow(i);
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
            username: "".to_owned(),
            messages: Arc::clone(&messages),
            socket: writer_socket,
            mode: 0
        }

    }
}

impl ChatApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert("hack".to_owned(),
            egui::FontData::from_static(include_bytes!("../Hack Regular Nerd Font Complete Mono.ttf")));

        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(0, "hack".to_owned());

        cc.egui_ctx.set_fonts(fonts);
        Default::default()
    }
}

macro_rules! hack_text {
    ( $template:expr, $( $x:expr ),* ) => {
        RichText::new(format!(
            $template, $($x),*
        )).font(egui::FontId::monospace(15.0))
    };
    ( $template:expr ) => {
        RichText::new($template).font(egui::FontId::monospace(15.0))
    }
}

fn multiply(string: &str, count: usize) -> String {
    let mut result = String::new();
    for _ in 0..count {
        result.push_str(string);
    }
    result
}

impl eframe::App for ChatApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let Self { message, username, messages, socket, mode } = self;

        match mode {
            0 => {

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("enter username: ");
            ui.text_edit_singleline(username);
            if ui.button("Join").clicked() {
                socket.write_message(Message::Text(username.to_string())).unwrap();
                *mode += 1;
            }
        });

            },
            1 => {

        egui::TopBottomPanel::bottom("input_box")
            .min_height(45.0)
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("enter message: ");
                ui.text_edit_multiline(message);
                let button = egui::Button::new("Send");
                if
                    ui.add_sized([45.0, 45.0], button).clicked() ||
                    (
                        ctx.input().keys_down.contains(&egui::Key::Enter) &&
                        !ctx.input().modifiers.matches(egui::Modifiers::SHIFT) &&
                        (*message).trim() != ""
                    )
                {
                    socket.write_message(Message::Text(message.trim().to_string())).unwrap();
                    message.clear();
                }
            })
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {

            match messages.try_read() {
                Ok(lock) => {


                    for message in lock.iter() {

                        let parts: Vec<&str> = message.split('|').collect();
                        let sender_str = base64::decode(parts[0]).unwrap();
                        let sender = String::from_utf8(sender_str).unwrap();
                        let message_str = base64::decode(parts[1]).unwrap();
                        let message = String::from_utf8(message_str).unwrap();

                        let mut longest = sender.len();
                        for line in message.split('\n') {
                            if line.len() > longest {
                                longest = line.len();
                            }
                        }

                        ui.label(hack_text!("╭─{}{}─╮", sender, "─".repeat(longest-sender.len())));
                        for line in message.split('\n') {
                            ui.label(hack_text!("│ {:longest$} │", line));
                        }
                        ui.label(hack_text!("╰{}╯", "─".repeat(longest+2)));

                    }
                },
                Err(_) => {
                    ui.label("no lock?");
                }
            }

            });
        });

            },
            _ => {}
        }



    }
}
