#![allow(dead_code)]
use std::{collections::HashMap, str::{Split, FromStr}, slice::Iter};
pub enum Hsval{
    Hs(HashMap<String, Hsval>),
    String(String),
    Int(i16),
    Vec(Vec<String>),
}

impl Hsval {
    pub fn parse(path: &str) -> Hsval {
        let mut data = String::new();
    
        {
            use std::{io::Read, fs::File};
    
            File::open(path)
            .expect("File not found")
            .read_to_string(&mut data)
            .expect("Error while reading file");
        }
    
        let data: Split<&str> = data.split("\r\n");
        let mut key_record: Vec<String>= Vec::new();
        let mut list: Hsval = Hsval::Hs(HashMap::new());
        let mut scope_count: usize = 0;
    
        for line in data {
            let scope = {
                let mut lower_bound: usize = 0;
                let mut upper_bound: usize = 4;
                let mut scope: usize = 0;
    
                while line.get(lower_bound..upper_bound) == Some("    ") {
                    lower_bound += 4;
                    upper_bound += 4;
                    scope += 1 as usize;
                };
                println!("\n(scope, scope_count) = ({scope}, {scope_count})");
                
                if scope < scope_count {
                    key_record.resize(scope, "".to_string());
                };
                
                scope_count = scope;

                println!("(scope, scope_count) = ({scope}, {scope_count})");
                print!("line ------------ \n{}\n-----------------", line);
    
                scope
            };
    
            let char_index = {
                let mut char_index: usize = 0 as usize;
                for char in line.chars() {
                    match char {
                        ';' => break,
                        ':' => break,
                        _ => char_index += 1 as usize
                    }
                }
                if char_index >= line.len() {
                    0
                } else {
                    char_index
                }
            };
    
            let top_hsval = {
                let mut top_hsval: &mut Hsval = &mut list;
                println!("{:?}", key_record);
                for key in &key_record {
                    match top_hsval {
                        Hsval::Hs(obj) => {
                            top_hsval = obj.entry(key.to_string()).or_insert(Hsval::Int(3));
                        },
                        _ => {}
                    }
                }
                top_hsval
            };
    
    
            let almost_past_len: usize = ((char_index + 1) >= line.len()) as usize;
    
            match (
                line.get((scope * 4)..(char_index)),
                //key: Option<&str>
                line.get((
                    (((char_index > (scope*4)) as usize) * (char_index + 1)) + 
                    ((!(char_index > (scope * 4)) as usize) * scope * 4) + almost_past_len)..),
                //val: Option<&str>
            ) {
                (Some(key), Some(val)) => {
                    match top_hsval {
                        Hsval::Hs(obj) => {
                            obj.insert(
                                key.to_string(), 
                                match line.get((char_index + 1)..(char_index + 2)) {
                                    Some("0") => Hsval::Int(FromStr::from_str(val)
                                    .expect("ayo, you gave some shit that aint a number")),
                                    _ => Hsval::String(val.to_string()),
                                    
                                }
                            );
                        }
                        _ => {}
                    };
                },
                (Some(key), None) => {
                    match top_hsval {
                        Hsval::Hs(obj) => {
                            obj.insert(
                                key.to_string(), 
                                match line.get((char_index + almost_past_len - 1)..(char_index + almost_past_len)) {
                                    Some(":") => Hsval::Hs(HashMap::new()),
                                    Some(";") => Hsval::Vec(Vec::new()),
                                    _ => Hsval::Hs(HashMap::new()),
                                }
                            );
                        },
                        _ => {}
                    };
                    key_record.push(key.to_string());
                },
                (None, Some(val)) => {
                    match top_hsval {
                        Hsval::Vec(vec) => {
                            vec.push(val.to_string());
                        },
                        _ => {},
                    } 
                },
                (None, None) => {}
            }
        };
        list
    }
    
    pub fn stringify<'a>(&self, path: &'a  str) -> String {
        let mut scope = String::new();
        let mut buf = String::new();
    
        fn stringify_via_recursion(obj: &Hsval, scope: &mut String, buf: &mut String) -> String {
            match obj {
                Hsval::Hs(table) => {
                    for (k, v) in table{
                        match v {
                            Hsval::Hs(_) => {
                                *buf = buf.to_owned() + scope + k + ":\n";
                                scope.push_str("    ");
                                *buf = stringify_via_recursion(v, scope, buf);
                                scope.truncate(scope.len() - 4);
                            },
                            Hsval::Vec(vec) => {
                                *buf = buf.to_owned() + scope + &k.to_string();
                                scope.push_str("    ");
                                for v in vec {
                                    *buf = buf.to_owned() + "\n" + scope + v;
                                }
                                scope.truncate(scope.len() - 4);
                                *buf = buf.to_owned() + "\n";
                            },
                            Hsval::String(val) => {
                                *buf = buf.to_owned() + scope + k + ":" + val + "\n";
                            },
                            Hsval::Int(val) => {
                                *buf = buf.to_owned() + scope + &k + ":0" + &val.to_string() + "\n";
                            },
                        }
                    }
                },
                _=>{}
            }
            buf.to_string()
        }
        std::fs::write(path, stringify_via_recursion(&self, &mut scope, &mut buf))
            .expect("Unable to write file");
        buf
    }
    
    pub fn get_obj<'a>(&mut self, keys: &mut Iter<String>) -> &mut Hsval  {
        match keys.next() {
            Some(key) => {
                match self {
                    Hsval::Hs(table) => Hsval::get_obj(
                        table.get_mut(&key.to_string()).unwrap(),
                        keys
                    ),
                    _ => self
                }
            },
            None => self,
        }
    }

    fn print(&self, scope: &mut String) {
        match &self {
            Hsval::Hs(vals) => {
                print!("{{\n");
                scope.push_str("    ");
                for (k, v) in vals {
                    print!("{scope}{k}:");
                    Hsval::print(v, scope);
                    print!(",\n");
                }
                scope.truncate(scope.len() - 4);
                print!("{scope}}}");
            },
            Hsval::String(val) => print!("{val}"),
            Hsval::Int(val) => print!("{val}"),
            Hsval::Vec(val) => print!("{val:?}"),
        };
    }
}

impl std::fmt::Debug for Hsval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut scope = "".to_string();
        Hsval::print(&self, &mut scope);
        write!(f, "")
    }
}
