use crate::fdg::FdgExporter;
use crate::graphviz::{DisplayMode, GraphVizExporter, OutputFormat};
use crate::json::JsonExporter;
use crate::parser::parse_line;
use crate::{graphviz, svg, Command, Interaction};
use anyhow::{anyhow, Context, Result};
use microdot_core::graph::Graph;
use microdot_core::{CommandResult, Line};
use rustyline::error::ReadlineError;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LockResult, RwLock};

pub fn repl<I: Interaction>(
    interaction: &mut I,
    json_file: &Path,
    graph: Arc<RwLock<Graph>>,
) -> Result<()> {
    loop {
        let readline = interaction.read(">> ");

        // when we start, make sure the existing pic is up to date.
        compile_graph(interaction, json_file, &graph)?;

        let dirty = match readline {
            Ok(line) => {
                interaction.add_history(line.as_str());

                let line = Line::new(line);

                let command = parse_line(line);

                match command {
                    Command::GraphCommand(graph_command) => {
                        let mut graph = graph.write().unwrap();
                        let applied = graph.apply_command(graph_command);
                        interaction.log(format!("({})", applied));
                        true
                    }
                    Command::ShowHelp => {
                        interaction.log(include_str!("help.txt"));
                        false
                    }
                    Command::RenameNodeUnlabelled { .. } => {
                        // no need to act, this is for auto-complete
                        false
                    }
                    Command::Show => {
                        let svg_file = json_file.with_extension("svg");
                        let svg_file = std::fs::canonicalize(svg_file)
                            .expect("could not canconcicalise file path");
                        let result = svg::open_in_gapplin(&svg_file);
                        interaction.log(result.to_string());
                        false
                    }
                    Command::PrintDot => {
                        let graph = graph.read().unwrap();
                        let mut exporter = GraphVizExporter::new(DisplayMode::Interactive);
                        let out = exporter.export_dot(&graph);
                        interaction.log(out);
                        interaction.log("Dot printed");
                        false
                    }
                    Command::PrintJson => {
                        let graph = graph.read().unwrap();
                        let mut exporter = JsonExporter::new();
                        let out = exporter.export_json(&graph);
                        interaction.log(out);
                        interaction.log("Json printed");
                        false
                    }
                    Command::Search { sub_label } => {
                        let mut graph = graph.write().unwrap();
                        interaction.log(format!("({})", graph.highlight_search_results(sub_label)));
                        true
                    }
                    Command::Save => {
                        interaction.log(format!("saving to {}", json_file.to_string_lossy()));
                        true
                    }
                    Command::ParseError { .. } => {
                        interaction.log("could not understand command; try 'h' for help");
                        false
                    }
                    Command::Exit => return Ok(()),
                }
            }
            Err(ReadlineError::Interrupted) => {
                interaction.log("CTRL-C");

                return Ok(());
            }
            Err(ReadlineError::Eof) => {
                interaction.log("CTRL-D");

                return Ok(());
            }
            Err(err) => {
                interaction.log(format!("Error: {:?}", err));
                return Err(anyhow::anyhow!("readline error: {}", err.to_string()));
            }
        };

        if dirty {
            compile_graph(interaction, json_file, &graph)?;
        }
    }
}

enum RenderMethod {
    GraphViz,
    Fdg,
}

const RENDER_METHOD: RenderMethod = RenderMethod::GraphViz;

fn compile_graph<I: Interaction>(
    interaction: &mut I,
    json_file: &Path,
    graph: &Arc<RwLock<Graph>>,
) -> Result<()> {
    let graph = match graph.write() {
        Ok(graph) => graph,
        Err(e) => return Err(anyhow!(e.to_string())),
    };
    match RENDER_METHOD {
        RenderMethod::GraphViz => {
            let interactive_dot_file = save_dot_file(json_file, &graph)?;
            if interaction.should_compile() {
                compile_dot(interactive_dot_file);
            }
        }
        RenderMethod::Fdg => {
            if interaction.should_compile() {
                compile_fdg(json_file, &graph)?;
            }
        }
    }

    Ok(())
}

fn compile_fdg(json_file: &Path, graph: &Graph) -> Result<PathBuf> {
    let mut fdg_exporter = FdgExporter::default();
    let svg = fdg_exporter.export(graph);
    let svg_file = json_file.with_extension("svg");
    std::fs::write(&svg_file, svg)?;

    Ok(svg_file)
}

fn save_dot_file(json_file: &Path, graph: &Graph) -> Result<PathBuf> {
    let mut json_exporter = JsonExporter::new();
    let json = json_exporter.export_json(graph);
    std::fs::write(json_file, json)?;

    let mut dot_exporter = GraphVizExporter::new(DisplayMode::Interactive);
    let interactive_dot = dot_exporter.export_dot(graph);
    let interactive_dot_file = json_file.with_extension("dot");
    std::fs::write(&interactive_dot_file, interactive_dot)?;

    Ok(interactive_dot_file)
}

fn compile_dot(interactive_dot_file: PathBuf) -> CommandResult {
    let svg_compile = graphviz::compile(
        &interactive_dot_file,
        DisplayMode::Interactive,
        OutputFormat::Svg,
    );

    let png_compile = graphviz::compile(
        &interactive_dot_file,
        DisplayMode::Interactive,
        OutputFormat::Png,
    );

    let msg = match (svg_compile, png_compile) {
        (Ok(_), Ok(_)) => format!(
            "compiled interactive dot: {}",
            interactive_dot_file.to_string_lossy()
        ),
        (Err(e), _) => format!("failed to compile interactive dot to svg: {}", e),
        (_, Err(e)) => format!("failed to compile interactive dot to png: {}", e),
    };

    CommandResult::new(msg)
}
