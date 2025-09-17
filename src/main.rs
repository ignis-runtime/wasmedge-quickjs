#![allow(dead_code, unused_imports, unused_must_use)]

use std::borrow::{Borrow, BorrowMut};
use wasmedge_quickjs::*;

fn args_parse() -> (Option<String>, String, Vec<String>) {
    use argparse::ArgumentParser;
    let mut file_path: Option<String> = None;
    let mut eval_code = String::new();
    let mut res_args: Vec<String> = vec![];
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Run JavaScript files or evaluate JavaScript code");
        ap.refer(&mut file_path)
            .add_argument("file", argparse::StoreOption, "js file");
        ap.refer(&mut eval_code)
            .add_option(&["-e", "--eval"], argparse::Store, "Evaluate JavaScript code");
        ap.refer(&mut res_args)
            .add_argument("arg", argparse::List, "arg");
        ap.parse_args_or_exit();
    }
    
    if !eval_code.is_empty() {
        (None, eval_code, res_args)
    } else {
        (file_path, String::new(), res_args)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    use wasmedge_quickjs as q;
    env_logger::init();

    let mut rt = q::Runtime::new();

    let r = rt
        .async_run_with_context(Box::new(|ctx| {
            let (file_path, eval_code, mut rest_arg) = args_parse();
            
            match file_path {
                Some(file_path) => {
                    // Execute file
                    let code = std::fs::read_to_string(&file_path);
                    match code {
                        Ok(code) => {
                            rest_arg.insert(0, file_path.clone());
                            ctx.put_args(rest_arg);
                            ctx.eval_buf(code.into_bytes(), &file_path, 1)
                        }
                        Err(e) => {
                            eprintln!("{}", e.to_string());
                            JsValue::UnDefined
                        }
                    }
                }
                None => {
                    // Execute inline code
                    if !eval_code.is_empty() {
                        ctx.put_args(rest_arg);
                        ctx.eval_buf(eval_code.into_bytes(), "<eval>", 1)
                    } else {
                        eprintln!("Error: No file or code provided");
                        JsValue::UnDefined
                    }
                }
            }
        }))
        .await;
    log::info!("{r:?}");
}
