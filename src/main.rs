use std::{
    env::args,
    fs::{read_to_string, File},
    io::{stdout, BufWriter, Write},
};

use span::Source;
use table::SymbolTable;

mod lexer;
mod parser;
mod span;
mod table;

enum OutputType {
    File,
    Stdout,
    FileAndStdout,
}

enum Output {
    File(BufWriter<File>),
    Stdout,
    FileAndStdout(BufWriter<File>),
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Output::File(file) => file.write(buf),
            Output::Stdout => std::io::stdout().write(buf),
            Output::FileAndStdout(file) => stdout().write(buf).and_then(|_| file.write(buf)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Output::File(file) => file.flush(),
            Output::Stdout => std::io::stdout().flush(),
            Output::FileAndStdout(file) => std::io::stdout().flush().and_then(|_| file.flush()),
        }
    }
}

fn usage(name: &str) {
    println!("Usage: {} <output> <file1> <file2> ...", name);
    println!("    <output> is: -f OR -t OR -ft");
    println!("        -f: output tokens and table to <file>.output for each given file");
    println!("        -t: output tokens and table to stdout for each given file");
    println!("        -ft: both -f and -t");
    println!("    <file1> <file2> ...: files to parse");
}

fn main() {
    let args = args().collect::<Vec<_>>();
    // let args = vec![
    //     "main".to_string(),
    //     "-ft".to_string(),
    //     "file6.txt".to_string(),
    // ];

    let output_type = match args.get(1).map(|s| s.as_str()) {
        Some("-f") => OutputType::File,
        Some("-t") => OutputType::Stdout,
        Some("-ft" | "-tf") => OutputType::FileAndStdout,
        Some(arg) => {
            println!("Invalid output option: {}", arg);
            usage(&args[0]);
            return;
        }
        None => {
            println!("No output option given");
            usage(&args[0]);
            return;
        }
    };

    let files = &args[2..];

    if files.is_empty() {
        println!("No files given");
        usage(&args[0]);
        return;
    }

    for filename in files {
        println!("Parsing {}", filename);
        println!("{:-<1$}", "", filename.len() + 8);

        let content = match read_to_string(filename) {
            Ok(content) => content,
            Err(err) => {
                println!("{}: {}", filename, err);
                println!("Could not read file {}", filename);
                println!("{:-<1$}", "", filename.len() + 20);
                continue;
            }
        };

        let src = Source {
            filename: filename.to_string(),
            content,
        };

        match lexer::parse(&src) {
            Ok((mut table, tokens)) => match parser::parse(&tokens, &mut table) {
                Ok(decls) => {
                    let mut out = match output_type {
                        OutputType::File => Output::File(BufWriter::new(
                            File::create(format!("{}.output", filename))
                                .expect(&format!("Could not create file {}.output", filename)),
                        )),
                        OutputType::Stdout => Output::Stdout,
                        OutputType::FileAndStdout => Output::FileAndStdout(BufWriter::new(
                            File::create(format!("{}.output", filename))
                                .expect(&format!("Could not create file {}.output", filename)),
                        )),
                    };

                    for token in &tokens {
                        writeln!(out, "{:?}", token)
                            .expect(&format!("Could not write to file {}.output", filename));
                    }

                    writeln!(out).expect("Could not write to file");

                    out.flush()
                        .expect(&format!("Could not write to file {}.output", filename));

                    for decl in &decls {
                        writeln!(
                            out,
                            "{}:\n{}\n--\n{:#?}\n",
                            decl.span.location(),
                            decl.span.fragment(),
                            decl.value,
                        )
                        .expect(&format!("Could not write to file {}.output", filename));
                    }

                    writeln!(out).expect("Could not write to file");

                    write_table(&mut out, &table);

                    out.flush()
                        .expect(&format!("Could not write to file {}.output", filename));

                    println!("Finished parsing {} with success", filename);
                    println!("{:-<1$}", "", filename.len() + 30);
                }
                Err(err) => {
                    println!("Parser error at {:?}", err);
                    println!("Finished parsing {} with failure", filename);
                    println!("{:-<1$}", "", filename.len() + 30);
                }
            },
            Err(err) => {
                println!("Lexer error at {}: {:?}", err.span.location(), err.value);
                println!("Finished parsing {} with failure", filename);
                println!("{:-<1$}", "", filename.len() + 30);
            }
        }
    }
}

fn write_table(f: &mut impl Write, table: &SymbolTable) {
    let mut symbols = table.symbols().iter().collect::<Vec<_>>();
    symbols.sort_by(|(a, _), (b, _)| a.cmp(b));

    if let Some((index_len, type_len, id_len)) = symbols
        .iter()
        .map(|(index, symbol)| {
            let index_len = index.to_string().len();
            let type_len = format!("{:?}", symbol.type_()).len();
            let id_len = symbol.id().len();

            (index_len, type_len, id_len)
        })
        .reduce(|(a1, b1, c1), (a2, b2, c2)| (a1.max(a2), b1.max(b2), c1.max(c2)))
    {
        let index_title = format!("{:1$}", "Index", index_len);
        let type_title = format!("{:1$}", "Type", type_len);
        let id_title = format!("{:1$}", "ID", id_len);

        let index_len = index_title.len().max(index_len);
        let type_len = type_title.len().max(type_len);
        let id_len = id_title.len().max(id_len);

        writeln!(f, "{:-<1$}", "", index_len + type_len + id_len + 4)
            .expect("Could not write to file");
        writeln!(f, "|{}|{}|{}|", index_title, type_title, id_title)
            .expect("Could not write to file");
        writeln!(f, "{:-<1$}", "", index_len + type_len + id_len + 4)
            .expect("Could not write to file");

        for (index, symbol) in symbols {
            let index = format!("{:<1$}", index, index_len);
            let type_ = format!("{:<1$}", format!("{:?}", symbol.type_()), type_len);
            let id = format!("{:<1$}", symbol.id(), id_len);

            writeln!(f, "|{}|{}|{}|", index, type_, id).expect("Could not write to file");
        }

        writeln!(f, "{:-<1$}", "", index_len + type_len + id_len + 4)
            .expect("Could not write to file");
    } else {
        writeln!(f, "No symbols.").expect("Could not write to file");
    }
}
