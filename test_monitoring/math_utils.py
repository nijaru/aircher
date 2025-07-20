"""Mathematical utility functions for testing semantic search"""

def fibonacci(n):
    """Generate nth Fibonacci number"""
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def factorial(n):
    """Calculate factorial of n"""
    if n <= 1:
        return 1
    return n * factorial(n-1)

class MathOperations:
    """Collection of mathematical operations"""
    
    def __init__(self):
        self.history = []
    
    def add_numbers(self, a, b):
        """Add two numbers and store in history"""
        result = a + b
        self.history.append(f"{a} + {b} = {result}")
        return result
    
    def divide_numbers(self, a, b):
        """Divide two numbers with error handling"""
        if b == 0:
            raise ValueError("Cannot divide by zero")
        result = a / b
        self.history.append(f"{a} / {b} = {result}")
        return result
EOF < /dev/null