use microdot_core::command::GraphCommand;
use microdot_core::{Id, Label, Line};
use rustyline::{Editor, Helper};

pub mod colors;
mod fdg;
pub mod graphviz;
pub mod helper;
pub mod json;
pub mod palettes;
pub mod parser;
pub mod repl;
mod storage;
pub mod svg;
pub mod util;

#[derive(PartialEq, Eq, Debug)]
pub enum Command {
    GraphCommand(GraphCommand),
    ShowHelp,
    Search { sub_label: Label },
    PrintDot,
    PrintJson,
    RenameNodeUnlabelled { id: Id },
    Save,
    Show,
    Exit,
    ParseError { line: Line },
}

impl Command {
    #[allow(dead_code)]
    fn to_help_string(&self) -> String {
        match self {
            Command::GraphCommand(c) => c.to_help_string(),
            Command::ShowHelp => "display this help".into(),
            Command::Search { sub_label } => {
                format!("search for <{}> and highlight matching nodes", sub_label)
            }
            Command::PrintDot => "print the dot definition for this graph to the terminal".into(),
            Command::PrintJson => "print the json definition for this graph to the terminal".into(),
            Command::RenameNodeUnlabelled { id } => {
                format!("rename <{}> but no new label text supplied", id)
            }
            Command::Save => "save the graph to disc".into(),
            Command::Show => "open the diagram in Gapplin".into(),
            Command::Exit => "exit microdot".into(),
            Command::ParseError { line } => format!("could not parse: \"{}\"", line),
        }
    }
}

impl From<GraphCommand> for Command {
    fn from(c: GraphCommand) -> Self {
        Command::GraphCommand(c)
    }
}

// a trait which deals with the R & P of REPL: Read and Print; can be mixed in with a loop
pub trait Interaction {
    // TODO: this makes more sense as something like a stream of input lines, with an EOF from
    // readline => None.
    fn read(&mut self, prompt: &str) -> rustyline::Result<String>;
    // TODO: should this be converted to a futures::sink::Sink? 'write to the history channel'
    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool;
    // TODO: possibly another futures::sink::Sink
    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S);
    // TODO: bad design. Should be handled outside; really corresponds to 'did the last command
    // dirty the cache'
    fn should_compile(&self) -> bool;
}

impl<H> Interaction for Editor<H>
where
    H: Helper,
{
    fn read(&mut self, prompt: &str) -> rustyline::Result<String> {
        self.readline(prompt)
    }

    fn add_history<S: AsRef<str> + Into<String>>(&mut self, history: S) -> bool {
        self.add_history_entry(history)
    }

    fn log<S: AsRef<str> + Into<String>>(&mut self, message: S) {
        println!("{}", message.into());
    }

    fn should_compile(&self) -> bool {
        true
    }
}
