use std::{collections::HashMap, fmt::format};

use crate::ast::*;

pub struct Compiler {
    num_of_constructors: u32,
    output: String,
    depth: u32,
    cons_map: HashMap<String, u32>,
    data_map: HashMap<String, (String, u32)>,
    var_counter: i32

}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            num_of_constructors: 0,
            output: String::new(),
            depth: 0,
            cons_map: HashMap::new(),
            data_map: HashMap::new(),
            var_counter: 0
        }
    }

    fn add_to_cons_map(&mut self, def : ADTDefinition) {
        for cons in def.constructors {
            let cons_id = cons.id.0;
            self.cons_map.insert(cons_id, self.num_of_constructors);
            self.num_of_constructors += 1;
        }
    }

    fn add_to_data_map(&mut self, def : ADTDefinition) {
        for (i, cons) in def.constructors.iter().enumerate() {
            let cons_id = cons.id.0.clone();
            self.data_map.insert(cons_id, (def.id.0.clone(), i as u32));
        }
    }

    pub fn compile(&mut self, prog : Program) {
        for def in prog.0 {
            match def {
                Definition::ADTDefinition(ddef) => self.compile_adt(ddef),
                Definition::FunctionDefinition(fdef) => self.compile_fun(fdef)
            }
            self.emit_char('\n');
        }
    }

    fn compile_fun(&mut self, fun : FunctionDefinition) {
        let mut output = match fun.signature.result_type {
            Type::Int => "int ".to_string(),
            Type::ADT(id) => id.0 + "* ",
            _ => todo!()
        };
        output += &fun.id.0;
        output.push('(');
        let args = fun.signature.argument_type.0;
        for (i , t) in args.iter().enumerate() {
            match t {
                Type::Int => output.push_str("int "),
                Type::ADT(id) => {
                    output += &id.0;
                    output.push(' ');
                }
                _ => todo!(),
            }
            output.push_str(fun.args[i].as_str());
            if i + 1 != args.len() {
                output.push(',');
            }
        }
        output.push_str(") {");
        self.emit_line(&output.as_str());
        self.indent();
        self.compile_exp(&fun.body);
        self.unindent();
        self.emit_line("}");
        return
    }

    fn compile_exp(&mut self, exp: &Expression) -> String{
        return match exp {
            Expression::Integer(i) => i.to_string(),
            Expression::Identifier(VID(n)) => n.to_string(),
            Expression::Constructor(FID(cons), exps) => {
                let ((data, constructor_num), tag) = {
                    let data: &(String, u32) = self.data_map.get(cons).unwrap();
                    let tag = self.cons_map.get(cons).unwrap();
                    (data.clone(), *tag)  // Clone values to avoid holding references
                };
                let id = self.get_unique_var_id();
                self.emit_line(&format!("{}* var{} = malloc(sizeof({}));", data, id, data));
                self.emit_line(&format!("var{}->tag = {};", id, tag));
                for (i, exp) in exps.iter().enumerate() {

                    let result: String = self.compile_exp(exp);
                    self.emit_line(&format!("var{}->data.constructor_{}.field_{} = {};", id, constructor_num, i, result))
                }
                return format!("var{}", id)
            },
            Expression::Operation(operator, exp1,exp2 ) => {
                let sop = operator.to_string();
                let result1 = self.compile_exp(exp1);
                let result2 = self.compile_exp(exp2);
                
                return format!("{} {} {}", result1, sop, result2);
            },
            Expression::FunctionCall(FID(name),TupleExpression(exps) ) => {
                let mut results = Vec::new();
                for exp in exps {
                    let result = self.compile_exp(exp);
                    results.push(result);
                }
                format!("{}({});",name, results.join(", "))
            },
            Expression::Match(match_expression) => {
                let result = self.compile_exp(&match_expression.exp);
                for case in match_expression.cases.iter() {
                    
                }

                return String::new()
            }
            _ => todo!()
        };
    }

    fn compile_adt(&mut self, data: ADTDefinition) {
        self.add_to_cons_map(data.clone());
        self.add_to_data_map(data.clone());
        self.emit_line(&format!("typedef struct {} {};", data.id.0, data.id.0));
        self.emit_line("typedef struct {");
        self.indent();
        self.emit_line("int tag;");
        self.emit_line("union {");
        self.indent();
        self.compile_constructors(data.constructors);
        self.unindent();
        self.emit_line("} data;");
        self.unindent();
        self.emit_line(("} ".to_string() + &data.id.0 + &";".to_string()).as_str());
    }

    fn compile_constructors(&mut self, constructors: Vec<ConstructorDefinition>) {
        for (index, cons) in constructors.iter().enumerate() {
            let args = &cons.argument.0;
            if args.is_empty() {
                continue;
            } 
            else if args.len() == 1 && self.is_single_non_tuple(&args[0]) {
                self.compile_single_field_constructor(index, &args[0]);
            } 
            else {
                self.emit_line(&format!("struct {{"));
                self.indent();
                for (i, arg) in args.iter().enumerate() {
                    self.compile_type(arg,i as u8);
                }
                self.unindent();
                self.emit_line(&format!("}} constructor_{};", index));
            }
        }
    }
    
    fn is_single_non_tuple(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::ADT(_) => true,
            Type::Tuple(rec) => {
                //Compiles as a chain of structs if one tuple has more than 1 element, 
                // might change later.
                rec.0.len() == 1 && self.is_single_non_tuple(&rec.0[0])
            }
        }
    }
    
    fn compile_single_field_constructor(&mut self, index: usize, ty: &Type) {
        match ty {
            Type::Int => {
                self.emit_line(&format!("int constructor_{};", index));
            }
            Type::ADT(id) => {
                self.emit_line(&format!("{}* constructor_{};", id.0, index));
            }
            Type::Tuple(rec) => {
                //For nested single element tuples
                self.compile_single_field_constructor(index, &rec.0[0]);
            }
        }
    }
    
    fn compile_type(&mut self, t: &Type, field_index: u8) {
        match t {
            Type::Int => {
                self.emit_line(&format!("int field_{};", field_index));
            }
            Type::ADT(id) => {
                self.emit_line(&format!("{}* field_{};", id.0, field_index));
            }
            Type::Tuple(rec) => {
                if rec.0.is_empty() {
                    self.emit_line(&format!("Unit field_{}", field_index));
                }
                else if rec.0.len() == 1 {
                    self.compile_type(&rec.0[0], field_index);
                } 
                else {
                    self.emit_line("struct {");
                    self.indent();
                    for (i,arg) in rec.0.iter().enumerate() {
                        self.compile_type(&arg, i as u8);
                    }
                    self.unindent();
                    self.emit_line(&format!("}} field_{};", field_index));
                }
            }
        }
    }

    fn emit_line(&mut self, line : &str) {
        self.output.push_str(&"\t".repeat(self.depth as usize));
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_str(&mut self, str : &str) {
        self.output.push_str(str)
    }

    fn emit_char(&mut self, c : char) {
        self.output.push(c);
    }

    fn indent(&mut self) {
        self.depth += 1
    }

    fn unindent(&mut self) {
        self.depth -= 1
    }

    fn get_unique_var_id(&mut self) -> i32 {
        self.var_counter += 1;
        return self.var_counter
    }
    
    pub fn get_output(&self) -> &str {
        self.output.as_str()
    }

}
