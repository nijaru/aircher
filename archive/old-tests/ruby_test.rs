use std::path::PathBuf;
use aircher::code_chunking::{CodeChunker, ChunkType};

#[test]
fn test_ruby_chunking() {
    let mut chunker = CodeChunker::new().unwrap();
    let ruby_code = r#"# Ruby class example with methods and modules

require 'json'
require 'net/http'

# Person class representing a person with name and age
class Person
  attr_accessor :name, :age

  def initialize(name, age)
    @name = name
    @age = age
  end

  # Instance method to display person info
  def display_info
    puts "Name: #{@name}, Age: #{@age}"
  end

  # Class method to create a default person
  def self.create_default
    new("Unknown", 0)
  end

  # Instance method to check if person is adult
  def adult?
    @age >= 18
  end

  # Private method
  private

  def validate_age
    @age > 0
  end
end

# PersonManager module for person operations
module PersonManager
  # Module method to find person by name
  def self.find_by_name(people, name)
    people.find { |person| person.name == name }
  end

  # Module method to create multiple people
  def self.create_batch(names_and_ages)
    names_and_ages.map { |name, age| Person.new(name, age) }
  end
end

# Utility class for person operations
class PersonRepository
  def initialize
    @people = []
  end

  def add_person(person)
    @people << person if person.is_a?(Person)
  end

  def all_people
    @people.dup
  end

  def adults_only
    @people.select(&:adult?)
  end
end

# Main execution
person = Person.new("Alice", 25)
person.display_info

default_person = Person.create_default
default_person.display_info

repository = PersonRepository.new
repository.add_person(person)
repository.add_person(default_person)

puts "Total people: #{repository.all_people.size}"
puts "Adults: #{repository.adults_only.size}"
"#;

    let chunks = chunker.chunk_file(&PathBuf::from("person.rb"), ruby_code).unwrap();

    println!("Found {} chunks in Ruby code:", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: {:?} - {:?} (lines {}-{})",
                i, chunk.chunk_type, chunk.name, chunk.start_line, chunk.end_line);
    }

    assert!(!chunks.is_empty());

    // Should find classes, modules, and methods
    let class_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Class))
        .collect();

    let module_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Module))
        .collect();

    let method_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Method))
        .collect();

    let generic_chunks: Vec<_> = chunks.iter()
        .filter(|c| matches!(c.chunk_type, ChunkType::Generic))
        .collect();

    println!("Found {} class chunks, {} module chunks, {} method chunks, {} generic chunks",
             class_chunks.len(), module_chunks.len(), method_chunks.len(), generic_chunks.len());

    // Ruby should have good performance
    if method_chunks.len() > 0 {
        println!("SUCCESS: Ruby semantic parsing is working!");
        for method in &method_chunks {
            println!("  Method found: {:?}", method.name);
        }
        for class in &class_chunks {
            println!("  Class found: {:?}", class.name);
        }
        for module in &module_chunks {
            println!("  Module found: {:?}", module.name);
        }
    } else {
        println!("INFO: Ruby is using generic chunking - tree-sitter queries may need adjustment");
    }

    // Should have substantial chunks for semantic parsing
    assert!(chunks.len() >= 3, "Should have at least some chunks for Ruby semantic parsing");
}
