use std::{collections::HashMap, rc::Rc};

use crate::interpreter::lox_instance::LoxInstance;

use crate::{
    environment::{self, Environment, EnvironmentRef},
    expr::Expr,
    expr_id::ExprId,
    interpreter::{self, lox_function::LoxFunction},
    stmt::{ClassMember, Stmt},
    token::{Literal, Token, TokenType, Value},
};

pub struct Interpreter {
    globals: EnvironmentRef,
    environment: EnvironmentRef,
    locals: HashMap<ExprId, LocalResolution>,
}

#[derive(Debug, Clone, Copy)]
struct LocalResolution {
    distance: usize,
    slot: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();

        globals.borrow_mut().define(
            "clock".to_string(),
            Some(Value::Callable(Rc::new(NativeClock))),
        );

        Self {
            globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> ExecutionResult {
        for statement in statements {
            match self.execute(statement) {
                ExecutionResult::Success => {}
                result => return result,
            }
        }

        ExecutionResult::Success
    }

    pub fn evaluate_expression(&mut self, expression: &Expr) -> Result<Value, RuntimeError> {
        self.evaluate(expression)
    }

    pub fn resolve(&mut self, id: ExprId, distance: usize, slot: usize) {
        self.locals.insert(id, LocalResolution { distance, slot });
    }

    fn define_variable(&mut self, name: &Token, value: Option<Value>) {
        if Rc::ptr_eq(&self.environment, &self.globals) {
            self.globals.borrow_mut().define(name.lexeme.clone(), value);
        } else {
            self.environment.borrow_mut().define_local(value);
        }
    }

    fn assign_variable(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if Rc::ptr_eq(&self.environment, &self.globals) {
            self.globals.borrow_mut().assign(name, value)
        } else {
            self.environment
                .borrow_mut()
                .assign_last_local(Some(value));

            Ok(())
        }
    }

    fn execute(&mut self, statement: &Stmt) -> ExecutionResult {
        match statement {
            Stmt::Expression { expression } => match self.evaluate(expression) {
                Ok(_) => ExecutionResult::Success,
                Err(error) => ExecutionResult::RuntimeError(error),
            },

            Stmt::Print { expression } => match self.evaluate(expression) {
                Ok(value) => {
                    println!("{}", self.stringify(&value));
                    ExecutionResult::Success
                }
                Err(error) => ExecutionResult::RuntimeError(error),
            },
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expression) => match self.evaluate(expression) {
                        Ok(value) => Some(value),
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    },
                    None => None,
                };

                self.define_variable(name, value);
                ExecutionResult::Success
            }

