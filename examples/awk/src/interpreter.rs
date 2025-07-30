use crate::ast::*;
use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;

pub struct Interpreter {
    // Variable storage for user-defined and built-in variables
    variables: HashMap<String, Value>,

    // Current record split into fields: fields[0] = whole record, fields[1] = $1, etc.
    fields: Vec<String>,

    // Built-in variable state
    record_number: usize,            // NR - Number of records processed
    field_separator: String,         // FS - Field separator
    output_field_separator: String,  // OFS - Output field separator
    record_separator: String,        // RS - Record separator
    output_record_separator: String, // ORS - Output record separator
}

impl Interpreter {
    pub fn new() -> Self {
        let mut variables = HashMap::new();

        // Initialize AWK's built-in variables with default values
        variables.insert("FS".to_string(), Value::String(" ".to_string()));
        variables.insert("OFS".to_string(), Value::String(" ".to_string()));
        variables.insert("RS".to_string(), Value::String("\n".to_string()));
        variables.insert("ORS".to_string(), Value::String("\n".to_string()));

        Self {
            variables,
            fields: Vec::new(),
            record_number: 0,
            field_separator: " ".to_string(),
            output_field_separator: " ".to_string(),
            record_separator: "\n".to_string(),
            output_record_separator: "\n".to_string(),
        }
    }

    // Allow external configuration of field separator (for -F command line option)
    pub fn set_field_separator(&mut self, fs: String) {
        self.field_separator = fs.clone();
        self.variables.insert("FS".to_string(), Value::String(fs));
    }

    pub fn run_program(&mut self, program: &Program, input: &str) -> Result<()> {
        // Phase 1: Execute all BEGIN rules before processing any input
        for rule in &program.rules {
            if let Some(Pattern::Begin) = rule.pattern {
                self.execute_statements(&rule.action)?;
            }
        }

        // Phase 2: Process input record by record
        let records: Vec<&str> = if self.record_separator == "\n" {
            input.lines().collect() // Default: split on newlines
        } else {
            input.split(&self.record_separator).collect() // Custom record separator
        };

        for record in records {
            if record.is_empty() {
                continue; // Skip empty records
            }

            // Update record state
            self.record_number += 1;
            self.split_fields(record);

            // Execute pattern-matching rules for this record
            for rule in &program.rules {
                match &rule.pattern {
                    Some(Pattern::Regex(regex_str)) => {
                        // TODO: regex can be cached and compiled once
                        let regex = Regex::new(regex_str)?;
                        if regex.is_match(record) {
                            self.execute_statements(&rule.action)?;
                        }
                    }
                    Some(Pattern::Expression(expr)) => {
                        if self.eval_expr(expr)?.is_truthy() {
                            if rule.action.is_empty() {
                                // If action is empty, print $0
                                println!("{}", self.fields[0]);
                            } else {
                                self.execute_statements(&rule.action)?;
                            }
                        }
                    }
                    None => {
                        // No pattern means always execute
                        self.execute_statements(&rule.action)?;
                    }
                    _ => {} // Skip BEGIN/END here
                }
            }
        }

        // Phase 3: Execute all END rules after processing all input
        for rule in &program.rules {
            if let Some(Pattern::End) = rule.pattern {
                self.execute_statements(&rule.action)?;
            }
        }

        Ok(())
    }

