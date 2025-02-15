use clap::{Arg, ArgAction, Command};
use easy_protobuf_edit::{handle_command, run_app};

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
                .required(false),
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
        .arg(
            Arg::new("gui")
                .long("gui")
                .short('g')
                .help("Open GUI To Edit")
                .action(ArgAction::SetFalse),
        )
        .get_matches();

    if matches.get_flag("gui") {
        if let Err(e) = run_app() {
            eprintln!("GUI Run Fail: {}", e);
        }
    } else {
        if matches.get_one::<String>("file_descriptor_set").is_none() {
            eprintln!("Please specify a file descriptor set.");
            return;
        }
        if matches.get_one::<String>("data_file").is_none() {
            eprintln!("Please specify a data file.");
            return;
        }
        let file_descriptor_set = matches.get_one::<String>("file_descriptor_set").unwrap();
        let json_file = matches.get_one::<String>("json_file").unwrap();
        let data_file = matches.get_one::<String>("data_file").unwrap();
        let message_full_name = matches.get_one::<String>("message_full_name");
        let encode_flag = matches.get_flag("encode");
        handle_command(
            json_file,
            data_file,
            message_full_name,
            encode_flag,
            file_descriptor_set,
        );
    }
}
