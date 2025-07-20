// Test Rust file for semantic search
use std::collections::HashMap;

pub struct Calculator {
    memory: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }
    
    pub fn add(&mut self, x: f64, y: f64) -> f64 {
        let result = x + y;
        self.memory = result;
        result
    }
    
    pub fn multiply(&mut self, x: f64, y: f64) -> f64 {
        let result = x * y;
        self.memory = result;
        result
    }
}

fn main() {
    let mut calc = Calculator::new();
    println!("2 + 3 = {}", calc.add(2.0, 3.0));
    println!("4 * 5 = {}", calc.multiply(4.0, 5.0));
}

pub fn power(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

pub fn square_root(n: f64) -> f64 {
    n.sqrt()
}