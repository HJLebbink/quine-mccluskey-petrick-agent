use clap::{Arg, Command, ArgMatches};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use regex::Regex;
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize)]
struct QMRequest {
    minterms: Vec<u32>,
    dont_cares: Option<Vec<u32>>,
    variables: usize,
    variable_names: Option<Vec<String>>, // A, B, C, etc.
    format: Option<String>,
}

#[derive(Debug, Serialize)]
struct QMResponse {
    original_minterms: Vec<u32>,
    dont_cares: Vec<u32>,
    minimized_sop: String,
    minimized_pos: Option<String>,
    prime_implicants: Vec<String>,
    essential_prime_implicants: Vec<String>,
    cost_reduction: Option<f64>,
    truth_table: Option<String>,
    steps: Option<Vec<String>>, // For educational purposes
}

fn main() {
    let matches = Command::new("qm-agent")
        .version("1.0.0")
        .author("Henk-Jan Lebbink")
        .about("Quine-McCluskey Boolean minimization agent for Claude")
        .subcommand(
            Command::new("minimize")
                .about("Minimize a Boolean function")
                .arg(Arg::new("input")
                    .short('i')
                    .long("input")
                    .help("Input: JSON file path, inline JSON, or natural language")
                    .required(true))
                .arg(Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("Output format")
                    .value_parser(["json", "human", "table", "steps"])
                    .default_value("human"))
                .arg(Arg::new("show-steps")
                    .long("show-steps")
                    .help("Show step-by-step solution")
                    .action(clap::ArgAction::SetTrue))
                .arg(Arg::new("include-pos")
                    .long("include-pos")
                    .help("Include Product of Sums form")
                    .action(clap::ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("interactive")
                .about("Interactive mode for complex queries")
        )
        .subcommand(
            Command::new("examples")
                .about("Show usage examples")
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("minimize", sub_matches)) => handle_minimize(sub_matches),
        Some(("interactive", _)) => handle_interactive(),
        Some(("examples", _)) => handle_examples(),
        _ => {
            eprintln!("Use --help for usage information");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_minimize(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<String>("input")
        .expect("input is required by clap");
    let format = matches.get_one::<String>("format")
        .expect("format has default value in clap");
    let show_steps = matches.get_flag("show-steps");
    let include_pos = matches.get_flag("include-pos");

    // Parse input in various formats
    let request = parse_input(input)?;

    // Run Quine-McCluskey algorithm
    let result = run_quine_mccluskey(&request, show_steps, include_pos)?;

    // Output in requested format
    match format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&result)?),
        "human" => print_human_readable(&result),
        "table" => print_table_format(&result),
        "steps" => print_steps(&result),
        _ => return Err(anyhow!("Unknown format: {}", format)),
    }

    Ok(())
}

fn parse_input(input: &str) -> Result<QMRequest> {
    // Try parsing as file path first
    if let Ok(file_content) = fs::read_to_string(input) {
        if let Ok(request) = serde_json::from_str::<QMRequest>(&file_content) {
            return Ok(request);
        }
    }

    // Try parsing as inline JSON
    if let Ok(request) = serde_json::from_str::<QMRequest>(input) {
        return Ok(request);
    }

    // Parse natural language formats
    parse_natural_input(input)
}

fn parse_natural_input(input: &str) -> Result<QMRequest> {
    let input = input.trim();

    // Pattern 1: f(A,B,C) = Σ(1,3,7) + d(2,4)
    let sigma_pattern = Regex::new(r"f\(([A-Z,\s]+)\)\s*=\s*Σ\(([0-9,\s]+)\)(?:\s*\+\s*d\(([0-9,\s]*)\))?")?;
    if let Some(caps) = sigma_pattern.captures(input) {
        let variables: Vec<String> = caps[1].split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let minterms: Vec<u32> = caps[2].split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to parse minterm: {}", e))?;
        let dont_cares: Option<Vec<u32>> = caps.get(3)
            .map(|m| -> Result<Vec<u32>> {
                m.as_str().split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().parse())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| anyhow!("Failed to parse don't care term: {}", e))
            })
            .transpose()?;

        return Ok(QMRequest {
            minterms,
            dont_cares,
            variables: variables.len(),
            variable_names: Some(variables),
            format: None,
        });
    }

    // Pattern 2: "minimize minterms 1,3,7 with 3 variables"
    let simple_pattern = Regex::new(r"minimize\s+minterms?\s+([0-9,\s]+)\s+with\s+(\d+)\s+variables?")?;
    if let Some(caps) = simple_pattern.captures(input) {
        let minterms: Vec<u32> = caps[1].split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to parse minterm: {}", e))?;
        let variables: usize = caps[2].parse()?;

        return Ok(QMRequest {
            minterms,
            dont_cares: None,
            variables,
            variable_names: None,
            format: None,
        });
    }

    // Pattern 3: Truth table format "truth table: 00110110"
    let tt_pattern = Regex::new(r"truth\s+table:\s*([01]+)")?;
    if let Some(caps) = tt_pattern.captures(input) {
        let truth_table = &caps[1];
        let variables = (truth_table.len() as f64).log2() as usize;
        let minterms: Vec<u32> = truth_table.chars()
            .enumerate()
            .filter_map(|(i, c)| if c == '1' { Some(i as u32) } else { None })
            .collect();

        return Ok(QMRequest {
            minterms,
            dont_cares: None,
            variables,
            variable_names: None,
            format: None,
        });
    }

    Err(anyhow!("Could not parse input format. Supported formats:\n\
        - JSON: {{\"minterms\": [1,3,7], \"variables\": 3}}\n\
        - Function notation: f(A,B,C) = Σ(1,3,7)\n\
        - With don't cares: f(A,B,C) = Σ(1,3,7) + d(2,4)\n\
        - Simple: minimize minterms 1,3,7 with 3 variables\n\
        - Truth table: truth table: 00110110"))
}

