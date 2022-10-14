#![allow(dead_code)]
use std::{collections::HashMap, fmt::{Result, Display, Debug, Formatter}, path::Path, str::FromStr};
pub enum Hsval{
    Hs(HashMap<String, Hsval>),
    String(String),
    Int(i16),
    Vec(Vec<String>),
}

//parse
fn index_of_whitespace_prefix(line: &str) -> usize {
    let mut index: usize = 0;
    for char in line.chars() {
        if char == ' ' {
            index += 1;
        } else {
            break
        }
    }
    index
}

fn update_scope_and_path(scope_index: &usize, scope_record: &mut Vec<usize>, key_path: &mut Vec<String>) {
    let new_len: usize = match scope_record.binary_search(&scope_index) {
        Ok(new_index) => new_index,
        Err(new_index) => new_index,
    };
    scope_record.drain((new_len)..);
    scope_record.push(*scope_index);
    key_path.drain(new_len..);
}

fn leftovers<T: FromStr>(line: &str, index: usize) -> T where <T as FromStr>::Err: Debug {
    let value: String = match line.get((index)..) {
        Some(val) => val.to_string(),
        None => "".to_string(),
    };
    match value.parse::<T>() {
        Ok(val) => val,
        Err(msg) => panic!("Failed to parse for type {:?} in line {}", msg, line)
    }
}


//for stringifying
fn to_nobl_under_the_hood(obj: &Hsval, buf: &mut String, scope: &mut String) -> String {
    match obj {
        Hsval::Hs(table) => {
            for (k, v) in table{
                match v {
                    Hsval::Hs(_) => {
                        buf.push_str(&(k.to_owned() + ":\n"));
                        scope.push_str("    ");
                        to_nobl_under_the_hood(v, buf, scope);
                        scope.truncate(scope.len() - 4);
                    },
                    Hsval::Vec(vec) => {
                        buf.push_str( &(scope.to_string() + k + ";\n"));
                        scope.push_str("    ");
                        for v in vec {
                            buf.push_str(&(scope.to_string() + v + "\n"));
                        }
                        scope.truncate(scope.len() - 4);
                    },
                    Hsval::String(val) => {
                        buf.push_str(&(scope.to_string() + k + ":" + val + "\n"));
                    },
                    Hsval::Int(val) => {
                        buf.push_str(&(scope.to_string() + k + ":0" + &val.to_string() + "\n"));
                    },
                }
            }
        },
        _=>{}
    }
    buf.to_string()
}

//for searching
fn spill(obj_list: Vec<&mut Hsval>) -> Vec<&mut Hsval> {
    let mut vec_buf: Vec<&mut Hsval> = vec![];
    for obj in obj_list {
        match obj {
            Hsval::Hs(table) => {
                for x in table.values_mut() {
                    vec_buf.push(x);
                };
            },
            _ => {vec_buf.push(obj);}
        };
    }
    vec_buf
}

fn filter<'a>(obj_list: Vec<&'a mut Hsval>, key: String) -> Vec<&'a mut Hsval> {
    let mut vec_buf: Vec<&mut Hsval> = vec![];
    for obj in obj_list {
        match obj.get_obj(&key) {
            Some(nested_obj) => {
                vec_buf.push(nested_obj);
            },
            None => {},
        }
    }
    vec_buf
}

impl Hsval {
    pub fn parse(data: String, err_ctx: String) -> Hsval {
        let mut key_path: Vec<String> = vec![];
        let mut obj: Hsval = Hsval::Hs(HashMap::new());
        let mut scope_record: Vec<usize> = vec![];
        let mut line_count = 0;
    
        for line in data.lines() {
            let err_msg = format!(" from {} at line {}, \"{}\"\n", err_ctx, line_count, line);
    
            let scope_index = index_of_whitespace_prefix(line);
    
            if scope_index == line.len() {
                continue;
            }
    
            update_scope_and_path(&scope_index, &mut scope_record, &mut key_path);
    
            let top_hsval: &mut Hsval = obj.get_obj_from_path(&key_path).expect(
                &format!("could not fetch obj from path given by key_path{}", err_msg)
            );
    
            let slice: String = leftovers::<String>(line, scope_index);
    
            match top_hsval {
                Hsval::Hs(table) => {
                    let chars: Vec<char> = slice.trim_end().chars().collect();
                    let mut key_str = String::new();
                    //new_key
                    let mut index = 0;
                    while index < chars.len() {
                        match (chars[index], chars.get(index + 1),) {
                            ('\\', Some(';')) => {
                                index += 1;
                                key_str.push_str("\\;");
                            }
                            ('\\', Some(':')) => {
                                index += 1;
                                key_str.push_str("\\:");
                            }
                            (':', Some('0')) => {
                                table.insert(key_str, Hsval::Int(leftovers::<i16>(&slice, index + 1)));
                                break;
                            }
                            (':', Some(_)) => {
                                table.insert(key_str, Hsval::String(leftovers::<String>(&slice, index + 1)));
                                break;
                            }
                            (ch, Some(_)) => {
                                key_str.push(ch);
                            }
                            (';', None) => {
                                table.insert(key_str.to_owned(), Hsval::Vec(Vec::new()));
                                key_path.push(key_str);
                                break;
                            }
                            (':', None) => {
                                table.insert(key_str.to_owned(), Hsval::Hs(HashMap::new()));
                                key_path.push(key_str);
                                break;
                            }
                            (_, None) => {
                                panic!("Failed to parse slice{}", err_ctx)
                            }
                        }
                        index += 1;
                    };
                }
                Hsval::Vec(vec) => {
                    vec.push(slice)
                }
                Hsval::String(_) => panic!("Expected vector or object but found string{}", err_msg),
                Hsval::Int(_) => panic!("Expected vector or object but found int{}", err_msg),
            }
    
            line_count += 1;
        };
        obj
    }
    
