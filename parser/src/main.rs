use arch::{
    combineArcHs, convertToOriginalForm, markAsSingleChild, ArcH, Fish, OriginalArcHForm, Vertex,
};
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
extern crate regex;
use regex::Regex;

mod arch;

// Custom error type for parsing
#[derive(Debug)]
enum ParseError {
    UnexpectedEndOfInput,
    IndentationMismatch,
    UnexpectedIndentation,
    MissingFish,
    InvalidSyntax(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::IndentationMismatch => write!(f, "Indentation mismatch"),
            ParseError::UnexpectedIndentation => write!(f, "Unexpected indentation"),
            ParseError::MissingFish => write!(f, "Missing fish operator (><)"),
            ParseError::InvalidSyntax(s) => write!(f, "Invalid syntax: {}", s),
        }
    }
}

macro_rules! println_ {
    ($($arg:tt)*) => {{
        // ----- uncomment to show logs at run time
        // print!("{}\n", format!($($arg)*));
    }};
}

impl Error for ParseError {}

fn parse_inputs(input: &str) -> HashMap<&str, Vec<ArcH>> {
    let mut files: HashMap<&str, Vec<&str>> = HashMap::new();

    let mut lines: Vec<&str> = input.lines().collect();

    let mut name = "";
    let mut linesCollected = Vec::new();
    let mut n = 0;
    while (n < lines.len()) {
        //     // check if starts/ends with []
        let line = lines[n].trim();
        if line.starts_with("[") && line.ends_with("]") {
            if (!linesCollected.is_empty()) {
                files.insert(name, linesCollected.clone());
            }
            name = line[1..line.len() - 1].trim();
            linesCollected = Vec::new();
        } else {
            linesCollected.push(line);
        }
        n = n + 1;
    }

    if (!linesCollected.is_empty()) {
        files.insert(name, linesCollected.clone());
    }

    let mut finalResult: HashMap<&str, Vec<ArcH>> = HashMap::new();

    for (_key, values) in files.iter_mut() {
        let parsed = parse_input(values.to_vec());
        finalResult.insert(_key, parsed.unwrap());
    }
    return finalResult;
    // return files;
}

fn collect_inputs(input: HashMap<&str, Vec<ArcH>>) -> Vec<Vec<OriginalArcHForm>> {
    let mut collectedArcH = Vec::new();
    for (_key, values) in input.iter() {
        // key as lines

        let prefix = parse_arch(&[_key], 0, 0);

        let ar0 = convertToOriginalForm(None, prefix.unwrap().0);
        //
        // concat map
        let ars = values
            .iter()
            .map(|a| convertToOriginalForm(ar0.first().map(|ref_val| ref_val.clone()), a.clone()));

        collectedArcH.extend(ars);
    }
    return collectedArcH;
}

// Function to parse the entire input into an ArcH
fn parse_input(lines_: Vec<&str>) -> Result<Vec<ArcH>, ParseError> {
    let mut lines = lines_;
    let mut collectedArcH = Vec::new();

    // keep parsing until we reach the end of the input
    while !lines.is_empty() {
        // break if encounters <|-endoftext-|>
        if lines[0].trim() == "<|-endoftext-|>" {
            break;
        }
        // check if the heading line is empty; if so, remove it
        if lines.len() > 0 && lines[0].trim().is_empty() {
            lines = lines[1..].to_vec();
        } else
        // check if the heading line starts with "##", if so, treat as comment and remove it
        if lines.len() > 0 && lines[0].trim().starts_with("##") {
            lines = lines[1..].to_vec();
        } else {
            let (arch, consumed) = parse_arch(&lines, 0, 0)?;
            collectedArcH.push(arch);

            lines = lines[consumed..].to_vec();
        }
    }

    Ok(collectedArcH)
}

