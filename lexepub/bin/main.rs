use lexepub::LexEpub;

#[cfg(feature = "embassy")]
#[embassy_executor::main]
async fn main(
    _spawner: embassy_executor::Spawner,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <epub_file>", args[0]);
        eprintln!("Example: {} examples/epubs/test-book.epub", args[0]);
        std::process::exit(1);
    }

    let epub_path = std::path::Path::new(&args[1]);

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

    println!("\nEPUB analysis completed successfully!");
    println!("{}", "=".repeat(60));

    Ok(())
}

#[cfg(not(feature = "embassy"))]
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <epub_file>", args[0]);
        eprintln!("Example: {} examples/epubs/test-book.epub", args[0]);
        std::process::exit(1);
    }

    let epub_path = std::path::Path::new(&args[1]);

    if !epub_path.exists() {
        eprintln!("Error: File '{}' does not exist", epub_path.display());
        std::process::exit(1);
    }

    println!("Analyzing EPUB: {}", epub_path.display());
    println!("{}", "=".repeat(60));

    // Open the EPUB
    let mut epub = futures::executor::block_on(LexEpub::open(epub_path))?;

    // Extract and display metadata
    println!("METADATA");
    println!("{}", "-".repeat(20));
    let metadata = futures::executor::block_on(epub.get_metadata())?;

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

    let chapters = futures::executor::block_on(epub.extract_text_only())?;
    println!("Chapters: {}", chapters.len());

    let total_words = futures::executor::block_on(epub.total_word_count())?;
    let total_chars = futures::executor::block_on(epub.total_char_count())?;
    println!("Total Words: {}", total_words);
    println!("Total Characters: {}", total_chars);

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
    let ast_chapters = futures::executor::block_on(epub.extract_ast())?;
    println!("Chapters with AST: {}", ast_chapters.len());

    println!("\nEPUB analysis completed successfully!");
    println!("{}", "=".repeat(60));

    Ok(())
}