    fn split_fields(&mut self, line: &str) {
        self.fields = vec![line.to_string()]; // $0 is always the entire record

        if self.field_separator == " " {
            // AWK's default behavior: split on any whitespace, ignore leading/trailing
            self.fields.extend(
                line.split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            );
        } else {
            // Custom field separator: split exactly on that string
            self.fields.extend(
                line.split(&self.field_separator)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            );
        }

        // Update built-in variables
        self.variables.insert(
            "NF".to_string(),
            Value::Number((self.fields.len() - 1) as f64),
        );
        self.variables
            .insert("NR".to_string(), Value::Number(self.record_number as f64));
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::Identifier(name) => {
                // Variable lookup with AWK semantics: undefined variables are empty strings
                Ok(self
                    .variables
                    .get(name)
                    .cloned()
                    .unwrap_or(Value::String("".to_string())))
            }

            Expr::BuiltinVar(name) => {
                // Built-in variables default to 0 if not found
                Ok(self
                    .variables
                    .get(name)
                    .cloned()
                    .unwrap_or(Value::Number(0.0)))
            }

            Expr::FieldRef(index_expr) => {
                let index = self.eval_expr(index_expr)?;
                let idx = index.to_number() as usize;

                // AWK allows accessing fields beyond NF (returns empty string)
                if idx < self.fields.len() {
                    Ok(Value::String(self.fields[idx].clone()))
                } else {
                    Ok(Value::String("".to_string()))
                }
            }

            Expr::BinaryOp { op, left, right } => {
                let lval = self.eval_expr(left)?;
                let rval = self.eval_expr(right)?;
                self.eval_binary_op(op, &lval, &rval)
            }

            Expr::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand)?;
                self.eval_unary_op(op, &val)
            }
        }
    }

    fn eval_binary_op(&self, op: &BinOp, left: &Value, right: &Value) -> Result<Value> {
        match op {
            // Arithmetic operations: always work on numeric values
            BinOp::Add => Ok(Value::Number(left.to_number() + right.to_number())),
            BinOp::Sub => Ok(Value::Number(left.to_number() - right.to_number())),
            BinOp::Mul => Ok(Value::Number(left.to_number() * right.to_number())),
            BinOp::Div => {
                let divisor = right.to_number();
                if divisor == 0.0 {
                    return Err(anyhow!("Division by zero"));
                }
                Ok(Value::Number(left.to_number() / divisor))
            }
            BinOp::Mod => {
                let divisor = right.to_number();
                if divisor == 0.0 {
                    return Err(anyhow!("Modulo by zero"));
                }
                Ok(Value::Number(left.to_number() % divisor))
            }
            BinOp::Pow => Ok(Value::Number(left.to_number().powf(right.to_number()))),

            // Comparison operations: return 1.0 for true, 0.0 for false
            BinOp::Eq => Ok(Value::Number(if self.values_equal(left, right) {
                1.0
            } else {
                0.0
            })),
            BinOp::Ne => Ok(Value::Number(if !self.values_equal(left, right) {
                1.0
            } else {
                0.0
            })),
            BinOp::Lt => Ok(Value::Number(if left.to_number() < right.to_number() {
                1.0
            } else {
                0.0
            })),
            BinOp::Le => Ok(Value::Number(if left.to_number() <= right.to_number() {
                1.0
            } else {
                0.0
            })),
            BinOp::Gt => Ok(Value::Number(if left.to_number() > right.to_number() {
                1.0
            } else {
                0.0
            })),
            BinOp::Ge => Ok(Value::Number(if left.to_number() >= right.to_number() {
                1.0
            } else {
                0.0
            })),

            // Pattern matching operations: use regex crate
            BinOp::Match => {
                let text = left.to_string(); // Display trait converts Value to String
                let pattern = right.to_string(); // Works for both Number and String variants
                let regex = Regex::new(&pattern)?;
                Ok(Value::Number(if regex.is_match(&text) { 1.0 } else { 0.0 }))
            }
            BinOp::NotMatch => {
                let text = left.to_string(); // Display trait ensures consistent formatting
                let pattern = right.to_string();
                let regex = Regex::new(&pattern)?;
                Ok(Value::Number(if !regex.is_match(&text) {
                    1.0
                } else {
                    0.0
                }))
            }

            // Logical operations: use AWK truthiness rules
            BinOp::And => Ok(Value::Number(if left.is_truthy() && right.is_truthy() {
                1.0
            } else {
                0.0
            })),
            BinOp::Or => Ok(Value::Number(if left.is_truthy() || right.is_truthy() {
                1.0
            } else {
                0.0
            })),
        }
    }

    fn eval_unary_op(&self, op: &UnOp, operand: &Value) -> Result<Value> {
        match op {
            UnOp::Neg => Ok(Value::Number(-operand.to_number())),
            UnOp::Not => Ok(Value::Number(if operand.is_truthy() { 0.0 } else { 1.0 })),
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => left.to_string() == right.to_string(),
        }
    }

    fn execute_statements(&mut self, statements: &[Statement]) -> Result<()> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Print(exprs) => {
                if exprs.is_empty() {
                    // Print $0
                    print!("{}{}", self.fields[0], self.output_record_separator);
                } else {
                    let values: Result<Vec<_>> =
                        exprs.iter().map(|expr| self.eval_expr(expr)).collect();
                    let values = values?;
                    let output: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    print!(
                        "{}{}",
                        output.join(&self.output_field_separator),
                        self.output_record_separator
                    );
                }
            }
            Statement::Assignment { target, op, value } => {
                let new_value = self.eval_expr(value)?;
                match target {
                    AssignTarget::Identifier(name) => {
                        let final_value = match op {
                            AssignOp::Assign => new_value,
                            AssignOp::AddAssign => {
                                let current = self
                                    .variables
                                    .get(name)
                                    .cloned()
                                    .unwrap_or(Value::Number(0.0));
                                Value::Number(current.to_number() + new_value.to_number())
                            }
                            AssignOp::SubAssign => {
                                let current = self
                                    .variables
                                    .get(name)
                                    .cloned()
                                    .unwrap_or(Value::Number(0.0));
                                Value::Number(current.to_number() - new_value.to_number())
                            }
                            AssignOp::MulAssign => {
                                let current = self
                                    .variables
                                    .get(name)
                                    .cloned()
                                    .unwrap_or(Value::Number(0.0));
                                Value::Number(current.to_number() * new_value.to_number())
                            }
                            AssignOp::DivAssign => {
                                let current = self
                                    .variables
                                    .get(name)
                                    .cloned()
                                    .unwrap_or(Value::Number(0.0));
                                Value::Number(current.to_number() / new_value.to_number())
                            }
                            AssignOp::ModAssign => {
                                let current = self
                                    .variables
                                    .get(name)
                                    .cloned()
                                    .unwrap_or(Value::Number(0.0));
                                Value::Number(current.to_number() % new_value.to_number())
                            }
                        };
                        self.variables.insert(name.clone(), final_value);

                        // Update separators if built-in variables are modified
                        if name == "FS" {
                            self.field_separator = self.variables.get("FS").unwrap().to_string();
                        } else if name == "OFS" {
                            self.output_field_separator =
                                self.variables.get("OFS").unwrap().to_string();
                        } else if name == "RS" {
                            self.record_separator = self.variables.get("RS").unwrap().to_string();
                        } else if name == "ORS" {
                            self.output_record_separator =
                                self.variables.get("ORS").unwrap().to_string();
                        }
                    }
                    AssignTarget::FieldRef(field_expr) => {
                        let index = self.eval_expr(field_expr)?;
                        let idx = index.to_number() as usize;

                        // Extend fields array if necessary
                        while self.fields.len() <= idx {
                            self.fields.push("".to_string());
                        }

                        self.fields[idx] = new_value.to_string();

                        // Update NF
                        self.variables.insert(
                            "NF".to_string(),
                            Value::Number((self.fields.len() - 1) as f64),
                        );
                    }
                }
            }
            Statement::Increment(target) => match target {
                AssignTarget::Identifier(name) => {
                    let current = self
                        .variables
                        .get(name)
                        .cloned()
                        .unwrap_or(Value::Number(0.0));
                    let new_value = Value::Number(current.to_number() + 1.0);
                    self.variables.insert(name.clone(), new_value);
                }
                AssignTarget::FieldRef(field_expr) => {
                    let index = self.eval_expr(field_expr)?;
                    let idx = index.to_number() as usize;

                    while self.fields.len() <= idx {
                        self.fields.push("".to_string());
                    }

                    let current = self.fields[idx].parse::<f64>().unwrap_or(0.0);
                    self.fields[idx] = (current + 1.0).to_string();

                    self.variables.insert(
                        "NF".to_string(),
                        Value::Number((self.fields.len() - 1) as f64),
                    );
                }
            },
            Statement::Decrement(target) => match target {
                AssignTarget::Identifier(name) => {
                    let current = self
                        .variables
                        .get(name)
                        .cloned()
                        .unwrap_or(Value::Number(0.0));
                    let new_value = Value::Number(current.to_number() - 1.0);
                    self.variables.insert(name.clone(), new_value);
                }
                AssignTarget::FieldRef(field_expr) => {
                    let index = self.eval_expr(field_expr)?;
                    let idx = index.to_number() as usize;

                    while self.fields.len() <= idx {
                        self.fields.push("".to_string());
                    }

                    let current = self.fields[idx].parse::<f64>().unwrap_or(0.0);
                    self.fields[idx] = (current - 1.0).to_string();

                    self.variables.insert(
                        "NF".to_string(),
                        Value::Number((self.fields.len() - 1) as f64),
                    );
                }
            },
            Statement::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.execute_statement(then_stmt)?;
                } else if let Some(else_branch) = else_stmt {
                    self.execute_statement(else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                while self.eval_expr(condition)?.is_truthy() {
                    self.execute_statement(body)?;
                }
            }
            Statement::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init_stmt) = init {
                    self.execute_statement(init_stmt)?;
                }

                loop {
                    if let Some(cond) = condition {
                        if !self.eval_expr(cond)?.is_truthy() {
                            break;
                        }
                    }

                    self.execute_statement(body)?;

                    if let Some(update_stmt) = update {
                        self.execute_statement(update_stmt)?;
                    }
                }
            }
            Statement::Block(statements) => {
                self.execute_statements(statements)?;
            }
            Statement::Expression(expr) => {
                self.eval_expr(expr)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_splitting() {
        let mut interpreter = Interpreter::new();
        interpreter.split_fields("Alice 25 Engineer");
        assert_eq!(interpreter.fields[0], "Alice 25 Engineer"); // $0
        assert_eq!(interpreter.fields[1], "Alice"); // $1
        assert_eq!(interpreter.fields[2], "25"); // $2
        assert_eq!(interpreter.fields[3], "Engineer"); // $3
    }

    #[test]
    fn test_expression_evaluation() {
        let interpreter = Interpreter::new();
        // Test that "25" > "100" evaluates correctly (numerically)
        let expr = Expr::BinaryOp {
            op: BinOp::Gt,
            left: Box::new(Expr::String("25".to_string())),
            right: Box::new(Expr::String("100".to_string())),
        };
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result.to_number(), 0.0); // 25 > 100 is false
    }
}