fn run_quine_mccluskey(request: &QMRequest, show_steps: bool, include_pos: bool) -> Result<QMResponse> {
    let empty_dont_cares = vec![];
    let dont_cares = request.dont_cares.as_ref().unwrap_or(&empty_dont_cares);
    let variable_names = request.variable_names.as_ref()
        .cloned()
        .unwrap_or_else(|| {
            (0..request.variables)
                .map(|i| ((b'A' + i as u8) as char).to_string())
                .collect()
        });

    // Use the actual QM implementation
    let (minimized_sop, prime_implicants_formatted, essential_pis_formatted, steps) =
        integrate_your_qm_solver(&request.minterms, dont_cares, request.variables, &variable_names, show_steps);

    let minimized_pos = if include_pos {
        Some(convert_to_pos(&minimized_sop))
    } else {
        None
    };

    Ok(QMResponse {
        original_minterms: request.minterms.clone(),
        dont_cares: dont_cares.clone(),
        minimized_sop,
        minimized_pos,
        prime_implicants: prime_implicants_formatted,
        essential_prime_implicants: essential_pis_formatted,
        cost_reduction: Some(calculate_cost_reduction(&request.minterms, request.variables)),
        truth_table: Some(generate_truth_table(&request.minterms, dont_cares, request.variables)),
        steps,
    })
}

fn integrate_your_qm_solver(
    minterms: &[u32],
    dont_cares: &[u32],
    variables: usize,
    _variable_names: &[String],
    show_steps: bool
) -> (String, Vec<String>, Vec<String>, Option<Vec<String>>) {
    use qm_agent::QMSolver;

    let mut solver = QMSolver::new(variables);
    solver.set_minterms(minterms);
    solver.set_dont_cares(dont_cares);

    let result = solver.solve();

    let steps = if show_steps {
        Some(result.solution_steps)
    } else {
        None
    };

    (
        result.minimized_expression,
        result.prime_implicants,
        result.essential_prime_implicants,
        steps
    )
}

fn convert_to_pos(sop_expression: &str) -> String {
    // Placeholder - implement De Morgan's laws conversion if needed
    format!("({})", sop_expression.replace(" + ", ")("))
}

fn calculate_cost_reduction(minterms: &[u32], variables: usize) -> f64 {
    // Simple cost calculation - replace with your actual cost analysis
    let original_cost = minterms.len() * variables;
    let minimized_cost = (minterms.len() as f64 * 0.6) as usize; // Placeholder

    if original_cost > 0 {
        ((original_cost - minimized_cost) as f64 / original_cost as f64) * 100.0
    } else {
        0.0
    }
}

fn generate_truth_table(minterms: &[u32], dont_cares: &[u32], variables: usize) -> String {
    let total_rows = 1 << variables;
    let minterm_set: HashSet<u32> = minterms.iter().copied().collect();
    let dont_care_set: HashSet<u32> = dont_cares.iter().copied().collect();

    let mut table = String::new();

    // Header
    for i in 0..variables {
        table.push_str(&format!("{} ", ((b'A' + i as u8) as char)));
    }
    table.push_str("| F\n");
    table.push_str(&"-".repeat(variables * 2 + 4));
    table.push('\n');

    // Rows
    for i in 0..total_rows {
        for j in (0..variables).rev() {
            table.push_str(&format!("{} ", (i >> j) & 1));
        }
        table.push_str("| ");

        if minterm_set.contains(&i) {
            table.push('1');
        } else if dont_care_set.contains(&i) {
            table.push('X');
        } else {
            table.push('0');
        }
        table.push('\n');
    }

    table
}

