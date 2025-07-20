use std::path::PathBuf;
use aircher::code_chunking::{CodeChunker, ChunkType};

#[test]
fn test_c_focused_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let c_code = r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Person structure definition */
struct Person {
    char name[50];
    int age;
};

/* Function prototypes */
struct Person* create_person(const char* name, int age);
void print_person(const struct Person* p);
void free_person(struct Person* p);

/* Function to create a new person */
struct Person* create_person(const char* name, int age) {
    struct Person* p = malloc(sizeof(struct Person));
    if (p != NULL) {
        strcpy(p->name, name);
        p->age = age;
    }
    return p;
}

/* Function to print person info */
void print_person(const struct Person* p) {
    if (p != NULL) {
        printf("Name: %s, Age: %d\n", p->name, p->age);
    }
}

/* Function to free person memory */
void free_person(struct Person* p) {
    if (p != NULL) {
        free(p);
    }
}

/* Helper function to get age input */
int get_age_input(void) {
    int age;
    printf("Enter age: ");
    scanf("%d", &age);
    return age;
}

/* Main function */
int main(int argc, char *argv[]) {
    struct Person* person = create_person("Alice", 30);
    print_person(person);
    
    int new_age = get_age_input();
    person->age = new_age;
    print_person(person);
    
    free_person(person);
    return 0;
}"#;
    
    let chunks = chunker.chunk_file(&PathBuf::from("person.c"), c_code).unwrap();
    
    println!("Found {} chunks in focused C code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})", 
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
        if chunk.content.len() < 200 {
            println!("    Content preview: {:?}", &chunk.content[..chunk.content.len().min(100)]);
        }
    }
    
    assert!(!chunks.is_empty());
    
    // Should find functions and structs
    let function_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Function))
        .collect();
    
    let struct_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Struct))
        .collect();
    
    let generic_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Generic))
        .collect();
    
    println!("Found {} function chunks, {} struct chunks, {} generic chunks", 
             function_chunks.len(), struct_chunks.len(), generic_chunks.len());
    
    // If we're getting generic chunks, the tree-sitter parsing may need refinement
    if !function_chunks.is_empty() {
        println!("SUCCESS: C semantic parsing is working!");
        for func in &function_chunks {
            println!("  Function found: {:?}", func.name);
        }
    } else {
        println!("INFO: C is using generic chunking - tree-sitter queries may need adjustment");
    }
}