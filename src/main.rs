use mlua::prelude::*;

fn preprocess(filepath: &str, lua: &Lua) -> LuaResult<()> {
    if std::fs::metadata(filepath).is_err() {
        println!("File not found: {filepath}");
        return Ok(());
    }

    let file = std::fs::read_to_string(filepath).expect("Unable to read file");
    let opening: Vec<usize> = file.match_indices("\"\"\"%").map(|(i, _)| i).collect();
    let closing: Vec<usize> = file.match_indices("%\"\"\"").map(|(i, _)| i).collect();
    let size = 4;

    let pairs = opening.iter().zip(closing.iter()).collect::<Vec<_>>();

    let mut open_syntax = "";
    let mut body_pos = 0;

    let mut new_file = file[0..opening[0]].to_string();

    let check_module = |name: &str| {
        let files = lua.globals().get::<_, mlua::Table>("files").unwrap();

        if !files.contains_key(name).unwrap() {
            let path = format!("{name}.py");
            files.set(name, true).unwrap();
            preprocess(&path, lua).unwrap();
        }
    };

    file.lines().for_each(|line| {
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            return;
        }

        match words[0] {
            "import" | "from" => check_module(words[1]),
            _ => (),
        }
    });

    pairs.iter().enumerate().for_each(|(i, (a, b))| {
        let code = &file[(*a + size)..**b];
        let body = &file[body_pos..**a];

        println!("[{filepath}:{i}] lua > {code}");

        if open_syntax != "" {
            let code = format!("{open_syntax} return [[\n{body}]] {code}");
            if let Some(result) = lua.load(code).eval::<Option<String>>().unwrap() {
                new_file.push_str(&result);
            }
            open_syntax = "";
        } else if lua.load(code).exec().is_err() {
            new_file.push_str(&body);
            open_syntax = code;
        } else {
            new_file.push_str(&body);
        }

        body_pos = *b + size;
    });

    new_file.push_str(&file[closing.last().unwrap_or(&&0) + size..]);

    std::fs::create_dir_all("output").expect("Unable to create directory");
    std::fs::write(format!("output/{filepath}"), new_file).expect("Unable to write file");

    Ok(())
}

fn main() -> LuaResult<()> {
    let lua = Lua::new();
    lua.globals().set("files", lua.create_table()?)?;

    preprocess("python.py", &lua)?;
    std::process::Command::new("python3")
        .arg("output/python.py")
        .spawn()?;

    Ok(())
}
