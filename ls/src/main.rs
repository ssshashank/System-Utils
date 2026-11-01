use chrono::{DateTime, Local};
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, time::SystemTime};

// format time handler
fn format_system_time(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

// format size handler
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut size = bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{:.2} {}", size, UNITS[unit])
}

// format permission handler
fn format_permissions(mode: u32) -> String {
    // check bit mode
    let file_type = if mode & 0o170000 == 0o040000 {
        'd'
    } else if mode & 0o170000 == 0o120000 {
        'l'
    } else {
        '-'
    };

    // mapping symbols for each hex
    let symbols = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];

    let mut result = String::from(file_type);

    for (bit, symbol) in symbols {
        result.push(if mode & bit != 0 { symbol } else { '-' });
    }

    result
}

fn check_file_type(is_file: bool, is_dir: bool) -> &'static str {
    if is_file == true {
        "file"
    } else if is_dir == true {
        "dir"
    } else {
        ""
    }
}

fn main() {
    // flag list
    let flag_list = ["-a", "-l"];

    // collect unique flags
    let mut flags: HashSet<String> = HashSet::new();

    // listing all args
    let args: Vec<String> = env::args().collect();

    // default path
    let mut path: String = String::from(".");

    // flag to check if path has been updated or not
    let mut path_provided = false;

    dbg!("outside", &args);
    if args.len() > 1 {
        dbg!("in here");

        for arg in args.iter().skip(1) {
            if flag_list.contains(&arg.as_str()) {
                // collect all unique flags
                flags.insert(arg.to_string());
            } else if !path_provided {
                // if path is provided than ignore other paths

                path = arg.to_string();
                path_provided = true;
            } else {
                panic!("Unable to parse the arguments.")
            }
        }
    }

    // read the current directory
    let curr_dir_files = fs::read_dir(path);
    dbg!(flags);
    match curr_dir_files {
        Ok(list) => {
            let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
            for l in list {
                if let Ok(l) = l {
                    let name = l.file_name().into_string().unwrap();
                    let meta_data = l.metadata().unwrap();
                    let is_file = meta_data.file_type().is_file();
                    let is_dir = meta_data.file_type().is_dir();
                    let permission = format_permissions(meta_data.permissions().mode());
                    let modified_date = format_system_time(meta_data.modified().unwrap());
                    let size = format_size(meta_data.len());
                    let file_type = check_file_type(is_file, is_dir);

                    rows.push((name, file_type.to_string(), size, permission, modified_date));
                }
            }

            // sorting
            rows.sort_by(|a, b| a.0.cmp(&b.0));

            // calculate maximum width of each column
            let index_width = rows.len().to_string().len().max(1);

            let name_width = rows
                .iter()
                .map(|r| r.0.len())
                .max()
                .unwrap_or(0)
                .max("Name".len());

            let type_width = rows
                .iter()
                .map(|r| r.1.len())
                .max()
                .unwrap_or(0)
                .max("Type".len());

            let size_width = rows
                .iter()
                .map(|r| r.2.len())
                .max()
                .unwrap_or(0)
                .max("Size".len());

            let permission_width = rows
                .iter()
                .map(|r| r.3.len())
                .max()
                .unwrap_or(0)
                .max("Permission".len());

            let date_width = rows
                .iter()
                .map(|r| r.4.len())
                .max()
                .unwrap_or(0)
                .max("Modified Date".len());

            // render heading
            println!(
                "\x1b[1;33m{:<index_width$} | {:<name_width$} | {:<type_width$} | {:<size_width$} | {:<permission_width$} | {:<date_width$}\x1b[0m",
                "#", "Name", "Type", "Size", "Permission", "Modified Date"
            );

            // render rows
            for (index, (name, file_type, size, permission, modified_date)) in
                rows.iter().enumerate()
            {
                println!(
                    "{:<index_width$} | {:<name_width$} | {:<type_width$} | {:<size_width$} | {:<permission_width$} | {:<date_width$}",
                    index, name, file_type, size, permission, modified_date
                );
            }
        }
        Err(_e) => {
            println!("Path not found");
        }
    }
}
