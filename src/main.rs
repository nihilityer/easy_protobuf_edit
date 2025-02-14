use bytes::Bytes;
use clap::{Arg, ArgAction, ArgGroup, Command};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, SerializeOptions};
use prost_types::FileDescriptorSet;
use serde_json::{Deserializer, Serializer};
use std::fs::{File, remove_file};
use std::io::{BufWriter, Read, Write};
use std::path::Path;

fn main() {
    let matches = Command::new("EasyProtobufEdit")
        .version("1.0")
        .about("Make reading and editing Protobuf data simpler")
        .arg(
            Arg::new("file_descriptor_set")
                .long("file_descriptor_set")
                .short('f')
                .value_name("SET")
                .help("Input file descriptor set")
                .required(false),
        )
        .arg(
            Arg::new("proto_file")
                .long("proto_file")
                .short('p')
                .value_name("PROTO")
                .help("Input proto file")
                .required(false),
        )
        .arg(
            Arg::new("message_full_name")
                .long("message_full_name")
                .short('m')
                .value_name("MESSAGE_FULL_NAME")
                .help("Decode Message Type Full Name")
                .required(false),
        )
        .arg(
            Arg::new("data_file")
                .long("data_file")
                .short('d')
                .value_name("DATA_FILE")
                .help("Protobuf Data File")
                .required(true),
        )
        .arg(
            Arg::new("json_file")
                .short('j')
                .long("json_file")
                .value_name("JSON_FILE")
                .default_value("data.json")
                .help("Json file path"),
        )
        .arg(
            Arg::new("encode")
                .long("encode")
                .short('e')
                .help("Enable encoding")
                .action(ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("input_group")
                .args(["file_descriptor_set", "proto_file"])
                .required(true)
                .multiple(false),
        )
        .get_matches();

    let json_file = matches.get_one::<String>("json_file").unwrap();
    let data_file = matches.get_one::<String>("data_file").unwrap();
    let message_full_name = matches.get_one::<String>("message_full_name");
    let encode_flag = matches.get_flag("encode");

    let pool = if let Some(fds) = matches.get_one::<String>("file_descriptor_set") {
        match File::open(fds) {
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
        }
    } else if let Some(proto) = matches.get_one::<String>("proto_file") {
        match generate_fds(proto.to_string()) {
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
                eprintln!("Generate file descriptor set Fail: {}", e);
                return;
            }
        }
    } else {
        eprintln!("Missing File Descriptor Set");
        return;
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
                                                            return;
                                                        }
                                                    }
                                                    println!("write Json Data to {}", json_file);
                                                }
                                                Err(e) => {
                                                    eprintln!("open json file fail: {}", e);
                                                    return;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("decode to DynamicMessage Fail: {}", e);
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Open Data File Error: {}", e);
                                return;
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
                                                    return;
                                                }
                                            }
                                            match File::create(data_file_path) {
                                                Ok(file) => {
                                                    let mut writer = BufWriter::new(file);
                                                    let mut buf = Vec::new();
                                                    if let Err(e) = dynamic_message.encode(&mut buf)
                                                    {
                                                        eprintln!("encode to protobuf fail: {}", e);
                                                        return;
                                                    }
                                                    if let Err(e) = writer.write_all(&buf) {
                                                        eprintln!(
                                                            "write to protobuf file fail: {}",
                                                            e
                                                        );
                                                        return;
                                                    }
                                                    println!(
                                                        "Write protobuf data to {}",
                                                        data_file
                                                    );
                                                }
                                                Err(e) => {
                                                    eprintln!("create Data file fail: {}", e);
                                                    return;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("deserialize DynamicMessage Fail: {}", e);
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("open json file fail: {}", e);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn generate_fds(proto_file: String) -> Result<FileDescriptorSet, std::io::Error> {
    let status = std::process::Command::new("protoc")
        .arg("--proto_path")
        .arg(".")
        .arg("--include_imports")
        .arg("--descriptor_set_out")
        .arg("tmp.fds")
        .arg(proto_file)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::last_os_error());
    }
    let mut file = File::open("tmp.fds")?;
    let mut fds_buffer = Vec::new();
    file.read_to_end(&mut fds_buffer)?;
    Ok(FileDescriptorSet::decode(Bytes::from(fds_buffer))?)
}
