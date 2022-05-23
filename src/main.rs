use std::fs::File;
use std::io::{BufRead, BufReader};
use serde::{Serialize, Deserialize};
use substring::Substring;

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
    UNKNOWN
}

//TODO: Deal with near_bindgen tags:
// #[init] -> use for init
// #[private] -> Use for callback functions
// #[payable] -> WRITE + PAYABLE Methods


fn main() {
    //TODO: Get list methods name from near-contract-parse
    let methods_name = ["set_greeting", "greeting", "get_greeting"];

    //TODO: Scan contract code
    let filename = "./src/test_contracts/contract_lib.rs";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut is_payable = false;
    let extracted_data: Vec<String> = reader.lines().into_iter().filter_map(|v| {
            let v_unwrap = v.unwrap().trim().to_string();
            if is_payable_function(v_unwrap.clone()) {
                is_payable = true;
            }
            if is_pub_fn_parttern(v_unwrap.clone()) {
                let concat_str = if is_payable {
                    is_payable = false;
                    v_unwrap + "#payable"
                } else {
                    v_unwrap
                };
                Some(concat_str)
            } else {
                    None
            }
        }
    ).collect();

    for func in extracted_data {
        println!("{:#?}", func);
        parse_contract_methods(func);
    }

}

fn is_pub_fn_parttern(line: String) -> bool {
    //TODO: Handle this case /**/
    if line.contains("pub") && line.contains("fn") && !line.starts_with("//") {
        return true;
    }
    false
}

fn is_payable_function(line: String) -> bool {
    line.trim() == "#[payable]"
}

fn parse_contract_methods(line: String) {
    //NOTE: FORMAT: pub fn name(&mut self, abc: String) -> Void {}
    let params_start = line.find('(').unwrap();
    let params_end = line.find(')').unwrap();

    let fn_index = line.find("fn").unwrap();
    let fn_name = line.substring(fn_index+ "fn".len(), params_start).trim();

    let params = line.substring(params_start + 1, params_end);
    let params_list: Vec<&str> = params.split(',').collect();
    let function_type = if line.contains(&"#payable") {
        FunctionType::PAYABLE
    } else if params_list[0] == "&mut self" {
        FunctionType::WRITE 
    } else if params_list[0] == "&self" {
        FunctionType::READ
    } else {
        FunctionType::UNKNOWN
    };

    let mut final_params = vec![];
    for param in params_list.iter().skip(1)  {
        
        let trimmed_param = param.trim();
        let single_param: Vec<&str> = trimmed_param.split(':').collect();

        final_params.push(
            ContractParam {
                name: single_param[0].trim().to_string(),
                param_type: single_param[1].trim().to_string()
            });
    }


    let contract_fn = ContractFunction {
        name: fn_name.to_string(),
        return_type: "".to_string(),
        params: final_params,
        fn_type: function_type
    };

    let json_object = serde_json::to_string(&contract_fn).unwrap();
    println!("Contract-fn: {} \n\n", json_object);
}





