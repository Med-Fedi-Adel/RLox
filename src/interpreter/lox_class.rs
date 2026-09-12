use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{
        Interpreter, LoxCallable, RuntimeError, lox_function::LoxFunction, lox_instance::LoxInstance,
    },
    token::Value,
};

pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
    getters: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        methods: HashMap<String, Rc<LoxFunction>>,
        getters: HashMap<String, Rc<LoxFunction>>,
    ) -> Self {
        Self {
            name,
            methods,
            getters,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.methods.get(name).cloned()
    }

    pub fn find_getter(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.getters.get(name).cloned()
    }

    pub fn arity(&self) -> usize {
        match self.find_method("init") {
            Some(initializer) => initializer.arity(),
            None => 0,
        }
    }

    pub fn call(
        self: &Rc<Self>,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));

        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }

        Ok(Value::Instance(instance))
    }
}

pub struct ClassObject {
    instance_class: Rc<LoxClass>,
    metaclass: Rc<LoxClass>,
}

impl ClassObject {
    pub fn new(instance_class: Rc<LoxClass>, metaclass: Rc<LoxClass>) -> Self {
        Self {
            instance_class,
            metaclass,
        }
    }

    pub fn name(&self) -> &str {
        self.instance_class.name()
    }

    pub fn find_static(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.metaclass.find_method(name)
    }

    pub fn arity(&self) -> usize {
        self.instance_class.arity()
    }

    pub fn call(
        self: &Rc<Self>,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        self.instance_class.call(interpreter, arguments)
    }
}
