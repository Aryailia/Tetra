// This is the set of functions that probably all custom flavours of this
// templating markup language can make use of. The bare bones of this language
// is this file and the executor.rs

//run: cargo test -- --nocapture

use std::io::Write;
use std::process;
use std::process::Stdio;

use super::{MyError, PureResult, Value};

pub fn code<'a, V>(args: &[Value<'a, V>]) -> PureResult<'a, V> {
    if args.len() > 2 {
        //println!("len {}", args.len());
        todo!("temp panic for when we put actual error handling");
    }

    //let lang = unwrap!(or_invalid args[0] => String(x) | Str(x) => x);
    let lang: &str = match &args[0] {
        Value::String(x) => x,
        Value::Str(x) => x,
        _ => return Err("Invalid type"),
    };
    let cell_body: &str = match &args[1] {
        Value::String(x) => x,
        Value::Str(x) => x,
        _ => return Err("Invalid type"),
    };

    match lang {
        "r" => {
            println!("markup.rs: Running r");
        }
        "graphviz" | "dot" => {
            return run_command("dot", Some(cell_body), &["-Tsvg"]).map(Value::String)
        }
        "sh" => println!("markup.rs: Running shell"),
        s => todo!("markup.rs: {}", s),
    }

    Ok(Value::String("".to_string()))
}
/******************************************************************************
 * In-built Commands
 ******************************************************************************/

////////////////////////////////////////////////////////////////////////////////
// Concat

// Just joins its arguments into a string
// Also doubles as the default push to the final knit
pub fn concat<'a, V>(args: &[Value<'a, V>]) -> PureResult<'a, V> {
    let mut buffer = String::with_capacity(recursive_calc_length(args)?);
    recursive_concat::<V>(args, &mut buffer);
    Ok(Value::String(buffer))
}

fn recursive_calc_length<V>(args: &[Value<V>]) -> Result<usize, MyError> {
    let mut sum = 0;
    for a in args {
        sum += match a {
            Value::Null => return Err("You left a null unprocessed"),
            Value::Str(s) => s.len(),
            Value::Char(c) => c.len_utf8(),
            Value::Usize(x) => x.to_string().len(),
            Value::String(s) => s.len(),
            Value::Bool(b) => b.then(|| "true").unwrap_or("false").len(),
            Value::List(l) => recursive_calc_length(l)?,
            Value::Custom(_) => todo!(),
        };
    }
    Ok(sum)
}

fn recursive_concat<'a, V>(args: &[Value<'a, V>], buffer: &mut String) {
    for arg in args {
        match arg {
            Value::Null => unreachable!(),
            Value::Str(s) => buffer.push_str(s),
            Value::Char(c) => buffer.push(*c),
            Value::Usize(x) => buffer.push_str(&x.to_string()),
            Value::String(s) => buffer.push_str(s.as_str()),
            Value::Bool(b) => buffer.push_str(b.then(|| "true").unwrap_or("false")),
            Value::List(l) => recursive_concat(l, buffer),
            Value::Custom(_) => todo!(),
        };
    }
}

////////////////////////////////////////////////////////////////////////////////
// env
pub fn env<'a, V>(args: &[Value<'a, V>]) -> PureResult<'a, V> {
    let name: &str = match &args[0] {
        Value::Str(s) => s,
        Value::String(s) => s,
        _ => return Err("Invalid type, expecting string"),
    };
    fetch_env_var(name).map(Value::String)
}

/******************************************************************************
 * Helpers
 ******************************************************************************/

pub fn run_command(program: &str, stdin: Option<&str>, args: &[&str]) -> Result<String, MyError> {
    let child = if let Some(s) = stdin {
        let mut child = process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(s.as_bytes()).unwrap();
        child
    } else {
        process::Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
    };
    let output = child.wait_with_output().unwrap();

    let out = String::from_utf8(output.stdout).unwrap();
    if output.status.success() {
        Ok(out)
    } else {
        panic!()
        //Err(CustomErr::NonZeroStatus(
        //    output.status.code().unwrap_or(1),
        //    out,
        //))
    }
}

pub fn fetch_env_var(key: &str) -> Result<String, MyError> {
    Ok(std::env::vars()
        .find(|(k, _)| k == key)
        .ok_or("Missing BIBLIOGRAPHY environment variable")?
        .1)
}
