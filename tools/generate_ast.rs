use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    process,
};

struct AstType {
    name: &'static str,
    fields: &'static [(&'static str, &'static str)],
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: generate_ast <output directory>");
        process::exit(64);
    }

    let output_dir = &args[1];

    fs::create_dir_all(output_dir)?;

    define_expr(output_dir)?;

    Ok(())
}

fn define_expr(output_dir: &str) -> io::Result<()> {
    let path = format!("{}/mod.rs", output_dir);
    let mut writer = File::create(path)?;

    let types = [
        AstType {
            name: "Binary",
            fields: &[
                ("Box<Expr>", "left"),
                ("Token", "operator"),
                ("Box<Expr>", "right"),
            ],
        },
        AstType {
            name: "Grouping",
            fields: &[("Box<Expr>", "expression")],
        },
        AstType {
            name: "Literal",
            fields: &[("Literal", "value")],
        },
        AstType {
            name: "Unary",
            fields: &[("Token", "operator"), ("Box<Expr>", "right")],
        },
    ];

    writeln!(writer, "use crate::token::{{Literal, Token}};")?;
    writeln!(writer)?;
    writeln!(writer, "#[derive(Debug, Clone, PartialEq)]")?;
    writeln!(writer, "pub enum Expr {{")?;

    for ast_type in &types {
        define_type(&mut writer, ast_type)?;
    }

    writeln!(writer, "}}")?;
    writeln!(writer)?;
    writeln!(writer, "#[cfg(test)]")?;
    writeln!(writer, "mod tests;")?;

    Ok(())
}

fn define_type(writer: &mut File, ast_type: &AstType) -> io::Result<()> {
    writeln!(writer, "    {} {{", ast_type.name)?;

    for (field_type, field_name) in ast_type.fields {
        writeln!(writer, "        {}: {},", field_name, field_type)?;
    }

    writeln!(writer, "    }},")?;
    writeln!(writer)?;

    Ok(())
}
