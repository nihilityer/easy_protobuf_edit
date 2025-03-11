use bytes::Bytes;
use egui::{FontData, FontDefinitions, FontFamily};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, SerializeOptions};
use prost_types::FileDescriptorSet;
use rfd::AsyncFileDialog;
use rust_i18n::{set_locale, t};
use serde_json::{Deserializer, Serializer};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct EasyProtobufEditApp {
    locale: Locale,
    descriptor_pool: Option<DescriptorPool>,
    message_full_name_list: Vec<String>,
    message_full_name: usize,
    protobuf_data: Vec<u8>,
    read_file_sender: Sender<Vec<u8>>,
    save_file_sender: Sender<Option<String>>,
    read_file_receiver: Receiver<Vec<u8>>,
    save_file_receiver: Receiver<Option<String>>,
    json: String,
    state: AppState,
    message: Option<String>,
}

#[derive(Eq, PartialEq)]
enum Locale {
    Zh,
    En,
}

#[derive(Debug, Default)]
enum AppState {
    #[default]
    UploadFileDescriptorSet,
    Editor,
}

impl Default for EasyProtobufEditApp {
    fn default() -> Self {
        let (read_file_sender, read_file_receiver) = channel();
        let (save_file_sender, save_file_receiver) = channel();
        println!("{}", rust_i18n::locale().to_string());
        Self {
            locale: Locale::Zh,
            read_file_sender,
            read_file_receiver,
            save_file_sender,
            save_file_receiver,
            descriptor_pool: None,
            message_full_name_list: Vec::new(),
            message_full_name: 0,
            protobuf_data: Vec::new(),
            json: String::new(),
            state: AppState::UploadFileDescriptorSet,
            message: None,
        }
    }
}

impl EasyProtobufEditApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "SmileySans".to_owned(),
            std::sync::Arc::new(FontData::from_static(include_bytes!(
                "../assets/SmileySans-Oblique.ttf"
            ))),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "SmileySans".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("SmileySans".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_zoom_factor(1.5);
        Default::default()
    }

    fn decode(&mut self) {
        if let Some(message_descriptor) = self
            .descriptor_pool
            .clone()
            .unwrap()
            .get_message_by_name(self.message_full_name_list[self.message_full_name].as_str())
        {
            match DynamicMessage::decode(
                message_descriptor,
                Bytes::from(self.protobuf_data.clone()),
            ) {
                Ok(dynamic_message) => {
                    let mut serializer = Serializer::pretty(vec![]);
                    let options = SerializeOptions::new().skip_default_fields(false);
                    if let Err(e) =
                        dynamic_message.serialize_with_options(&mut serializer, &options)
                    {
                        self.message = Some(format!("{}{}", t!("serialize2json_fail"), e));
                    } else {
                        self.json = String::from_utf8(serializer.into_inner()).unwrap();
                        self.message = Some(t!("decode_success").to_string());
                    }
                }
                Err(e) => {
                    self.message = Some(format!("{}{}", t!("decode2dynamic_message_fail"), e));
                }
            }
        }
    }

    fn encode(&mut self) {
        if let Some(message_descriptor) = self
            .descriptor_pool
            .clone()
            .unwrap()
            .get_message_by_name(self.message_full_name_list[self.message_full_name].as_str())
        {
            let mut deserializer = Deserializer::from_str(self.json.as_str());
            match DynamicMessage::deserialize(message_descriptor, &mut deserializer) {
                Ok(dynamic_message) => {
                    let mut buf = Vec::new();
                    if let Err(e) = dynamic_message.encode(&mut buf) {
                        self.message = Some(format!("{}{}", t!("encode2protobuf_fail"), e));
                    } else {
                        self.protobuf_data = buf;
                        self.message = Some(t!("protobuf_data_store").to_string());
                    }
                }
                Err(e) => {
                    self.message = Some(format!("{}{}", t!("deserialize_dynamic_message_fail"), e));
                }
            }
        }
    }
}