// Recursive function to parse lines into ArcH
fn parse_arch(
    lines: &[&str],
    indent_level: usize,
    index: usize,
) -> Result<(ArcH, usize), ParseError> {
    let mut index_consumed = 0;
    if index >= lines.len() {
        return Err(ParseError::UnexpectedEndOfInput);
    }
    let line = lines[index];
    if (line.trim().starts_with("EVAL:")) {
        return Ok((
            ArcH::EvalStatement {
                expression: line[5..].trim().to_string(),
            },
            index + 1,
        ));
    }

    let line_indent = count_leading_spaces(line) / 2;
    println_!("Parsing Line {} ({}) ===> {}", index, indent_level, line);
    if line_indent != indent_level {
        return Err(ParseError::IndentationMismatch);
    }
    let trimmed_line = line.trim_start();

    if (indent_level != 0) {
        let mut itemLine = "";
        let isNewExp = trimmed_line.starts_with("- ");
        // Determine if it's a '- ' line or a regular line
        if isNewExp {
            itemLine = trimmed_line[2..].trim();
        } else {
            itemLine = trimmed_line;
        }
        println_!("OH! Found nested child!");
        // It's a child node (Single Vertex)
        let mut allLines: Vec<&str> = Vec::new();
        allLines.push(itemLine);
        let mut next_index = index + 1;
        // greedily gather lines that are children (i.e. have the indent level + 1)
        while next_index < lines.len() {
            let child_line = lines[next_index];
            let child_indent = count_leading_spaces(child_line) / 2;
            println_!("child line desu : {} ({})", next_index, child_line);
            if child_indent < indent_level {
                break; // Done with current level
            } else {
                // remove white space from the beginning of the line by indent * 2
                println_!("removed_indent: {}", child_indent);
                let removed_indent = &child_line[indent_level * 2..];
                allLines.push(removed_indent);
                next_index = next_index + 1;
            }
        }
        println_!(
            "\n\nAITE! Lines gathered: {:?}, {}\n\n",
            allLines.as_slice(),
            next_index
        );
        let (arcH, i) = parse_arch(allLines.as_slice(), 0, 0)?;
        if (isNewExp) {
            return Ok((arcH, index + i));
        } else {
            // merge with parent
            return Ok((markAsSingleChild(arcH), index + i));
        }
    } else {
        // Parse Vertex and Fish
        let (vertex_str, after_first_vertex) = split_vertex_and_fish(trimmed_line)?;
        let mut fullArcH: ArcH;

        if (after_first_vertex.is_empty()) {
            // in the case of ```\n, we take everything until \n```\n is met again
            if (vertex_str == "```") {
                // find the next occurrence of ```
                let mut i = 1;
                let mut newLine = "".to_string();
                // log lines
                println_!("lines: {:?}", lines);
                while (i < lines.len()) {
                    println_!("finding line closure... at {}", i);
                    let line = lines[i].to_string();

                    if (line.trim() == "```") {
                        break;
                    } else {
                        newLine = format!("{}\n{}", newLine, line);
                    }
                    i = i + 1;
                }
                let vertex = parse_vertex(newLine.as_str())?;
                println_!("Created Single mulit-line-str Vertex arCH: {:?}", vertex);
                fullArcH = ArcH::Single {
                    vertex,
                    is_single_child: false,
                };
                return Ok((fullArcH, index + i));
            } else {
                let vertex = parse_vertex(vertex_str)?;
                println_!("Created Single Vertex arCH: {:?}", vertex);
                fullArcH = ArcH::Single {
                    vertex,
                    is_single_child: false,
                }
            }
        } else {
            let vertex = parse_vertex(vertex_str)?;
            let (fish, after_first_fish) = parse_fish(after_first_vertex)?;
            let r = after_first_fish.as_str();
            let mut restOfLines = lines[index + 1..].to_vec();
            restOfLines.insert(0, r);
            // now we parse r + the rest of the lines
            let (arcH, s) = parse_arch(&restOfLines.as_slice(), indent_level, index)?;
            println_!("Created Multi Vertex arCH: {:?}", vertex);
            fullArcH = ArcH::ArcH {
                vertex,
                fish,
                next: Box::new(arcH),
                is_single_child: false,
            };
            index_consumed = s;
        }

        // Now, parse children if any
        let mut children = Vec::new();
        let mut next_index = index + 1 + index_consumed;
        while next_index < lines.len() {
            println_!("Parsing Children if exist at: {}", next_index);
            let child_line = lines[next_index];
            let child_indent = count_leading_spaces(child_line) / 2;
            if child_indent <= indent_level {
                break; // Done with current level
            } else if (child_indent == indent_level + 1) {
                // Parse child
                let (child_arch, consumed) = parse_arch(lines, child_indent, next_index)?;
                children.push(child_arch);
                next_index = consumed;
            } else {
                break;
            }
            // else {
            //     break;
            // }
        }
        println_!(" ");
        println_!(">>>><<<< returning ^^ >>>><<<< ");
        println_!(" ");
        if children.is_empty() {
            // println_!(">>>><<<< no children :) ");
            // println_!(" ");
            Ok((fullArcH, next_index))
        } else {
            Ok((
                ArcH::ArcHWithNewLines {
                    prefix: Box::new(fullArcH),
                    children,
                    is_single_child: false,
                },
                next_index,
            ))
        }
    }
}

