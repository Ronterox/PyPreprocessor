use mlua::prelude::*;

const DEBUG: bool = false;
const DEPTH: usize = 5;
const OUTPUT_DIR: &str = "output";

const OPEN_COMMENT: &str = "\"\"\"";
const CLOSE_COMMENT: &str = "\"\"\"";
const LUA_CODE: &str = "%";

macro_rules! pprintln {
    ($($args: expr),*) => {
        println!(
            r#"
        ┌─────────────────────────────────────────────────────────────────────────────────┐
        │ ┌──────────────────────────────────────────────────────────────────────────────┐│
        │ │ {line}
        │ └──────────────────────────────────────────────────────────────────────────────┘│
        └─────────────────────────────────────────────────────────────────────────────────┘
        "#,
            line = format!("{}", $($args),*)
        )
    };
}

macro_rules! trace {
    ($($args: expr),*) => {
        $(pprintln!(format!("rust > {}: {}", stringify!($args), $args));)*
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

fn preprocess(
    filepath: &str,
    lua: &Lua,
    depth: usize,
    module: bool,
    overwrite: bool,
) -> LuaResult<()> {
    if module && std::fs::metadata(filepath).is_err() {
        return match filepath.split('/').last() {
            Some(filename) => {
                if DEBUG {
                    println!(
                        "Skipping '{}' module. Not a local file",
                        filename.replace(".py", "")
                    );
                }
                Ok(())
            }
            None => runtimeerror!("Unable to get filename"),
        };
    }

    if DEBUG {
        trace!(depth, filepath);
    }

    let file = maperror!(
        std::fs::read_to_string(filepath),
        "Unable to read file: {filepath}"
    );

    let open = format!("{OPEN_COMMENT}{}", LUA_CODE.repeat(depth));
    let close = format!("{}{CLOSE_COMMENT}", LUA_CODE.repeat(depth));
    let opening: Vec<usize> = file.match_indices(&open).map(|(i, _)| i).collect();
    let closing: Vec<usize> = file.match_indices(&close).map(|(i, _)| i).collect();

    let size = if !opening.is_empty() {
        open.chars().count()
    } else {
        0
    };

    let pairs = opening.iter().zip(closing.iter()).collect::<Vec<_>>();
    let mut file_content = String::new();
    let mut open_syntax: Option<String> = None;
    let mut body_pos = 0;

    let check_module = |name: &str| -> LuaResult<()> {
        let files = lua.globals().get::<_, mlua::Table>("files").unwrap();
        let key = format!("{depth}_{}", name.replace(".py", ""));

        if !files.contains_key(key.as_str()).unwrap() {
            files.set(key.as_str(), true).unwrap();
            preprocess(&name, lua, depth, true, overwrite)?
        }

        Ok(())
    };

    let result = pairs.iter().enumerate().try_for_each(|(i, (a, b))| {
        let code = &file[(*a + size)..**b];
        let body = &file[body_pos..**a];

        // TODO: Highlight the error line
        if DEBUG {
            println!("┌─────────────────────────────────────────────────────────────────────────────────┐");
            println!("│                                                                                 │");
            println!("│ [{filepath}:{i}] lua >");
            code.lines().for_each(|line| println!("│     {line}"));
            println!("│                                                                                 │");
            println!("└─────────────────────────────────────────────────────────────────────────────────┘");
        }

        if let Some(open) = &open_syntax {
            let code = format!("{open} return [[\n{body}]] {code}");
            let result = lua.load(&code).eval::<Option<String>>();
            match result {
                Ok(result) => {
                    if let Some(result) = result { file_content.push_str(&result); }
                    open_syntax = None;
                }
                Err(e) if e.to_string().contains("expected") => { open_syntax = Some(code.to_string()); }
                Err(e) => { Err(e)? }
            }
        } else if let Err(e) = lua.load(code).exec() {
            file_content.push_str(&body);
            open_syntax = Some(code.to_string());
            if !e.to_string().contains("expected") { Err(e)? }
        } else {
            file_content.push_str(&body);
        }

        body_pos = *b + size;

        Ok(())
    });

    if let Err(e) = result { return Err(e); }
    if let Some(open) = open_syntax {
        return runtimeerror!("Unclosed code block -> {open}");
    }

    file_content.push_str(&file[closing.last().unwrap_or(&&0) + size..]);

    let fullpath = if overwrite {
        filepath.to_string()
    } else {
        format!("{OUTPUT_DIR}/{filepath}")
    };

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
                        Some(parent) if !parent.as_os_str().is_empty() => {
                            format!("{}/{}", parent.to_str().unwrap(), words[1])
                        }
                        _ => words[1].to_string(),
                    }
                };

                if std::path::Path::new(&path).is_dir() {
                    std::fs::read_dir(path).unwrap().try_for_each(|entry| {
                        if let Ok(path) = entry {
                            match path.path().extension() {
                                Some(ext) if ext == "py" => {
                                    return check_module(&path.path().to_str().unwrap());
                                }
                                _ => {}
                            }
                        }
                        Ok(())
                    })
                } else {
                    check_module(format!("{path}.py").as_str())
                }
            }
            _ => Ok(()),
        }
    })
}

#[inline]
fn run_preprocessor(filepath: &str) -> LuaResult<()> {
    let lua = Lua::new();
    lua.globals().set("files", lua.create_table()?)?;

    preprocess(filepath, &lua, DEPTH, false, false)?;
    (1..DEPTH).rev().try_for_each(|depth| {
        preprocess(
            &format!("{OUTPUT_DIR}/{filepath}"),
            &lua,
            depth,
            false,
            true,
        )
    })
}

fn main() -> LuaResult<()> {
    let filepath = std::env::args().nth(1);

    if let Some(filepath) = filepath {
        pprintln!(format!("Parsing: {filepath}"));

        if let Err(e) = run_preprocessor(&filepath) {
            eprintln!("{e}");
        } else if DEBUG {
            pprintln!("Done. Now running with python3...");
            #[cfg(debug_assertions)]
            std::process::Command::new("python3")
                .arg(format!("{OUTPUT_DIR}/{filepath}"))
                .spawn()?;
        } else {
            pprintln!("Done.");
        }
    } else {
        eprintln!("Error: No filepath provided as parameter");
    }

    Ok(())
}
