use mlua::prelude::*;

const DEBUG: bool = true;
const OPEN_CODE: &str = "\"\"\"%";
const CLOSE_CODE: &str = "%\"\"\"";

macro_rules! trace {
    ($($args: expr),*) => {
        print!("[{}:{}]", file!(), line!());
        $(print!(" rust > {}: {}", stringify!($args), $args);)*
        println!(""); // to get a new line at the end
    }
}

macro_rules! runtimeerror {
    ($m:expr) => {
        Err(mlua::Error::RuntimeError(format!($m)))
    };
}

macro_rules! maperror {
    ($e:expr, $m:expr) => {
        match $e {
            Ok(v) => v,
            Err(_) => return runtimeerror!($m),
        }
    };
}

fn preprocess(filepath: &str, lua: &Lua, module: bool) -> LuaResult<()> {
    if module && std::fs::metadata(filepath).is_err() {
        return match filepath.split('/').last() {
            Some(filename) => {
                println!(
                    "Skipping '{}' module. Not a local file",
                    filename.replace(".py", "")
                );
                Ok(())
            }
            None => runtimeerror!("Unable to get filename"),
        };
    }

    if DEBUG {
        trace!(filepath);
    }

    let file = maperror!(
        std::fs::read_to_string(filepath),
        "Unable to read file: {filepath}"
    );

    let opening: Vec<usize> = file.match_indices(OPEN_CODE).map(|(i, _)| i).collect();
    let closing: Vec<usize> = file.match_indices(CLOSE_CODE).map(|(i, _)| i).collect();

    let size = OPEN_CODE.chars().count();
    let pairs = opening.iter().zip(closing.iter()).collect::<Vec<_>>();

    let mut open_syntax = "";
    let mut body_pos = 0;

    let mut file_content = file[0..opening[0]].to_string();

    let check_module = |name: &str| -> LuaResult<()> {
        let files = lua.globals().get::<_, mlua::Table>("files").unwrap();

        if !files.contains_key(name).unwrap() {
            let path = format!("{name}.py");
            files.set(name, true).unwrap();
            preprocess(&path, lua, true)?
        }

        Ok(())
    };

    pairs.iter().enumerate().for_each(|(i, (a, b))| {
        let code = &file[(*a + size)..**b];
        let body = &file[body_pos..**a];

        if DEBUG {
            println!("[{filepath}:{i}] lua > {code}");
        }

        if open_syntax != "" {
            let code = format!("{open_syntax} return [[\n{body}]] {code}");
            if let Some(result) = lua.load(code).eval::<Option<String>>().unwrap() {
                file_content.push_str(&result);
            }
            open_syntax = "";
        } else if lua.load(code).exec().is_err() {
            file_content.push_str(&body);
            open_syntax = code;
        } else {
            file_content.push_str(&body);
        }

        body_pos = *b + size;
    });
    file_content.push_str(&file[closing.last().unwrap_or(&&0) + size..]);

    let fullpath = format!("output/{filepath}");
    if let Some(parent) = std::path::Path::new(&fullpath).parent() {
        maperror!(
            std::fs::create_dir_all(parent),
            "Unable to create directory: {fullpath}"
        );
    } else {
        return runtimeerror!("Unable to get parent: {fullpath}");
    }

    maperror!(
        std::fs::write(&fullpath, file_content),
        "Unable to write file: {fullpath}"
    );

    file.lines().try_for_each(|line| -> LuaResult<()> {
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            return Ok(());
        }

        match words[0] {
            "import" | "from" => {
                let path = if words[1].contains(".") {
                    words[1].replace(".", "/")
                } else {
                    match std::path::Path::new(&filepath).parent() {
                        Some(parent) => format!("{}/{}", parent.to_str().unwrap(), words[1]),
                        None => words[1].to_string(),
                    }
                };
                check_module(&path)
            }
            _ => Ok(()),
        }
    })
}

#[inline]
fn run_preprocessor(filepath: &str) -> LuaResult<()> {
    let lua = Lua::new();
    lua.globals().set("files", lua.create_table()?)?;
    preprocess(filepath, &lua, false)
}

fn main() -> LuaResult<()> {
    let filepath = std::env::args().nth(1);
    if let Some(filepath) = filepath {
        if let Err(e) = run_preprocessor(&filepath) {
            eprintln!("{e}");
        } else {
            #[cfg(debug_assertions)]
            std::process::Command::new("python3")
                .arg(format!("output/{filepath}"))
                .spawn()?;
        }
    } else {
        eprintln!("Error: No filepath provided as parameter");
    }

    Ok(())
}