fn print_human_readable(result: &QMResponse) {
    println!("🔍 Quine-McCluskey Boolean Minimization Result");
    println!("════════════════════════════════════════════");

    println!("\n📊 Input:");
    println!("   Minterms: {:?}", result.original_minterms);
    if !result.dont_cares.is_empty() {
        println!("   Don't cares: {:?}", result.dont_cares);
    }

    println!("\n✨ Minimized Expression (SOP):");
    println!("   F = {}", result.minimized_sop);

    if let Some(ref pos) = result.minimized_pos {
        println!("\n✨ Minimized Expression (POS):");
        println!("   F = {}", pos);
    }

    println!("\n🎯 Prime Implicants:");
    for pi in &result.prime_implicants {
        println!("   • {}", pi);
    }

    println!("\n⭐ Essential Prime Implicants:");
    for epi in &result.essential_prime_implicants {
        println!("   • {}", epi);
    }

    if let Some(cost) = result.cost_reduction {
        println!("\n💰 Cost Reduction: {:.1}%", cost);
    }

    if let Some(ref steps) = result.steps {
        println!("\n📝 Solution Steps:");
        for (i, step) in steps.iter().enumerate() {
            println!("   {}. {}", i + 1, step);
        }
    }
}

fn print_table_format(result: &QMResponse) {
    if let Some(ref truth_table) = result.truth_table {
        println!("Truth Table:");
        println!("{}", truth_table);
    }

    println!("\nMinimized Expression: {}", result.minimized_sop);
}

fn print_steps(result: &QMResponse) {
    if let Some(ref steps) = result.steps {
        println!("Quine-McCluskey Solution Steps:");
        println!("===============================");
        for (i, step) in steps.iter().enumerate() {
            println!("{}. {}", i + 1, step);
        }
    } else {
        println!("No step-by-step information available. Use --show-steps flag.");
    }
}

fn handle_interactive() -> Result<()> {
    println!("🚀 QM Agent Interactive Mode");
    println!("============================");
    println!("Enter Boolean functions in various formats:");
    println!("• JSON: {{\"minterms\": [1,3,7], \"variables\": 3}}");
    println!("• Function: f(A,B,C) = Σ(1,3,7)");
    println!("• With don't cares: f(A,B,C) = Σ(1,3,7) + d(2,4)");
    println!("• Simple: minimize minterms 1,3,7 with 3 variables");
    println!("• Truth table: truth table: 00110110");
    println!("• Type 'help' for more options, 'quit' to exit\n");

    loop {
        print!("qm> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "quit" | "exit" => break,
            "help" => print_interactive_help(),
            "examples" => print_examples(),
            "" => continue,
            _ => {
                match parse_input(input) {
                    Ok(request) => {
                        match run_quine_mccluskey(&request, false, false) {
                            Ok(result) => {
                                println!();
                                print_human_readable(&result);
                                println!();
                            }
                            Err(e) => eprintln!("❌ Error processing: {}", e),
                        }
                    }
                    Err(e) => eprintln!("❌ Parse error: {}", e),
                }
            }
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}

fn print_interactive_help() {
    println!("\n📚 Interactive Mode Commands:");
    println!("• help - Show this help");
    println!("• examples - Show usage examples");
    println!("• quit/exit - Exit interactive mode");
    println!("• Any valid input format to minimize\n");
}

fn handle_examples() -> Result<()> {
    print_examples();
    Ok(())
}

fn print_examples() {
    println!("\n📚 Usage Examples:");
    println!("==================");

    println!("\n1. Function notation:");
    println!("   qm-agent minimize -i 'f(A,B,C) = Σ(1,3,7)'");

    println!("\n2. With don't cares:");
    println!("   qm-agent minimize -i 'f(A,B,C) = Σ(1,3,7) + d(2,4)'");

    println!("\n3. Simple format:");
    println!("   qm-agent minimize -i 'minimize minterms 1,3,7 with 3 variables'");

    println!("\n4. JSON format:");
    println!("   qm-agent minimize -i '{{\"minterms\": [1,3,7], \"variables\": 3}}'");

    println!("\n5. Truth table:");
    println!("   qm-agent minimize -i 'truth table: 00110110'");

    println!("\n6. From file:");
    println!("   qm-agent minimize -i input.json");

    println!("\n7. Show steps:");
    println!("   qm-agent minimize -i 'f(A,B) = Σ(1,3)' --show-steps");

    println!("\n8. Interactive mode:");
    println!("   qm-agent interactive");
}