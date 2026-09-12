use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{
        Interpreter, LoxCallable, RuntimeError, lox_function::LoxFunction, lox_instance::LoxInstance,
    },
    token::Value,
};

pub struct LoxClass {
    name: String,
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
    getters: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
        getters: HashMap<String, Rc<LoxFunction>>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
            getters,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }

        if let Some(superclass) = &self.superclass {
            return superclass.find_method(name);
        }

        None
    }

    pub fn find_getter(&self, name: &str) -> Option<Rc<LoxFunction>> {
        if let Some(getter) = self.getters.get(name) {
            return Some(getter.clone());
        }

        if let Some(superclass) = &self.superclass {
            return superclass.find_getter(name);
        }

        None
    }

    pub fn arity(&self) -> usize {
        match self.find_method("init") {
            Some(initializer) => initializer.arity(),
            None => self
                .superclass
                .as_ref()
                .map(|superclass| superclass.arity())
                .unwrap_or(0),
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
        } else if let Some(superclass) = &self.superclass {
            initializer_chain(superclass, interpreter, instance.clone(), arguments)?;
        }

        Ok(Value::Instance(instance))
    }
}

fn initializer_chain(
    superclass: &Rc<LoxClass>,
    interpreter: &mut Interpreter,
    instance: Rc<RefCell<LoxInstance>>,
    arguments: Vec<Value>,
) -> Result<(), RuntimeError> {
    if let Some(initializer) = superclass.find_method("init") {
        initializer.bind(instance).call(interpreter, arguments)?;
    } else if let Some(enclosing) = &superclass.superclass {
        initializer_chain(enclosing, interpreter, instance, arguments)?;
    }

    Ok(())
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

    pub fn instance_class(&self) -> &Rc<LoxClass> {
        &self.instance_class
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
