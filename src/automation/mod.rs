use std::{collections::HashMap, fs::File, io::{Error, Write}, rc::Rc, time::Duration};

use alanglib::ast::{SourcePos, WithPosition};
use ast::{Expression, Statement};
use parser::Parser;
use scanner::Scanner;
use tokens::{Token, TokenType};

use crate::{univm::object::{ExecutionInterrupt, UniLValue}, utils::{duration_to_hms, lang::traceback_part, object::{expect_int, expect_number, expect_string}}, UniV};

mod scanner;
mod tokens;
mod parser;
mod ast;

const DEFAULT_LENGTH: usize = 256;
const DEFAULT_SPEED: f64 = 1.0;
const DEFAULT_SCALING: f64 = 1.0;

const RUN_ALL_UNIQUE_LIMIT: usize = 2;
const RUN_ALL_MIN_LENGTH: usize = 4;

#[derive(Debug)]
pub struct AutomationFile {
    pub source: Rc<str>,
    pub filename: Rc<str>
}

impl AutomationFile {
    pub fn new(source: Rc<str>, filename: Rc<str>) -> Self {
        AutomationFile { source, filename }
    }
}

pub enum AutomationMode {
    Normal,
    RunCategory(Rc<str>),
    RunSorts,
    RunShuffles
}

pub struct AutomationInterpreter {
    pub mode: AutomationMode,

    variables: HashMap<Rc<str>, UniLValue>,
    in_run_all_shuffles: bool,
    run_all_sorts: Option<Rc<str>>,
}

impl AutomationInterpreter {
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        variables.insert(Rc::from("SLOW_N_SQUARED_SCALE"), UniLValue::Int(6));
        variables.insert(Rc::from("N_SQUARED_SCALE"), UniLValue::Int(4));
        variables.insert(Rc::from("NLOG2N_SCALE"), UniLValue::Int(3));
        variables.insert(Rc::from("C_NLOGN_SCALE"), UniLValue::Int(2));
        variables.insert(Rc::from("NLOGN_SCALE"), UniLValue::Float(1.5));

        AutomationInterpreter { 
            variables,
            in_run_all_shuffles: false,
            mode: AutomationMode::Normal,
            run_all_sorts: None
        }
    }

    pub fn reset(&mut self) {
        self.mode = AutomationMode::Normal;
        self.in_run_all_shuffles = false;
        self.run_all_sorts = None;
    }

    pub fn create_exception_tok(&mut self, message: Rc<str>, tok: &Token) -> ExecutionInterrupt {
        ExecutionInterrupt::Exception { 
            value: UniLValue::String(message), 
            traceback: traceback_part(&tok.source, &tok.filename, tok.pos, tok.end - tok.pos, tok.line), 
            thread: 0 
        }
    }

    pub fn create_exception_ast(&mut self, message: Rc<str>, pos: SourcePos) -> ExecutionInterrupt {
        ExecutionInterrupt::Exception { 
            value: UniLValue::String(message), 
            traceback: traceback_part(&pos.source, &pos.filename, pos.start, pos.end - pos.start, pos.line), 
            thread: 0 
        }
    }

    pub fn create_exception(&mut self, message: Rc<str>) -> ExecutionInterrupt {
        ExecutionInterrupt::Exception { 
            value: UniLValue::String(message), 
            traceback: String::from(""), 
            thread: 0 
        }
    }
}

impl UniV {
    fn write_to_timestamp_file_inner(&mut self, text: &str) -> Result<(), Error> {
        if let Some(file) = &mut self.render.timestamp_file {
            file.write_all(duration_to_hms(&self.render.recording_duration).as_bytes())?;
            file.write_all(b" - ")?;
            file.write_all(text.as_bytes())?;
            file.write_all(b"\n")?;
        }

        Ok(())
    }

    fn write_to_timestamp_file(&mut self, text: &str, tok: &Token) -> Result<(), ExecutionInterrupt> {
        self.write_to_timestamp_file_inner(text)
            .map_err(|e| self.automation_interpreter.create_exception_tok(
                format!("Failed to write to timestamp file: {}", e.to_string()).into(), 
                tok
            ))?;

        Ok(())
    }

