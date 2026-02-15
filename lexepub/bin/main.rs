use lexepub::prelude::*;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <epub_file>", args[0]);
        eprintln!("Example: {} examples/epubs/test-book.epub", args[0]);
        std::process::exit(1);
    }

    let epub_path = Path::new(&args[1]);

    if !epub_path.exists() {
        eprintln!("Error: File '{}' does not exist", epub_path.display());
        std::process::exit(1);
    }

    println!("Analyzing EPUB: {}", epub_path.display());
    println!("{}", "=".repeat(60));

    // Open the EPUB
    let mut epub = LexEpub::open(epub_path).await?;

    // Extract and display metadata
    println!("METADATA");
    println!("{}", "-".repeat(20));
    let metadata = epub.get_metadata().await?;

    if let Some(title) = &metadata.title {
        println!("Title: {}", title);
    } else {
        println!("Title: (unknown)");
    }

    if !metadata.authors.is_empty() {
        println!("Authors: {}", metadata.authors.join(", "));
    }

    if !metadata.languages.is_empty() {
        println!("Languages: {}", metadata.languages.join(", "));
    }

    if let Some(publisher) = &metadata.publisher {
        println!("Publisher: {}", publisher);
    }

    if let Some(date) = &metadata.date {
        println!("Publication Date: {}", date);
    }

    // Extract text content statistics
    println!("\nCONTENT STATISTICS");
    println!("{}", "-".repeat(25));

    let chapters = epub.extract_text_only().await?;
    println!("Chapters: {}", chapters.len());

    let total_words = epub.total_word_count().await?;
    let total_chars = epub.total_char_count().await?;
    println!("Total Words: {}", total_words);
    println!("Total Characters: {}", total_chars);

    // Check for cover image
    // TODO: Implement has_cover method
    // let has_cover = epub.has_cover().await?;
    // println!("Has Cover Image: {}", if has_cover { "Yes" } else { "No" });

    // Show first chapter preview
    if let Some(first_chapter) = chapters.first() {
        println!("\nFIRST CHAPTER PREVIEW");
        println!("{}", "-".repeat(27));
        let preview: String = first_chapter.chars().take(300).collect();
        println!("{}\n...", preview.trim());
    }

    // Extract with AST for advanced processing
    println!("\nAST ANALYSIS");
    println!("{}", "-".repeat(15));
    let ast_chapters = epub.extract_ast().await?;
    println!("Chapters with AST: {}", ast_chapters.len());

    if let Some(first_ast_chapter) = ast_chapters.first() {
        if let Some(_ast) = &first_ast_chapter.ast {
            println!("First chapter AST available: Yes");
            println!(
                "Content length: {} characters",
                first_ast_chapter.content.len()
            );
        } else {
            println!("First chapter AST available: No");
        }
    }

    println!("\nEPUB analysis completed successfully!");
    println!("{}", "=".repeat(60));

    Ok(())
}
