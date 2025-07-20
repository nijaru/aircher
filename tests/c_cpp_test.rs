use std::path::PathBuf;
use aircher::code_chunking::{CodeChunker, ChunkType};

#[test]
fn test_c_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let c_code = r#"#include <stdio.h>
#include <stdlib.h>

// Person structure
struct Person {
    char name[50];
    int age;
};

// Function to create a new person
struct Person* create_person(const char* name, int age) {
    struct Person* p = malloc(sizeof(struct Person));
    strcpy(p->name, name);
    p->age = age;
    return p;
}

// Function to print person info
void print_person(const struct Person* p) {
    printf("Name: %s, Age: %d\n", p->name, p->age);
}

int main() {
    struct Person* person = create_person("Alice", 30);
    print_person(person);
    free(person);
    return 0;
}"#;
    
    let chunks = chunker.chunk_file(&PathBuf::from("test.c"), c_code).unwrap();
    
    println!("Found {} chunks in C code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})", 
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
    }
    
    assert!(!chunks.is_empty());
    
    // Should find functions and structs
    let function_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Function))
        .collect();
    
    println!("Found {} function chunks", function_chunks.len());
    assert!(chunks.len() >= 2, "Should have at least some chunks");
}

#[test]
fn test_cpp_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let cpp_code = r#"#include <iostream>
#include <string>

// Person class
class Person {
private:
    std::string name;
    int age;
    
public:
    // Constructor
    Person(const std::string& name, int age) : name(name), age(age) {}
    
    // Getter methods
    std::string getName() const { return name; }
    int getAge() const { return age; }
    
    // Method to display info
    void displayInfo() const {
        std::cout << "Name: " << name << ", Age: " << age << std::endl;
    }
};

// Template function
template<typename T>
T max(T a, T b) {
    return (a > b) ? a : b;
}

int main() {
    Person person("Bob", 25);
    person.displayInfo();
    
    int result = max(10, 20);
    std::cout << "Max: " << result << std::endl;
    
    return 0;
}"#;
    
    let chunks = chunker.chunk_file(&PathBuf::from("test.cpp"), cpp_code).unwrap();
    
    println!("Found {} chunks in C++ code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})", 
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
    }
    
    assert!(!chunks.is_empty());
    
    // Should find classes and functions
    let class_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Class))
        .collect();
    
    let function_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Function))
        .collect();
    
    println!("Found {} class chunks and {} function chunks", 
             class_chunks.len(), function_chunks.len());
    
    assert!(chunks.len() >= 2, "Should have at least some chunks");
}