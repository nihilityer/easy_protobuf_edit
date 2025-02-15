use std::fs::{remove_file, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, SerializeOptions};
use prost_types::FileDescriptorSet;
use serde_json::{Deserializer, Serializer};

pub fn handle_command(json_file: &String, data_file: &String, message_full_name: Option<&String>, encode_flag: bool, file_descriptor_set: &String) {
    let pool = match File::open(file_descriptor_set) {
        Ok(mut fds_file) => {
            let mut fds_buffer = Vec::new();
            if let Err(e) = fds_file.read_to_end(&mut fds_buffer) {
                eprintln!("Error reading file: {}", e);
                return;
            }
            match FileDescriptorSet::decode(Bytes::from(fds_buffer)) {
                Ok(file_descriptor_set) => {
                    match DescriptorPool::from_file_descriptor_set(file_descriptor_set) {
                        Ok(pool) => pool,
                        Err(e) => {
                            eprintln!("Create DescriptorPool Fail: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("FileDescriptorSet decode Fail: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            eprintln!("fds File Path Error: {:?}", e);
            return;
        }
    };
    match message_full_name {
        None => {
            println!("Message Full Name:\n");
            for message in pool.all_messages() {
                println!("{}", message.full_name())
            }
        }
        Some(full_name) => {
            for message in pool.all_messages() {
                if message.full_name() == full_name {
                    if !encode_flag {
                        match File::open(data_file) {
                            Ok(mut file) => {
                                let mut data_buffer = Vec::new();
                                if let Err(e) = file.read_to_end(&mut data_buffer) {
                                    eprintln!("Error reading file: {}", e);
                                    return;
                                }
                                if let Some(message_descriptor) =
                                    pool.get_message_by_name(message.full_name())
                                {
                                    match DynamicMessage::decode(
                                        message_descriptor,
                                        Bytes::from(data_buffer),
                                    ) {
                                        Ok(dynamic_message) => {
                                            let mut serializer = Serializer::pretty(vec![]);
                                            let options =
                                                SerializeOptions::new().skip_default_fields(false);
                                            if let Err(e) = dynamic_message
                                                .serialize_with_options(&mut serializer, &options)
                                            {
                                                eprintln!("serialize to json fail: {}", e);
                                                return;
                                            }
                                            let json_file_path = Path::new(json_file);
                                            if json_file_path.exists() {
                                                if let Err(e) = remove_file(json_file_path) {
                                                    eprintln!("Failed to remove JSON file: {}", e);
                                                    return;
                                                }
                                            }
                                            match File::create(json_file_path) {
                                                Ok(mut json) => {
                                                    match write!(
                                                        json,
                                                        "{}",
                                                        String::from_utf8(serializer.into_inner())
                                                            .unwrap()
                                                    ) {
                                                        Ok(_) => {}
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Error writing to json file fail: {}",
                                                                e
                                                            );
                                                            return ;
                                                        }
                                                    }
                                                    println!("write Json Data to {}", json_file);
                                                }
                                                Err(e) => {
                                                    eprintln!("open json file fail: {}", e);
                                                    return ;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("decode to DynamicMessage Fail: {}", e);
                                            return ;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Open Data File Error: {}", e);
                                return ;
                            }
                        }
                    } else {
                        match File::open(json_file) {
                            Ok(json) => {
                                let mut deserializer = Deserializer::from_reader(json);
                                if let Some(message_descriptor) =
                                    pool.get_message_by_name(message.full_name())
                                {
                                    match DynamicMessage::deserialize(
                                        message_descriptor,
                                        &mut deserializer,
                                    ) {
                                        Ok(dynamic_message) => {
                                            let data_file_path = Path::new(data_file);
                                            if data_file_path.exists() {
                                                if let Err(e) = remove_file(data_file_path) {
                                                    eprintln!("Failed to remove Data file: {}", e);
                                                    return ;
                                                }
                                            }
                                            match File::create(data_file_path) {
                                                Ok(file) => {
                                                    let mut writer = BufWriter::new(file);
                                                    let mut buf = Vec::new();
                                                    if let Err(e) = dynamic_message.encode(&mut buf)
                                                    {
                                                        eprintln!("encode to protobuf fail: {}", e);
                                                        return ;
                                                    }
                                                    if let Err(e) = writer.write_all(&buf) {
                                                        eprintln!(
                                                            "write to protobuf file fail: {}",
                                                            e
                                                        );
                                                        return ;
                                                    }
                                                    println!(
                                                        "Write protobuf data to {}",
                                                        data_file
                                                    );
                                                }
                                                Err(e) => {
                                                    eprintln!("create Data file fail: {}", e);
                                                    return ;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("deserialize DynamicMessage Fail: {}", e);
                                            return ;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("open json file fail: {}", e);
                                return ;
                            }
                        }
                    }
                }
            }
        }
    }
}