// Function to parse a vertex string into a Vertex
fn parse_vertex(s: &str) -> Result<Vertex, ParseError> {
    let parts: Vec<String> = s.split("::").map(|part| part.trim().to_string()).collect();
    if parts.is_empty() {
        Err(ParseError::InvalidSyntax("Empty vertex".to_string()))
    } else {
        Ok(Vertex(parts))
    }
}

// Function to parse a fish string into a Fish
fn parse_fish(s: &str) -> Result<(Fish, String), ParseError> {
    let s = s.trim();

    if !s.starts_with("><") {
        return Err(ParseError::InvalidSyntax(
            "Fish operator should start with '><'".to_string(),
        ));
    }

    // Remove the initial '><'
    let rest = &s[2..];

    // Find the closing '>' character after '><'
    if let Some(end_pos) = rest.find('>') {
        // Extract the fish content between '><' and the next '>'
        let fish_content = &rest[..end_pos];
        // find the remaining content
        let remaining = &rest[end_pos + 1..];
        // log it
        println_!("Remaining: {}", remaining);

        Ok((
            Fish(fish_content.trim().to_string()),
            remaining.trim().to_string(),
        ))
    } else {
        // No closing '>' found after '><'
        Err(ParseError::InvalidSyntax(
            "Fish operator missing closing '>'".to_string(),
        ))
    }
}

// Function to split a line into vertex and fish parts
fn split_vertex_and_fish(s: &str) -> Result<(&str, &str), ParseError> {
    // find $(...)
    let re = Regex::new(r"\$\(.*?\)").unwrap();
    match re.find(s) {
        Some(pos) => {
            let (vertex_str, remaining_str) = s.split_at(pos.end());
            // remove the starting $( and closing )
            let vertex_str = &vertex_str[2..vertex_str.len() - 1];
            return Ok((vertex_str.trim(), remaining_str.trim()));
        }
        None => {}
    }

    match s.find("><") {
        Some(pos) => {
            let (vertex_str, remaining_str) = s.split_at(pos);
            Ok((vertex_str.trim(), remaining_str.trim()))
        }
        None => Ok((s.trim(), "")),
    }
}

// Function to count leading spaces (each indent level is 2 spaces)
fn count_leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

// Example usage
fn main() -> Result<(), Box<dyn Error>> {
    // // example inputs
    // let input = r#"UI::App ><renders>
    // - UI::List
    // - UI::AddNewTask_Button"#;
    // let input = r#"UI::App ><renders> a ><go> b"#;
    // ----------------

    // get first arg as path ::
    let path = std::env::args().nth(1);

    // read from file
    let mut fileContent = std::fs::read_to_string(path.unwrap());

    let input = match fileContent {
        // (10/FEB) hot fix only:
        // to-do: fix a bug that causes error if last line is not ##
        Ok(fileContent) => fileContent + "\n\n\n##",
        Err(e) => {
            println_!("Error reading file: {}", e);
            return Ok(());
        }
    };

    // let input = r#"UI::App"#;

    let oringalForms = collect_inputs(parse_inputs(input.as_str()));
    println_!("{:#?}", oringalForms);

    // concat
    let mut allOriginalForms = Vec::new();
    for of in oringalForms {
        allOriginalForms.extend(of.clone());
        for o in of {
            println_!("\n\n{}\n\n", o);
        }
    }

    // write result as JONS to file
    let json = serde_json::to_string_pretty(&allOriginalForms).unwrap();
    std::fs::write("output.json", json).unwrap();

    Ok(())
}
