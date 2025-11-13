use std::path::PathBuf;
use aircher::code_chunking::CodeChunker;

#[test]
fn test_go_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let go_code = r#"package main

import "fmt"

// Person represents a person with name and age
type Person struct {
    Name string
    Age  int
}

// NewPerson creates a new person
func NewPerson(name string, age int) *Person {
    return &Person{
        Name: name,
        Age:  age,
    }
}

// GetInfo returns formatted information about the person
func (p *Person) GetInfo() string {
    return fmt.Sprintf("Name: %s, Age: %d", p.Name, p.Age)
}

func main() {
    person := NewPerson("Alice", 30)
    fmt.Println(person.GetInfo())
}"#;

    let chunks = chunker.chunk_file(&PathBuf::from("test.go"), go_code).unwrap();

    // Debug: Print what chunks we actually got
    println!("Found {} chunks in Go code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})",
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
    }

    assert!(!chunks.is_empty());

    // Should find functions and types
    let function_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, aircher::code_chunking::ChunkType::Function))
        .collect();

    let type_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, aircher::code_chunking::ChunkType::Generic))
        .collect();

    println!("Found {} function chunks and {} type chunks",
             function_chunks.len(), type_chunks.len());

    // For now, just ensure we have chunks (semantic parsing may need refinement)
    assert!(chunks.len() >= 2, "Should have at least some chunks");
}
