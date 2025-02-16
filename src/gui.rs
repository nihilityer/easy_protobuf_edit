use bytes::Bytes;
use iced::widget::{button, center, radio, text, text_editor, Column, Row};
use iced::{event, window, Center, Element, Event, Subscription, Task};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, SerializeOptions};
use prost_types::FileDescriptorSet;
use rfd::FileDialog;
use serde_json::{Deserializer, Serializer};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

pub fn run_app() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application("Easy Protobuf Edit", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Default)]
struct App {
    descriptor_pool: Option<DescriptorPool>,
    message_full_name_list: Vec<String>,
    message_full_name: Option<usize>,
    protobuf_data: Vec<u8>,
    content: text_editor::Content,
    json: String,
    state: AppState,
    message: Option<String>,
}

#[derive(Debug, Default)]
enum AppState {
    #[default]
    UploadFileDescriptorSet,
    Editor,
}

#[derive(Debug, Clone)]
enum AppMessage {
    EventOccurred(Event),
    Edit(text_editor::Action),
    RadioSelected(usize),
    Decode,
    Encode,
    SaveFile,
    OpenFile,
}

impl App {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::EventOccurred(event) => match event {
                Event::Window(window::Event::FileDropped(path)) => match self.state {
                    AppState::UploadFileDescriptorSet => match read_file_descriptor_set(path) {
                        Ok(pool) => {
                            self.message_full_name_list = pool
                                .all_messages()
                                .map(|m| m.full_name().to_string())
                                .collect();
                            self.descriptor_pool = Some(pool);
                            self.state = AppState::Editor;
                            self.message = Some("Drop Protobuf Data File Here".to_string());
                            Task::none()
                        }
                        Err(e) => {
                            self.message = Some(e.to_string());
                            Task::none()
                        }
                    },
                    AppState::Editor => match read_protobuf_data(&path) {
                        Ok(data) => {
                            self.protobuf_data = data;
                            self.message =
                                Some(format!("Protobuf Data Load Success: {}", path.display()));
                            Task::none()
                        }
                        Err(e) => {
                            self.message = Some(e.to_string());
                            Task::none()
                        }
                    },
                },
                _ => Task::none(),
            },
            AppMessage::Decode => {
                if let Some(message_full_name) = self.message_full_name {
                    if let Some(message_descriptor) =
                        self.descriptor_pool.clone().unwrap().get_message_by_name(
                            self.message_full_name_list[message_full_name].as_str(),
                        )
                    {
                        match DynamicMessage::decode(
                            message_descriptor,
                            Bytes::from(self.protobuf_data.clone()),
                        ) {
                            Ok(dynamic_message) => {
                                let mut serializer = Serializer::pretty(vec![]);
                                let options = SerializeOptions::new().skip_default_fields(false);
                                if let Err(e) = dynamic_message
                                    .serialize_with_options(&mut serializer, &options)
                                {
                                    self.message = Some(format!("serialize to json fail: {}", e));
                                    return Task::none();
                                }
                                self.json = String::from_utf8(serializer.into_inner()).unwrap();
                                self.content = text_editor::Content::with_text(self.json.as_str());
                                Task::none()
                            }
                            Err(e) => {
                                self.message =
                                    Some(format!("decode to DynamicMessage Fail: {}", e));
                                Task::none()
                            }
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    self.message = Some(String::from("Message full name not set"));
                    Task::none()
                }
            }
            AppMessage::Encode => {
                if let Some(message_full_name) = self.message_full_name {
                    if let Some(message_descriptor) =
                        self.descriptor_pool.clone().unwrap().get_message_by_name(
                            self.message_full_name_list[message_full_name].as_str(),
                        )
                    {
                        let mut deserializer = Deserializer::from_str(self.json.as_str());
                        match DynamicMessage::deserialize(message_descriptor, &mut deserializer) {
                            Ok(dynamic_message) => {
                                let mut buf = Vec::new();
                                if let Err(e) = dynamic_message.encode(&mut buf) {
                                    self.message = Some(format!("encode to protobuf fail: {}", e));
                                    return Task::none();
                                }
                                self.protobuf_data = buf;
                                self.message = Some("Protobuf Data Store".to_string());
                                Task::none()
                            }
                            Err(e) => {
                                self.message =
                                    Some(format!("deserialize DynamicMessage Fail: {}", e));
                                Task::none()
                            }
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    self.message = Some(String::from("Message full name not set"));
                    Task::none()
                }
            }
            AppMessage::Edit(action) => {
                self.content.perform(action);
                self.json = self.content.text();
                Task::none()
            }
            AppMessage::RadioSelected(name) => {
                self.message_full_name = Some(name);
                Task::none()
            }
            AppMessage::SaveFile => {
                let files = FileDialog::new().save_file();
                match files {
                    Some(path) => match File::create(path) {
                        Ok(file) => {
                            let mut writer = BufWriter::new(file);
                            if let Err(e) = writer.write_all(&self.protobuf_data) {
                                self.message = Some(format!("write to protobuf file fail: {}", e));
                            } else {
                                self.message = Some(String::from("Save file successful"));
                            }
                        }
                        Err(e) => {
                            self.message = Some(format!("create file fail: {}", e));
                        }
                    },
                    None => {
                        self.message = Some(String::from("File Path not set"));
                    }
                }
                Task::none()
            }
            AppMessage::OpenFile => {
                let files = FileDialog::new().pick_file();
                if let Some(path) = files {
                    match self.state {
                        AppState::UploadFileDescriptorSet => match read_file_descriptor_set(path) {
                            Ok(pool) => {
                                self.message_full_name_list = pool
                                    .all_messages()
                                    .map(|m| m.full_name().to_string())
                                    .collect();
                                self.descriptor_pool = Some(pool);
                                self.state = AppState::Editor;
                                self.message = Some("Drop Protobuf Data File Here".to_string());
                            }
                            Err(e) => {
                                self.message = Some(e.to_string());
                            }
                        },
                        AppState::Editor => match read_protobuf_data(&path) {
                            Ok(data) => {
                                self.protobuf_data = data;
                                self.message =
                                    Some(format!("Protobuf Data Load Success: {}", path.display()));
                                return Task::done(AppMessage::Decode);
                            }
                            Err(e) => {
                                self.message = Some(e.to_string());
                            }
                        },
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        event::listen().map(AppMessage::EventOccurred)
    }

    fn view(&self) -> Element<AppMessage> {
        match self.state {
            AppState::UploadFileDescriptorSet => {
                let mut content = Column::new()
                    .align_x(Center)
                    .spacing(20)
                    .push(text("Drop FileDescriptorSet File Here"))
                    .push(button("Open").on_press(AppMessage::OpenFile));
                if let Some(error_message) = self.message.clone() {
                    content = content.push(text(error_message));
                }
                center(content).into()
            }
            AppState::Editor => {
                let mut top = Row::new().align_y(Center).spacing(20);
                let mut message_type = Column::new().align_x(Center).spacing(20);
                for (index, message_full_name) in self.message_full_name_list.iter().enumerate() {
                    message_type = message_type.push(radio(
                        message_full_name,
                        index,
                        self.message_full_name,
                        AppMessage::RadioSelected,
                    ));
                }
                let mut buttons = Row::new().align_y(Center).spacing(20);
                buttons = buttons.push(button("Decode").on_press(AppMessage::Decode));
                buttons = buttons.push(button("Encode").on_press(AppMessage::Encode));
                buttons = buttons.push(button("Open").on_press(AppMessage::OpenFile));
                buttons = buttons.push(button("Save").on_press(AppMessage::SaveFile));
                top = top.push(message_type);
                top = top.push(buttons);
                let mut content = Column::new().align_x(Center).spacing(20);
                content = content.push(top);
                if let Some(error_message) = self.message.clone() {
                    content = content.push(text(error_message));
                }
                content = content.push(text_editor(&self.content).on_action(AppMessage::Edit));
                center(content).into()
            }
        }
    }
}

fn read_file_descriptor_set(path: PathBuf) -> Result<DescriptorPool, String> {
    match File::open(path) {
        Ok(mut fds_file) => {
            let mut fds_buffer = Vec::new();
            if let Err(e) = fds_file.read_to_end(&mut fds_buffer) {
                return Err(format!("Error reading file: {}", e));
            }
            match FileDescriptorSet::decode(Bytes::from(fds_buffer)) {
                Ok(file_descriptor_set) => {
                    match DescriptorPool::from_file_descriptor_set(file_descriptor_set) {
                        Ok(pool) => Ok(pool),
                        Err(e) => Err(format!("Create DescriptorPool Fail: {}", e)),
                    }
                }
                Err(e) => Err(format!("FileDescriptorSet decode Fail: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to open file descriptor set: {}", e)),
    }
}

fn read_protobuf_data(path: &PathBuf) -> Result<Vec<u8>, String> {
    match File::open(path) {
        Ok(mut fds_file) => {
            let mut fds_buffer = Vec::new();
            if let Err(e) = fds_file.read_to_end(&mut fds_buffer) {
                return Err(format!("Error reading file: {}", e));
            }
            Ok(fds_buffer)
        }
        Err(e) => Err(format!("Failed to open protobuf data file: {}", e)),
    }
}
