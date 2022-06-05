use pcre2::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str;
use substring::Substring;

#[derive(Serialize, Deserialize, Debug)]
struct ContractFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<ContractParam>,
    pub fn_type: String, //READ || WRITE
}

#[derive(Serialize, Deserialize, Debug)]
struct ContractParam {
    name: String,
    param_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum FunctionType {
    READ,
    WRITE,
    UNKNOWN
}

//TODO: Deal with near_bindgen tags:
// #[init] -> use for init
// #[private] -> Use for callback functions
// #[payable] -> WRITE + PAYABLE Methods

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum FunctionTag {
    Init,
    Private,
    Payable,
}

fn main() {
    //TODO: Scan contract code
    let filename = "./src/test_contracts/contract_lib.rs";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    //Remove comment  //
    let one_line_data = reader
        .lines()
        .into_iter()
        .filter_map(|v| {
            let unwrap_v = v.unwrap();
            let trimmed = unwrap_v.trim();

            if trimmed.starts_with("//") {
                None
            } else if trimmed.contains("//") {
                //Handle something like this let a = abc; //comment here
                let comment_pos = trimmed.find("//").unwrap();
                Some(trimmed.substring(0, comment_pos).trim().to_string())
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    //Remove /**/
    let useful_code = one_line_data
        .split("/*")
        .map(|v| v.split("*/").last().unwrap().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    let re = Regex::new(r"{(?:[^{}]+|(?R))*}").unwrap();
    let abi = re.captures_iter(useful_code.as_bytes()).flat_map(|v| {
        let caps = v.unwrap();

        let scope_str = str::from_utf8(caps.get(0).unwrap().as_bytes())
            .ok()
            .unwrap();

        //Extract functions
        let fn_regex = Regex::new(r"(?:#\[(?P<fn_macro>\w*)] )?(?:pub fn) (?P<fn_name>\w*)(?P<fn_params>\((?:`[()]|[^()]|(?1))*\)) (?:-> (?P<return_type>\w*))?").unwrap();
        let abi = fn_regex
            .captures_iter(scope_str.as_bytes())
            .map(|v| {
                let caps = v.unwrap();
                let full_fn = str::from_utf8(caps.get(0).unwrap().as_bytes())
                    .ok()
                    .unwrap();

                println!("\n\n\nfn {:#?}", full_fn);
                let name = str::from_utf8(&caps["fn_name"]).unwrap().to_string();
                // let fn_macro = str::from_utf8(&caps["fn_macro"]);
                let fn_params = str::from_utf8(&caps["fn_params"]).unwrap();

                let return_type_u8 = if let Some(rt) = caps.name("return_type") {
                    rt.as_bytes()
                } else {
                    "".as_bytes()
                };

                let macro_u8 = if let Some(m) = caps.name("fn_macro") {
                    m.as_bytes()
                } else {
                    "".as_bytes()
                };

                let return_type = str::from_utf8(return_type_u8).unwrap().to_string();
                let fn_macro = str::from_utf8(macro_u8).unwrap();

                let contract_fn = if fn_macro.is_empty() {
                 ContractFunction {
                    name,
                    return_type,
                    params: parse_params_(fn_params.to_string()).1,
                    fn_type: parse_params_(fn_params.to_string()).0 
                } 
                } else {
                 ContractFunction {
                    name,
                    return_type,
                    params: parse_params_(fn_params.to_string()).1,
                    fn_type: fn_macro.to_string()
                } 
                };

                println!("contract_fn: {:#?}", contract_fn);


                serde_json::to_string(&contract_fn).unwrap()
            })
            .collect::<Vec<String>>();
        abi
    }).collect::<Vec<String>>();
    

    //TODO: Save as abi file
}

fn _check_tag_line(line: String) -> Option<FunctionTag> {
    if line == "#[init]" {
        Some(FunctionTag::Init)
    } else if line == "#[payable]" {
        Some(FunctionTag::Payable)
    } else if line == "#[private]" {
        Some(FunctionTag::Private)
    } else {
        None
    }
}

fn parse_params_(params_string: String) -> (String, Vec<ContractParam>) {
    let params = params_string.substring(1, params_string.len() - 1);

    if params.is_empty() {
        ("".to_string(), vec![])
    } else {
        let mut fn_type = "".to_string();
        let param_pairs = params
            .split(',')
            .filter_map(|v| {
                let r = v.split(':').collect::<Vec<&str>>();
                if r.len() == 1 {
                    if r[0] == "&mut self" {
                        fn_type = "WRITE".to_string();
                    } else {
                        fn_type = "READ".to_string();
                    }
                    None
                } else {
                    Some(ContractParam {
                        name: r[0].trim().to_string(),
                        param_type: r[1].trim().to_string(),
                    })
                }
            })
            .collect::<Vec<ContractParam>>();
        (fn_type, param_pairs)
    }
}