    pub fn parse_file<T: Display>(path: T) -> Hsval {
        let mut data = String::new();
        std::io::Read::read_to_string(&mut std::fs::File::open(path.to_string())
        .expect("File not found"), &mut data)
        .expect("Error while reading file");
        Self::parse(data, path.to_string())
    }
    
    pub fn stringify<T: AsRef<Path>>(&self, path: T) -> String {
        let stringified = self.to_nobl();
        std::fs::write(
            path, 
            stringified.to_owned()
        )
        .expect("Unable to write file");
        stringified
    }

    pub fn search(&mut self, key_path: Vec<Option<String>>) -> Vec<&mut Hsval> {
        let mut obj_list = vec![self];
        for possible_key in key_path {
            match possible_key {
                Some(key) => {
                    obj_list = filter(obj_list, key);
                },
                None => {
                    obj_list = spill(obj_list);
                },
            }
        }
        obj_list
    }
    
    pub fn get_obj(&mut self, key: &String) -> Option<&mut Hsval> {
        match self {
            Hsval::Hs(val) => {
                val.get_mut(key)
            },
            _ => None
        }
    }
    
    pub fn get_obj_from_path(&mut self, keys: &Vec<String>) -> Option<&mut Hsval> {
        let mut current_obj: Option<&mut Hsval> = Some(self);
        for key in keys {
            match current_obj {
                Some(new_obj) => {
                    current_obj = new_obj.get_obj(key);
                }
                None => {
                    current_obj = None;
                    break;
                }
            };
        };
        current_obj
    }

    pub fn template(&mut self, model: &Hsval) -> Hsval {
        match (&self, model) {
            //read (A, B) as "If I Have A and expect B"
    
            //if both values are tables, confirm values inside table also follow template
            (Hsval::Hs(_), Hsval::Hs(template_table)) => {
                for (key, val) in template_table.into_iter() {
                    match self.get_obj(key) {
                        Some(obj) => {
                            obj.template(&val);
                        },
                        None => {
                            match self {
                                Hsval::Hs(table) => {
                                    table.insert(key.to_string(), val.clone());
                                },
                                _=>{}
                            }
                        },
                    }
                }
            }
    
            //if types match, do nothing
            (Hsval::String(_), Hsval::String(_)) => {}
            (Hsval::Int(_), Hsval::Int(_)) => {}
            (Hsval::Vec(_), Hsval::Vec(_)) => {}
    
            //if you can, extract and convert self
            (Hsval::Int(val), Hsval::String(_)) => {
                *self = Hsval::String(val.to_string());
            }
            (Hsval::Vec(val), Hsval::String(_)) => {
                *self = Hsval::String(format!("{:?}", val));
            }
            (Hsval::String(val), Hsval::Vec(_)) => {
                *self = Hsval::Vec(vec![val.to_string()]);
            }
            (Hsval::String(val), Hsval::Int(model_int)) => {
                let int = match val.parse::<i16>() {
                    Ok(int_from_str) => int_from_str,
                    Err(_) => model_int.to_owned(),
                };
                *self = Hsval::Int(int);
            }
    
    
            //if they don't match, and data can not be salvaged, then overwrite
            (_, _) => {
                *self = model.clone();
            }
        };
        self.clone()
    }

    pub fn append(&mut self, obj: Hsval) {
        match (self, obj) {
            (Hsval::Hs(table1), Hsval::Hs(table2)) => {
                for (k, v) in table2 {
                    table1.insert(k, v);
                }
            },
            (_, _) => panic!("expected key-value pairs")
        }
    }

    pub fn to_nobl(&self) -> String {
        to_nobl_under_the_hood(&self, &mut String::new(), &mut String::new())
    }
    
    fn to_json(&self) -> String {
        fn json_fmt(obj: &Hsval, buf: &mut String, scope: &mut String) {
            match obj {
                Hsval::Hs(vals) => {
                    buf.push_str(&format!("{{").to_owned());
                    scope.push_str("    ");
                    for (k, v) in vals {
                        buf.push_str(&format!("\n{scope}\"{k}\":").to_owned());
                        json_fmt(v, buf, scope);
                        buf.push_str(&format!(",").to_owned());
                    }
                    buf.pop();
                    scope.truncate(scope.len() - 4);
                    buf.push_str(&format!("\n{scope}}}").to_owned());
                },
                Hsval::String(val) => buf.push_str(&format!("\"{val}\"").to_owned()),
                Hsval::Int(val) => buf.push_str(&format!("{val}").to_owned()),
                Hsval::Vec(val) => buf.push_str(&&format!("{val:?}").to_owned()),
            };
        }
        let mut buf = String::new();
        json_fmt(&self, &mut buf, &mut "".to_owned());
        buf
    }
}

impl Display for Hsval {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", &self.to_nobl())
    }
}

impl Debug for Hsval {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", &self.to_json())
    }
}

impl Clone for Hsval {
    fn clone(&self) -> Self {
        match self {
            Self::Hs(arg0) => Self::Hs(arg0.clone()),
            Self::String(arg0) => Self::String(arg0.clone()),
            Self::Int(arg0) => Self::Int(arg0.clone()),
            Self::Vec(arg0) => Self::Vec(arg0.clone()),
        }
    }
}