    fn evaluate_automation_expression(&mut self, expression: &Expression) -> Result<UniLValue, ExecutionInterrupt> {
        match expression {
            Expression::Float(token) => {
                Ok(UniLValue::Float(token.lexeme.parse::<f64>().expect("Parser produced unparsable float literal")))
            }
            Expression::Int(token) => {
                Ok(UniLValue::Int(token.lexeme.parse::<i64>().expect("Parser produced unparsable int literal")))
            }
            Expression::String(token) => {
                Ok(UniLValue::String(Rc::clone(&token.lexeme)))
            }
            Expression::Identifier(name) => {
                if let Some(value) = self.automation_interpreter.variables.get(&name.lexeme) {
                    Ok(value.clone())
                } else {
                    Err(self.automation_interpreter.create_exception_tok(format!(
                        "Unknown variable '{}'", name.lexeme
                    ).into(), name))
                }
            }
        }
    }

    fn evaluate_automation_statement(&mut self, statement: &Statement) -> Result<(), ExecutionInterrupt> {
        match statement {
            Statement::Describe { .. } => (), // ignore descriptions while running
            Statement::Set { type_, value: value_expr } => {
                let value = self.evaluate_automation_expression(value_expr)?;

                match type_.type_ {
                    TokenType::Visual => {
                        let name = expect_string(&value, "visual name", self)?;
                        self.set_visual_by_name(&name)?;
                    }
                    TokenType::Speed => {
                        let number = expect_number(&value, "speed", self)?;
                        self.set_speed(number)
                            .map_err(|e| {
                                self.automation_interpreter.create_exception_ast(
                                    e, value_expr.get_pos()
                                )
                            })?;
                    }
                    _ => unreachable!()
                }
            }
            Statement::Pop { .. } => {
                self.pop_autovalue();
            }
            Statement::Push { value, .. } => {
                let value = self.evaluate_automation_expression(value)?;
                self.push_autovalue(value);
            }
            Statement::Reset { type_ } => {
                match type_.type_ {
                    TokenType::Speed => self.reset_speed(),
                    TokenType::Queue => self.reset_autovalues(),
                    _ => unreachable!()
                }
            }
            Statement::Define { name, value } => {
                let value = self.evaluate_automation_expression(value)?;
                self.automation_interpreter.variables.insert(Rc::clone(&name.lexeme), value);
            }
            Statement::Timestamp { kw, value } => {
                let text = self.evaluate_automation_expression(&value)?.stringify();
                self.write_to_timestamp_file(&text, kw)?;
            }
            Statement::RunShuffle { kw, name, timestamp } => {
                let value = self.evaluate_automation_expression(name)?;
                let name = expect_string(&value, "shuffle name", self)?;

                if *timestamp || self.automation_interpreter.in_run_all_shuffles {
                    self.write_to_timestamp_file(format!("Shuffle: {}", name).as_str(), kw)?;
                }

                if self.automation_interpreter.in_run_all_shuffles {
                    let shuffle_id;
                    if let Ok(x) = self.gui.shuffles.binary_search(&name) {
                        shuffle_id = x as i32;
                    } else {
                        return Err(self.automation_interpreter.create_exception_tok(format!(
                            "Unknown shuffle \"{}\"", name
                        ).into(), kw));
                    }

                    self.runall_sorting_sequence(
                        self.gui.run_all_shuffles.distribution, 
                        shuffle_id, 
                        self.gui.run_all_shuffles.category, 
                        self.gui.run_all_shuffles.sort, 
                        self.gui.run_all_shuffles.array_length, 
                        self.gui.run_all_shuffles.unique_amt, 
                        self.gui.run_all_shuffles.speed, 
                        false
                    )?;
                } else {
                    self.run_shuffle(&name)?;
                }
            }
            Statement::RunDistribution { kw, name, length, unique, timestamp } => {
                let value = self.evaluate_automation_expression(name)?;
                let name = expect_string(&value, "distribution name", self)?;

                let length = {
                    if let Some(length) = length {
                        let value = self.evaluate_automation_expression(length)?;
                        expect_int(&value, "length", self)? as usize
                    } else {
                        DEFAULT_LENGTH
                    }
                };

                let unique = {
                    if let Some(unique) = unique {
                        let value = self.evaluate_automation_expression(unique)?;
                        expect_int(&value, "unique", self)? as usize
                    } else {
                        length / 2
                    }
                };

                if unique > length {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Unique amount cannot be greater than array length"), 
                        kw
                    ));
                }

                if *timestamp {
                    self.write_to_timestamp_file(format!("Distribution: {}", name).as_str(), kw)?;
                }

                self.run_distribution(&name, length, unique)?;
            }
            Statement::RunAllShuffles { kw, statements } => {
                if !matches!(self.automation_interpreter.mode, AutomationMode::RunShuffles) {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Cannot use run all shuffles automation outside run all shuffles context"), 
                        kw
                    ));
                }

                if self.automation_interpreter.in_run_all_shuffles {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Cannot define run all shuffles within run all shuffles"), 
                        kw
                    ));
                }

                if self.automation_interpreter.run_all_sorts.is_some() {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Cannot define run all shuffles within run all sorts"), 
                        kw
                    ));
                }

                self.automation_interpreter.in_run_all_shuffles = true;

                for statement in statements {
                    self.evaluate_automation_statement(statement)?;
                }

                self.automation_interpreter.in_run_all_shuffles = false;
            }
            Statement::RunSort { kw, name, category, length, speed, speed_scale, max_length, timestamp } => {
                let value = self.evaluate_automation_expression(name)?;
                let name = expect_string(&value, "sort name", self)?;

                if let Some(run_all_category) = self.automation_interpreter.run_all_sorts.clone() {
                    let length = {
                        if let Some(length) = length {
                            let value = self.evaluate_automation_expression(length)?;
                            expect_int(&value, "length", self)? as usize
                        } else {
                            DEFAULT_LENGTH
                        }
                    };

                    let mut speed = {
                        if let Some(speed) = speed {
                            let value = self.evaluate_automation_expression(speed)?;
                            expect_number(&value, "speed", self)?
                        } else {
                            DEFAULT_SPEED
                        }
                    };

                    let speed_scale = {
                        if let Some(speed_scale) = speed_scale {
                            let value = self.evaluate_automation_expression(speed_scale)?;
                            expect_number(&value, "speed scaling", self)?
                        } else {
                            DEFAULT_SCALING
                        }
                    };

                    if speed_scale == 0.0 {
                        return Err(self.automation_interpreter.create_exception_tok(
                            Rc::from("Speed scale cannot be 0"),
                            kw
                        ));
                    }

                    let category_id;
                    if let Ok(x) = self.gui.categories.binary_search(&run_all_category) {
                        category_id = x as i32;
                    } else {
                        return Err(self.automation_interpreter.create_exception_tok(format!(
                            "Unknown sort category \"{}\"", run_all_category
                        ).into(), kw));
                    }

                    let sort_id;
                    if let Ok(x) = self.gui.sorts[&run_all_category].binary_search(&name) {
                        sort_id = x as i32;
                    } else {
                        return Err(self.automation_interpreter.create_exception_tok(format!(
                            "Unknown sort \"{}\"", name
                        ).into(), kw));
                    }

                    let mut length = (length as f64 * self.gui.run_all_sorts.length_mlt) as usize;

                    if let Some(max_length) = max_length {
                        let max_length_value = self.evaluate_automation_expression(max_length)?;
                        let max_length = expect_int(&max_length_value, "max length", self)? as usize;
                        
                        if length > max_length {
                            length = max_length;
                        }
                    }

                    if length < RUN_ALL_MIN_LENGTH {
                        length = RUN_ALL_MIN_LENGTH;
                    }

                    let mut unique = (length as f64 / self.gui.run_all_sorts.unique_div) as usize;

                    if unique < RUN_ALL_UNIQUE_LIMIT {
                        unique = RUN_ALL_UNIQUE_LIMIT;
                    }

                    speed *= self.gui.run_all_sorts.length_mlt * self.gui.run_all_sorts.speed;

                    if self.gui.run_all_sorts.length_mlt > 1.0 {
                        speed *= speed_scale;
                    } else if self.gui.run_all_sorts.length_mlt < 1.0 {
                        speed /= speed_scale;
                    }

                    let full_name = Rc::clone(
                        &self.sorts.get(&run_all_category)
                            .expect("GUI sort data was not syncronized with UniV sort data")
                            .get(&name)
                            .expect("GUI sort data was not syncronized with UniV sort data")
                            .name
                    );

                    self.write_to_timestamp_file(&full_name, kw)?;

                    self.runall_sorting_sequence(
                        self.gui.run_all_sorts.distribution, 
                        self.gui.run_all_sorts.shuffle, 
                        category_id, 
                        sort_id, 
                        length, 
                        unique, 
                        speed, 
                        true
                    )?;
                } else {
                    if let Some(category) = category {
                        let value = self.evaluate_automation_expression(category)?;
                        let category = expect_string(&value, "category", self)?;

                        if *timestamp {
                            let full_name = {
                                if let Some(sort_category) = self.sorts.get(&category) {
                                    if let Some(sort_data) = sort_category.get(&name) {
                                        Rc::clone(&sort_data.name)
                                    } else {
                                        return Err(self.automation_interpreter.create_exception_tok(format!(
                                            "Unknown sort \"{}\"", name
                                        ).into(), kw));
                                    }
                                } else {
                                    return Err(self.automation_interpreter.create_exception_tok(format!(
                                        "Unknown sort category \"{}\"", category
                                    ).into(), kw));
                                } 
                            };

                            self.write_to_timestamp_file(&full_name, kw)?;              
                        }

                        self.run_sort(&category, &name)?;
                    } else {
                        return Err(self.automation_interpreter.create_exception_tok(
                            Rc::from("No category specified for sort"), 
                            kw
                        ));
                    }
                }
            }
            Statement::RunAllSorts { kw, categories } => {
                if !matches!(self.automation_interpreter.mode, AutomationMode::RunSorts | AutomationMode::RunCategory(_)) {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Cannot use run all sorts automation outside run all sorts context"), 
                        kw
                    ));
                }

                if self.automation_interpreter.in_run_all_shuffles {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Cannot define run all sorts within run all shuffles"), 
                        kw
                    ));
                }

                if self.automation_interpreter.run_all_sorts.is_some() {
                    return Err(self.automation_interpreter.create_exception_tok(
                        Rc::from("Cannot define run all sorts within run all sorts"), 
                        kw
                    ));
                }

                for category in categories {
                    let value = self.evaluate_automation_expression(&category.name)?;
                    let run_all_category = expect_string(&value, "category", self)?;

                    if let AutomationMode::RunCategory(selected) = &self.automation_interpreter.mode {
                        if run_all_category.as_ref() != selected.as_ref() {
                            continue;
                        }
                    }

                    self.automation_interpreter.run_all_sorts = Some(run_all_category);

                    for statement in &category.statements {
                        self.evaluate_automation_statement(statement)?;
                    }
                }

                self.automation_interpreter.run_all_sorts = None;
            }
        }
        
        Ok(())
    }

    fn parse_automation(&mut self, source: Rc<str>, filename: Rc<str>) -> Result<Vec<Statement>, ExecutionInterrupt> {
        let mut scanner = Scanner::new(source, filename);
        scanner.scan_tokens();

        if !scanner.errors.is_empty() {
            let mut error_buf = String::from("Something went wrong while scanning automation:");

            for error in &scanner.errors {
                error_buf.push('\n');
                error_buf.push_str(&error.to_string());
            }

            return Err(self.automation_interpreter.create_exception(error_buf.into()));
        }
    
        let mut parser = Parser::new(scanner.tokens);
        let script = parser.parse();

        if !parser.errors.is_empty() {
            let mut error_buf = String::from("Something went wrong while parsing automation:");

            for error in &parser.errors {
                error_buf.push('\n');
                error_buf.push_str(&error.to_string());
            }

            return Err(self.automation_interpreter.create_exception(error_buf.into()));
        }

        Ok(script)
    }

    fn init_timestamp_file(&mut self) -> Result<(), ExecutionInterrupt> {
        if self.render.active {
            self.render.recording_duration = Duration::ZERO;

            if self.settings.render.save_timestamps {
                self.render.timestamp_file = Some(
                    File::create("timestamps.txt")
                        .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?
                );
            }
        }

        Ok(())
    }

    fn execute_automation_inner(&mut self, source: Rc<str>, filename: Rc<str>) -> Result<(), ExecutionInterrupt> {
        let script = self.parse_automation(source, filename)?;
        
        self.init_timestamp_file()?;

        for statement in script {
            self.evaluate_automation_statement(&statement)?;
        }

        Ok(())
    }

    pub fn execute_automation(&mut self, source: Rc<str>, filename: Rc<str>) -> Result<(), ExecutionInterrupt> {
        let ret = self.execute_automation_inner(source, filename);
        self.render.timestamp_file.take(); // release the timestamps file, if any
        ret
    }

    pub fn get_automation_description(&mut self, source: Rc<str>, filename: Rc<str>) -> Result<String, ExecutionInterrupt> {
        let script = self.parse_automation(source, filename)?;

        let mut output = String::new();
        for statement in script {
            if let Statement::Describe { value, .. } = statement {
                output.push_str(&self.evaluate_automation_expression(&value)?.stringify());
                output.push('\n');
            }
        }

        if output == "" {
            output.push_str("No description provided");
        }

        Ok(output)
    }
}