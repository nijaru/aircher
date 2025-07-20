use std::path::PathBuf;
use aircher::code_chunking::{CodeChunker, ChunkType};

#[test]
fn test_java_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let java_code = r#"package com.example.demo;

import java.util.List;
import java.util.ArrayList;

/**
 * Person class representing a person with name and age
 */
public class Person {
    private String name;
    private int age;
    
    // Constructor
    public Person(String name, int age) {
        this.name = name;
        this.age = age;
    }
    
    // Getter methods
    public String getName() {
        return name;
    }
    
    public int getAge() {
        return age;
    }
    
    // Method to display info
    public void displayInfo() {
        System.out.println("Name: " + name + ", Age: " + age);
    }
    
    // Static method
    public static Person createDefault() {
        return new Person("Unknown", 0);
    }
}

/**
 * Utility interface for common operations
 */
public interface PersonOperations {
    void updateAge(int newAge);
    boolean isAdult();
}

/**
 * Main class with application entry point
 */
public class Application {
    public static void main(String[] args) {
        Person person = new Person("Alice", 25);
        person.displayInfo();
        
        Person defaultPerson = Person.createDefault();
        defaultPerson.displayInfo();
    }
    
    // Helper method
    private static void printWelcome() {
        System.out.println("Welcome to the Person Demo!");
    }
}"#;
    
    let chunks = chunker.chunk_file(&PathBuf::from("Application.java"), java_code).unwrap();
    
    println!("Found {} chunks in Java code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})", 
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
    }
    
    assert!(!chunks.is_empty());
    
    // Should find classes, interfaces, and methods
    let class_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Class))
        .collect();
    
    let interface_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Interface))
        .collect();
    
    let method_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Method))
        .collect();
    
    println!("Found {} class chunks, {} interface chunks, and {} method chunks", 
             class_chunks.len(), interface_chunks.len(), method_chunks.len());
    
    // Should have at least some chunks - Java parsing should work well
    assert!(chunks.len() >= 3, "Should have at least some chunks");
}