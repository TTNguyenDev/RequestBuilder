use pcre2::bytes::{Regex, Captures};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use substring::Substring;
use std::str;

#[derive(Serialize, Deserialize, Debug)]
struct ContractFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<ContractParam>,
    pub fn_type: FunctionType, //READ || WRITE
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
    PAYABLE,
    INIT,
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

    // let mut scope_seperate_stack = vec![];

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

    println!("{:#?}", one_line_data);

    //Remove /**/
    let useful_code = one_line_data
        .split("/*")
        .map(|v| v.split("*/").last().unwrap().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    println!("\n\n\n");
    let re = Regex::new(r"{(?:[^{}]+|(?R))*}").unwrap();
    for result in re.captures_iter(useful_code.as_bytes()) {
        let caps = result.unwrap();

        let scope_str = str::from_utf8(caps.get(0).unwrap().as_bytes()).ok().unwrap();
        // println!("{}", scope_str);
        //Extract functions
        let fn_regex = Regex::new(r"pub fn [a-z_]+(\((?:`[()]|[^()]|(?1))*\))").unwrap();
        
        for fn_name in fn_regex.captures_iter(scope_str.as_bytes()) {
            println!("fn_name: {:#?}", fn_name);
        }
    }

    // for (index, line) in reader.lines().into_iter().enumerate() {
    //     let unwraped_data = line.unwrap();
    //     let trimmed_line= unwraped_data.trim();
    //
    //     // println!("stack {:#?} {:#?}", scope_seperate_stack, trimmed_line);
    //
    //     //NOTE: Handle comment
    //     if trimmed_line.starts_with("//") {
    //         continue;
    //     }
    //
    //     for c in trimmed_line.chars() {
    //
    //     }
    //
    //     if trimmed_line.contains("/*") {
    //         scope_seperate_stack.push("/*");
    //     }
    //
    //     if trimmed_line.contains("*/") && scope_seperate_stack.last().unwrap().eq(&"/*".to_string()) {
    //         scope_seperate_stack.pop();
    //         continue;
    //     }
    //
    //     if !scope_seperate_stack.is_empty() && scope_seperate_stack.last().unwrap().eq(&"/*".to_string()) {
    //         continue;
    //     }
    //     println!("Non comment code {:#?} {:?}", index, trimmed_line);
    //
    //
    //
    // }

    // let mut extracted_data: Vec<String> = vec![];
    // let mut holding_tag: Option<FunctionTag> = None;
    // for (_index, data) in reader.lines().into_iter().enumerate() {
    //     let unwraped_data = data.unwrap();
    //     let trimmed_data = unwraped_data.trim();
    //
    //     if is_pub_fn_parttern(trimmed_data.to_string()) {
    //         if let Some(parsed_data) = parse_contract_methods(trimmed_data.to_string(), holding_tag) {
    //             extracted_data.push(serde_json::to_string(&parsed_data).unwrap());
    //         }
    //     }
    //
    //     holding_tag = check_tag_line(trimmed_data.to_string());
    // }
    //
    // for i in extracted_data {
    //     println!("{}", i);
    // }

    // let json_object = serde_json::to_string(&contract_fn).unwrap();
    // println!("Contract-fn: {} \n\n", json_object);
    // contract_fn
}

fn check_tag_line(line: String) -> Option<FunctionTag> {
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

fn is_pub_fn_parttern(line: String) -> bool {
    //TODO: Handle this case /**/
    if line.contains("pub") && line.contains("fn") && !line.starts_with("//") {
        return true;
    }
    false
}

fn parse_contract_methods(line: String, tag: Option<FunctionTag>) -> Option<ContractFunction> {
    if let Some(tag) = tag {
        match tag {
            FunctionTag::Private => None,
            FunctionTag::Init | FunctionTag::Payable => {
                let skip = 1;

                let params_start = line.find('(').unwrap();
                let params_end = line.find(')').unwrap();

                let fn_index = line.find("fn").unwrap();
                let fn_name = line.substring(fn_index + "fn".len(), params_start).trim();

                let params = line.substring(params_start + 1, params_end);
                let params_list: Vec<&str> = params.split(',').collect();

                let fn_type = if tag == FunctionTag::Init {
                    FunctionType::INIT
                } else {
                    FunctionType::PAYABLE
                };

                Some(ContractFunction {
                    name: fn_name.to_string(),
                    return_type: "".to_string(),
                    params: parse_params(params_list, skip),
                    fn_type,
                })
            }
        }
    } else {
        let skip = if line.contains("self") {
            //Handle 3 cases: &mut self, self, &self
            1
        } else {
            0
        };

        let params_start = line.find('(').unwrap();
        let params_end = line.find(')').unwrap();

        let fn_index = line.find("fn").unwrap();
        let fn_name = line.substring(fn_index + "fn".len(), params_start).trim();

        let params = line.substring(params_start + 1, params_end);
        let params_list: Vec<&str> = params.split(',').collect();

        let fn_type = if line.contains("&mut self") {
            FunctionType::WRITE
        } else {
            FunctionType::READ
        };

        Some(ContractFunction {
            name: fn_name.to_string(),
            return_type: "".to_string(),
            params: parse_params(params_list, skip),
            fn_type,
        })
    }
}

fn parse_params(params_list: Vec<&str>, skip: usize) -> Vec<ContractParam> {
    let mut final_params = vec![];
    for param in params_list.iter().skip(skip) {
        let trimmed_param = param.trim();
        let single_param: Vec<&str> = trimmed_param.split(':').collect();

        final_params.push(ContractParam {
            name: single_param[0].trim().to_string(),
            param_type: single_param[1].trim().to_string(),
        });
    }
    final_params
}
