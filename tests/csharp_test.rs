use std::path::PathBuf;
use aircher::code_chunking::{CodeChunker, ChunkType};

#[test]
fn test_csharp_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let csharp_code = r#"using System;
using System.Collections.Generic;

namespace PersonDemo
{
    /// <summary>
    /// Represents a person with name and age
    /// </summary>
    public class Person
    {
        private string name;
        private int age;
        
        // Constructor
        public Person(string name, int age)
        {
            this.name = name;
            this.age = age;
        }
        
        // Properties
        public string Name 
        { 
            get { return name; } 
            set { name = value; } 
        }
        
        public int Age 
        { 
            get { return age; } 
            set { age = value; } 
        }
        
        // Methods
        public void DisplayInfo()
        {
            Console.WriteLine($"Name: {name}, Age: {age}");
        }
        
        public static Person CreateDefault()
        {
            return new Person("Unknown", 0);
        }
    }

    /// <summary>
    /// Interface for person operations
    /// </summary>
    public interface IPersonOperations
    {
        void UpdateAge(int newAge);
        bool IsAdult();
    }

    /// <summary>
    /// Struct representing a simple point
    /// </summary>
    public struct Point
    {
        public int X { get; set; }
        public int Y { get; set; }
        
        public Point(int x, int y)
        {
            X = x;
            Y = y;
        }
    }

    /// <summary>
    /// Main program class
    /// </summary>
    public class Program
    {
        public static void Main(string[] args)
        {
            Person person = new Person("Alice", 25);
            person.DisplayInfo();
            
            Person defaultPerson = Person.CreateDefault();
            defaultPerson.DisplayInfo();
            
            Point point = new Point(10, 20);
            Console.WriteLine($"Point: ({point.X}, {point.Y})");
        }
        
        private static void PrintWelcome()
        {
            Console.WriteLine("Welcome to the C# Person Demo!");
        }
    }
}"#;
    
    let chunks = chunker.chunk_file(&PathBuf::from("Program.cs"), csharp_code).unwrap();
    
    println!("Found {} chunks in C# code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})", 
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
    }
    
    assert!(!chunks.is_empty());
    
    // Should find classes, interfaces, structs, and methods
    let class_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Class))
        .collect();
    
    let interface_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Interface))
        .collect();
    
    let struct_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Struct))
        .collect();
    
    let method_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Method))
        .collect();
    
    println!("Found {} class chunks, {} interface chunks, {} struct chunks, and {} method chunks", 
             class_chunks.len(), interface_chunks.len(), struct_chunks.len(), method_chunks.len());
    
    // C# should have excellent performance like Java
    assert!(chunks.len() >= 5, "Should have substantial chunks for C# semantic parsing");
}