impl eframe::App for EasyProtobufEditApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.locale {
            Locale::Zh => {
                set_locale("zh-CN");
            }
            Locale::En => {
                set_locale("en");
            }
        }
        let window_height = ctx.screen_rect().height();
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::UploadFileDescriptorSet => ui.vertical_centered_justified(|ui| {
                    ui.add_space(window_height / 5.0 * 2.0);
                    egui::ComboBox::from_id_salt("locales selected")
                        .selected_text(match self.locale {
                            Locale::Zh => "中文".to_string(),
                            Locale::En => "English".to_string(),
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.locale, Locale::Zh, "中文");
                            ui.selectable_value(&mut self.locale, Locale::En, "English");
                        });
                    ui.label(t!("tips_upload_file_descriptor_set"));
                    if let Some(message) = self.message.clone() {
                        ui.label(message);
                    }
                    if ui.button(t!("upload")).clicked() {
                        let task = AsyncFileDialog::new()
                            .set_title(t!("upload_file_descriptor_set_dialog_title"))
                            .pick_file();
                        let sender = self.read_file_sender.clone();
                        let future = async move {
                            let file = task.await;
                            if let Some(file_handle) = file {
                                sender.send(file_handle.read().await).unwrap();
                            }
                        };
                        execute(Box::pin(future));
                    }
                }),
                AppState::Editor => {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if !self.message_full_name_list.is_empty() {
                                egui::ComboBox::from_label(t!("full_message_name"))
                                    .selected_text(
                                        self.message_full_name_list[self.message_full_name]
                                            .to_string(),
                                    )
                                    .show_ui(ui, |ui| {
                                        for (index, name) in
                                            self.message_full_name_list.iter().enumerate()
                                        {
                                            ui.selectable_value(
                                                &mut self.message_full_name,
                                                index,
                                                name,
                                            );
                                        }
                                    });
                            }
                            if ui.button(t!("previous")).clicked() {
                                self.descriptor_pool = None;
                                self.state = AppState::UploadFileDescriptorSet;
                                self.message_full_name_list = Vec::new();
                                self.message_full_name = 0;
                                self.json = String::new();
                                self.protobuf_data = Vec::new();
                                self.message = None;
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button(t!("upload")).clicked() {
                                let task = AsyncFileDialog::new()
                                    .set_title(t!("upload_protobuf_data_dialog_title"))
                                    .pick_file();
                                let sender = self.read_file_sender.clone();
                                let future = async move {
                                    let file = task.await;
                                    if let Some(file_handle) = file {
                                        sender.send(file_handle.read().await).unwrap();
                                    }
                                };
                                execute(Box::pin(future));
                            }
                            if ui.button(t!("save")).clicked() {
                                self.encode();
                                let task = AsyncFileDialog::new()
                                    .set_title(t!("save_protobuf_data_dialog_title"))
                                    .save_file();
                                let data = self.protobuf_data.clone();
                                let sender = self.save_file_sender.clone();
                                let future = async move {
                                    let file = task.await;
                                    let message = if let Some(file_handle) = file {
                                        match file_handle.write(data.as_slice()).await {
                                            Ok(_) => Some(String::from(t!("save_success"))),
                                            Err(e) => Some(format!(
                                                "{}{}",
                                                t!("write_protobuf_file_fail"),
                                                e
                                            )),
                                        }
                                    } else {
                                        Some(t!("write_file_not_exist").to_string())
                                    };
                                    sender.send(message).unwrap();
                                };
                                execute(Box::pin(future));
                            }
                            if ui.button(t!("decode")).clicked() {
                                self.decode();
                            }
                            if ui.button(t!("encode")).clicked() {
                                self.encode();
                            }
                        });
                        if let Some(message) = self.message.clone() {
                            ui.label(message);
                        }
                        let theme = egui_extras::syntax_highlighting::CodeTheme::light(40.0);
                        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                                ui.ctx(),
                                ui.style(),
                                &theme,
                                string,
                                "json",
                            );
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts(|f| f.layout_job(layout_job))
                        };
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.json)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .code_editor()
                                    .desired_rows(10)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut layouter),
                            );
                        });
                    })
                }
            }
        });
        if let Ok(data) = &self.read_file_receiver.try_recv() {
            match self.state {
                AppState::UploadFileDescriptorSet => {
                    match FileDescriptorSet::decode(Bytes::from(data.clone())) {
                        Ok(file_descriptor_set) => {
                            match DescriptorPool::from_file_descriptor_set(file_descriptor_set) {
                                Ok(pool) => {
                                    self.message_full_name_list = pool
                                        .all_messages()
                                        .map(|x| x.full_name().to_string())
                                        .collect();
                                    self.descriptor_pool = Some(pool);
                                    self.state = AppState::Editor;
                                }
                                Err(e) => {
                                    self.message =
                                        Some(format!("{}{}", t!("create_descriptor_pool_fail"), e));
                                }
                            }
                        }
                        Err(e) => {
                            self.message =
                                Some(format!("{}{}", t!("decode_descriptor_set_fail"), e));
                        }
                    }
                }
                AppState::Editor => {
                    self.protobuf_data = data.clone();
                    self.decode();
                    self.message = Some(t!("open_protobuf_data_file_success").to_string());
                }
            }
        }
        if let Ok(message) = &self.save_file_receiver.try_recv() {
            self.message = message.clone();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    tokio::spawn(f);
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