            Stmt::Block { statements } => {
                let environment = Environment::from(self.environment.clone());

                self.execute_block(statements, environment)
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = match self.evaluate(condition) {
                    Ok(value) => value,
                    Err(error) => return ExecutionResult::RuntimeError(error),
                };

                if self.is_truthy(&condition) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    ExecutionResult::Success
                }
            }

            Stmt::While { condition, body } => {
                loop {
                    let condition_value = match self.evaluate(condition) {
                        Ok(value) => value,
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    };

                    if !self.is_truthy(&condition_value) {
                        break;
                    }

                    match self.execute(body) {
                        ExecutionResult::Success => {}
                        ExecutionResult::Break => break,
                        ExecutionResult::RuntimeError(error) => {
                            return ExecutionResult::RuntimeError(error);
                        }
                        result @ ExecutionResult::Return(_) => return result,
                    }
                }

                ExecutionResult::Success
            }

            Stmt::Break => ExecutionResult::Break,

            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = LoxFunction::new(
                    Some(name.clone()),
                    parameters.clone(),
                    body.clone(),
                    self.environment.clone(),
                    false,
                );

                self.define_variable(name, Some(Value::Callable(Rc::new(function))));
                ExecutionResult::Success
            }

            Stmt::Class {
                name,
                superclass,
                members,
            } => {
                self.define_variable(name, None);

                let mut superclass_object = None;

                if let Some(super_expr) = superclass {
                    let super_value = match self.evaluate(super_expr) {
                        Ok(value) => value,
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    };

                    match super_value {
                        Value::Class(class) => superclass_object = Some(class),

                        _ => {
                            let error_token = match super_expr {
                                Expr::Variable { name: super_name, .. } => super_name.clone(),
                                _ => name.clone(),
                            };

                            return ExecutionResult::RuntimeError(RuntimeError::new(
                                error_token,
                                "Superclass must be a class.",
                            ));
                        }
                    }
                }

                if let Some(ref super_class) = superclass_object {
                    let super_environment = Environment::from(self.environment.clone());

                    super_environment
                        .borrow_mut()
                        .define_local(Some(Value::Class(super_class.clone())));

                    self.environment = super_environment;
                }

                let mut instance_methods = HashMap::new();
                let mut getters = HashMap::new();
                let mut static_methods = HashMap::new();

                for member in members {
                    match member {
                        ClassMember::Method {
                            name: method_name,
                            parameters,
                            body,
                        } => {
                            let function = LoxFunction::new(
                                Some(method_name.clone()),
                                parameters.clone(),
                                body.clone(),
                                self.environment.clone(),
                                method_name.lexeme == "init",
                            );

                            instance_methods
                                .insert(method_name.lexeme.clone(), Rc::new(function));
                        }

                        ClassMember::StaticMethod {
                            name: method_name,
                            parameters,
                            body,
                        } => {
                            let function = LoxFunction::new(
                                Some(method_name.clone()),
                                parameters.clone(),
                                body.clone(),
                                self.environment.clone(),
                                false,
                            );

                            static_methods.insert(method_name.lexeme.clone(), Rc::new(function));
                        }

                        ClassMember::Getter {
                            name: method_name,
                            body,
                        } => {
                            let function = LoxFunction::new(
                                Some(method_name.clone()),
                                Rc::new(Vec::new()),
                                body.clone(),
                                self.environment.clone(),
                                false,
                            );

                            getters.insert(method_name.lexeme.clone(), Rc::new(function));
                        }
                    }
                }

                let superclass = superclass_object
                    .as_ref()
                    .map(|class| class.instance_class().clone());

                let instance_class = Rc::new(lox_class::LoxClass::new(
                    name.lexeme.clone(),
                    superclass,
                    instance_methods,
                    getters,
                ));

                let metaclass = Rc::new(lox_class::LoxClass::new(
                    format!("{} metaclass", name.lexeme),
                    None,
                    static_methods,
                    HashMap::new(),
                ));

                let class_object =
                    Rc::new(lox_class::ClassObject::new(instance_class, metaclass));

                if superclass_object.is_some() {
                    let enclosing = self
                        .environment
                        .borrow()
                        .enclosing()
                        .expect("Superclass environment must have an enclosing scope");

                    self.environment = enclosing;
                }

                match self.assign_variable(name, Value::Class(class_object)) {
                    Ok(()) => ExecutionResult::Success,
                    Err(error) => ExecutionResult::RuntimeError(error),
                }
            }

            Stmt::Return { keyword, value } => {
                let return_value = match value {
                    Some(expression) => match self.evaluate(expression) {
                        Ok(value) => value,
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    },
                    None => Value::Literal(Literal::Nil),
                };

                ExecutionResult::Return(return_value)
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: EnvironmentRef,
    ) -> ExecutionResult {
        let previous = self.environment.clone();

        self.environment = environment;

        let result = (|| {
            for statement in statements {
                match self.execute(statement) {
                    ExecutionResult::Success => {}
                    result => return result,
                }
            }

            ExecutionResult::Success
        })();

        self.environment = previous;

        result
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value, RuntimeError> {
        match expression {
            Expr::Literal { value } => Ok(Value::Literal(value.clone())),

            Expr::Grouping { expression } => self.evaluate(expression),

            Expr::Variable { id, name } => self.look_up_variable(*id, name),

            Expr::Assign { id, name, value } => {
                let value = self.evaluate(value)?;

                if let Some(resolution) = self.locals.get(id) {
                    Environment::assign_at(
                        self.environment.clone(),
                        resolution.distance,
                        resolution.slot,
                        value.clone(),
                    );
                } else {
                    self.globals.borrow_mut().assign(name, value.clone())?;
                }

                Ok(value)
            }

            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Literal(Literal::Number(value)) => {
                            Ok(Value::Literal(Literal::Number(-value)))
                        }

                        _ => Err(RuntimeError::new(
                            operator.clone(),
                            "Operand must be a number.",
                        )),
                    },

                    TokenType::Bang => {
                        Ok(Value::Literal(Literal::Boolean(!self.is_truthy(&right))))
                    }

                    _ => unreachable!("Invalid unary operator"),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                self.evaluate_binary(&left, operator, &right)
            }

            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                if operator.token_type == TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }

                self.evaluate(right)
            }

            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;

                let mut arguments_values = Vec::new();

                for argument in arguments {
                    arguments_values.push(self.evaluate(argument)?);
                }

                match callee {
                    Value::Callable(function) => {
                        if arguments_values.len() != function.arity() {
                            return Err(RuntimeError::new(
                                paren.clone(),
                                format!(
                                    "Expected {} arguments but got {}.",
                                    function.arity(),
                                    arguments_values.len()
                                ),
                            ));
                        }

                        function.call(self, arguments_values)
                    }

                    Value::Class(class) => {
                        if arguments_values.len() != class.arity() {
                            return Err(RuntimeError::new(
                                paren.clone(),
                                format!(
                                    "Expected {} arguments but got {}.",
                                    class.arity(),
                                    arguments_values.len()
                                ),
                            ));
                        }

                        class.call(self, arguments_values)
                    }

                    _ => Err(RuntimeError::new(
                        paren.clone(),
                        "Can only call functions and classes.",
                    )),
                }
            }

            Expr::Get { object, name } => {
                let object = self.evaluate(object)?;

                match object {
                    Value::Class(class) => match class.find_static(&name.lexeme) {
                        Some(method) => Ok(Value::Callable(method)),

                        None => Err(RuntimeError::new(
                            name.clone(),
                            format!("Undefined property '{}'.", name.lexeme),
                        )),
                    },

                    Value::Instance(instance) => LoxInstance::get(self, instance, name),

                    _ => Err(RuntimeError::new(
                        name.clone(),
                        "Only instances have properties.",
                    )),
                }
            }

            Expr::This { id, keyword } => self.look_up_variable(*id, keyword),

            Expr::Super {
                id,
                keyword,
                method,
            } => {
                let resolution = self
                    .locals
                    .get(id)
                    .expect("Super expression must be resolved");

                let super_value = Environment::get_at(
                    self.environment.clone(),
                    resolution.distance,
                    resolution.slot,
                    keyword,
                )?;

                let superclass = match super_value {
                    Value::Class(class) => class,

                    _ => {
                        return Err(RuntimeError::new(
                            keyword.clone(),
                            "Superclass must be a class.",
                        ));
                    }
                };

                let this = Token::new(TokenType::This, "this".to_string(), None, 1);

                let this_value = Environment::get_at(
                    self.environment.clone(),
                    resolution.distance - 1,
                    0,
                    &this,
                )?;

                let instance = match this_value {
                    Value::Instance(instance) => instance,

                    _ => {
                        return Err(RuntimeError::new(
                            keyword.clone(),
                            "Super expression requires a bound instance.",
                        ));
                    }
                };

                match superclass.instance_class().find_method(&method.lexeme) {
                    Some(method_fn) => Ok(Value::Callable(method_fn.bind(instance))),

                    None => Err(RuntimeError::new(
                        method.clone(),
                        format!("Undefined property '{}'.", method.lexeme),
                    )),
                }
            }

            Expr::Set { object, name, value } => {
                let object = self.evaluate(object)?;

                let instance = match object {
                    Value::Instance(instance) => instance,

                    _ => {
                        return Err(RuntimeError::new(
                            name.clone(),
                            "Only instances have fields.",
                        ));
                    }
                };

                let value = self.evaluate(value)?;

                instance.borrow_mut().set(name, value.clone());

                Ok(value)
            }

            Expr::Function { params, body } => {
                let function = LoxFunction::new(
                    None,
                    params.clone(),
                    body.clone(),
                    self.environment.clone(),
                    false,
                );

                Ok(Value::Callable(Rc::new(function)))
            }
        }
    }

    fn look_up_variable(&self, id: ExprId, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(resolution) = self.locals.get(&id) {
            Environment::get_at(
                self.environment.clone(),
                resolution.distance,
                resolution.slot,
                name,
            )
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn evaluate_binary(
        &self,
        left: &Value,
        operator: &Token,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
        match operator.token_type {
            TokenType::Minus => self.number_binary(left, operator, right, |a, b| a - b),

            TokenType::Slash => {
                match right {
                    Value::Literal(Literal::Number(value)) => {
                        if *value == 0.0 {
                            return Err(RuntimeError::new(operator.clone(), "Division by zero."));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            operator.clone(),
                            "Operands must be numbers.",
                        ));
                    }
                };

                self.number_binary(left, operator, right, |a, b| a / b)
            }

            TokenType::Star => self.number_binary(left, operator, right, |a, b| a * b),

            TokenType::Plus => self.add(left, right, operator),

            TokenType::Greater => self.number_comparison(left, operator, right, |a, b| a > b),

            TokenType::GreaterEqual => self.number_comparison(left, operator, right, |a, b| a >= b),

            TokenType::Less => self.number_comparison(left, operator, right, |a, b| a < b),

            TokenType::LessEqual => self.number_comparison(left, operator, right, |a, b| a <= b),

            TokenType::EqualEqual => {
                Ok(Value::Literal(Literal::Boolean(self.is_equal(left, right))))
            }

            TokenType::BangEqual => Ok(Value::Literal(Literal::Boolean(
                !self.is_equal(left, right),
            ))),

            _ => unreachable!("Invalid binary operator"),
        }
    }

    fn add(&self, left: &Value, right: &Value, operator: &Token) -> Result<Value, RuntimeError> {
        match (left, right) {
            // Number + Number
            (Value::Literal(Literal::Number(left)), Value::Literal(Literal::Number(right))) => {
                Ok(Value::Literal(Literal::Number(left + right)))
            }

            // String + String
            (Value::Literal(Literal::String(left)), Value::Literal(Literal::String(right))) => Ok(
                Value::Literal(Literal::String(format!("{}{}", left, right))),
            ),

            // String + anything
            (Value::Literal(Literal::String(left)), right) => Ok(Value::Literal(Literal::String(
                format!("{}{}", left, self.stringify(right)),
            ))),

            // Anything + String
            (left, Value::Literal(Literal::String(right))) => Ok(Value::Literal(Literal::String(
                format!("{}{}", self.stringify(left), right),
            ))),

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be two numbers or at least one string.",
            )),
        }
    }

    fn number_binary<F>(
        &self,
        left: &Value,
        operator: &Token,
        right: &Value,
        operation: F,
    ) -> Result<Value, RuntimeError>
    where
        F: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(b))) => {
                Ok(Value::Literal(Literal::Number(operation(*a, *b))))
            }

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be numbers.",
            )),
        }
    }

    fn number_comparison<F>(
        &self,
        left: &Value,
        operator: &Token,
        right: &Value,
        operation: F,
    ) -> Result<Value, RuntimeError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(b))) => {
                Ok(Value::Literal(Literal::Boolean(operation(*a, *b))))
            }

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be numbers.",
            )),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Literal(Literal::Nil) => false,
            Value::Literal(Literal::Boolean(value)) => *value,
            _ => true,
        }
    }

    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Literal(Literal::Nil), Value::Literal(Literal::Nil)) => true,

            (Value::Literal(Literal::Nil), _) => false,

            (_, Value::Literal(Literal::Nil)) => false,

            (Value::Literal(left), Value::Literal(right)) => left == right,

            _ => false,
        }
    }

    pub fn stringify(&self, value: &Value) -> String {
        match value {
            Value::Literal(Literal::Number(value)) => {
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    value.to_string()
                }
            }

            Value::Literal(Literal::String(value)) => value.clone(),

            Value::Literal(Literal::Boolean(value)) => value.to_string(),

            Value::Literal(Literal::Nil) => "nil".to_string(),

            Value::Callable(callable) => format!("<fn {}>", callable.name()),

            Value::Class(class) => class.name().to_string(),

            Value::Instance(instance) => {
                format!("{} instance", instance.borrow().klass_name())
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: impl Into<String>) -> Self {
        Self {
            token,
            message: message.into(),
        }
    }
}

pub enum ExecutionResult {
    Success,
    RuntimeError(RuntimeError),
    Break,
    Return(Value),
}

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;

    fn arity(&self) -> usize;

    fn name(&self) -> String;
}

struct NativeClock;

impl LoxCallable for NativeClock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Ok(Value::Literal(Literal::Number(time)))
    }

    fn name(&self) -> String {
        "clock".to_string()
    }
}

#[cfg(test)]
mod tests;

pub mod lox_class;
pub mod lox_instance;

pub use lox_class::ClassObject;
mod lox_function;
