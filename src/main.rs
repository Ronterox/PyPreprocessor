use mlua::prelude::*;

fn preprocess(filepath: &str, lua: &Lua) -> LuaResult<()> {
    if std::fs::metadata(filepath).is_err() {
        println!("File not found: {filepath}");
        return Ok(());
    }

    let file = std::fs::read_to_string(filepath).expect("Unable to read file");
    let opening = file
        .match_indices("\"\"\"%")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let closing = file
        .match_indices("%\"\"\"")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let size = 4;

    let pairs = opening.iter().zip(closing.iter()).collect::<Vec<_>>();

    let mut open_syntax = "";
    let mut body_pos = 0;

    let mut new_file = file[0..opening[0]].to_string();

    new_file.lines().for_each(|line| {
        let (imp, name) = line.split_once(' ').unwrap_or(("", ""));
        let files = lua.globals().get::<_, mlua::Table>("files").unwrap();

        if imp == "import" && !files.contains_key(name).unwrap() {
            let path = format!("{name}.py");
            preprocess(&path, lua).unwrap();
            files.set(name, true).unwrap();
        }
    });

    pairs.iter().enumerate().for_each(|(i, (a, b))| {
        let code = &file[(*a + size)..**b];
        println!("[{filepath}:{i}] lua > {code}");

        if open_syntax != "" {
            let body = &file[body_pos..**a];
            let code = format!("{open_syntax} return [[{body}]] {code}");
            if let Some(result) = lua.load(code).eval::<Option<String>>().unwrap() {
                new_file.push_str(&result);
            }
            open_syntax = "";
        } else if lua.load(code).exec().is_err() {
            open_syntax = code;
        } else {
            new_file.push_str(&file[body_pos..**a]);
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
    let files = lua.create_table()?;
    lua.globals().set("files", files)?;

    preprocess("python.py", &lua)?;
    std::process::Command::new("python3")
        .arg("output/python.py")
        .spawn()?;

    Ok(())
}
