// build.rs

use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::string::String;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("vsvc_gen.rs");
    
    let mut output = String::new();
    
    let mut handlers: Vec<String> = Vec::new();
    let mut svcs: Vec<(String, String)> = Vec::new();
    
    if let Ok(lines) = read_lines("src/vm/vsvc.rs") {
        for line_try in lines {
            if let Ok(line) = line_try {
                if line.contains("impl") {
                    let split: Vec<&str> = line.split(' ').collect();
                    
                    if split[1] != "SvcHandler" { continue; }
                    
                    handlers.push(String::from(split[3]));
                }
            }
        }
    }
    
    let mut found_enum = false;
    if let Ok(lines) = read_lines("src/hos/svc.rs") {
        for line_try in lines {
            if let Ok(line) = line_try {
                if line.contains("enum HorizonSvc") {
                    found_enum = true;
                }
                if !found_enum { continue; }
                
                if line.contains("}") {
                    found_enum = false;
                }
                
                let split: Vec<&str> = line.split('(').collect();
                
                if split.len() < 2 { continue; }
                
                let name = split[0];
                
                let split2: Vec<&str> = split[1].split(')').collect();
                let handler = split2[0];
                
                let mut name_str = String::from(name);
                remove_whitespace(&mut name_str);
                
                if name_str.contains("//") || name_str.contains("/*") { continue; }
                
                svcs.push((name_str, String::from(handler)));
            }
        }
    }
    
    for _handler in &handlers
    {
        let handler = &_handler;
        output += "#[allow(non_snake_case)]\n";
        output += "async fn _svc_shim_";
        output += handler;
        output += "(ctx: [u64; 32]) -> [u64; 32] {
                   let handler = ";
        output += handler;
        output += ";
                   return handler.handle(ctx).await;
               }
            ";
    }
    
    output += "fn _svc_gen_pre(iss: u32, thread_ctx: u64, ctx: [u64; 32]) {
                   let svc = HorizonSvc::from_iss(iss);
                   let id = match svc {\n";

    for _svc in &svcs
    {
        let svc = &_svc.0;
        let handler = &_svc.1;
        output += "                       HorizonSvc::";
        output += svc;
        output += "(_) => {task_run_svc(thread_ctx, _svc_shim_";
        output += handler;
        output += "(ctx))},\n"
    }

    output += "    _ => {task_run_svc(thread_ctx, _svc_shim_SvcInvalid(ctx))},
                   };
               }
    ";
    
    
    fs::write(
        &dest_path,
        output
    ).unwrap();
    
    
    output = String::new();
    let dest_path = Path::new(&out_dir).join("svc_gen.rs");

    for _handler in &handlers
    {
        let handler = &_handler;
        output += "#[derive(Copy, Clone)]\n";
        output += "pub struct ";
        output += handler;
        output += ";\n\n";
    }
    fs::write(
        &dest_path,
        output
    ).unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/vm/vsvc.rs");
}
