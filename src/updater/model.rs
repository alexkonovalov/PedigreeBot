use strum_macros::EnumString;
use strum_macros::Display;
use std::fmt::{self, Display};
use petgraph::{graph::{NodeIndex}};

#[derive(EnumString, Display, Debug)]
pub enum ButtonCommand {
    No,
}


pub struct Person {
    pub name: String,
    pub completeness: NodeCompleteness,
}

impl Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Person {
    pub fn new(name: String, completeness: NodeCompleteness) -> Self { Self { name, completeness } }
}

pub struct DescribedNodeInfo {
    pub ix: Option<NodeIndex<u32>>,
}

impl DescribedNodeInfo {
    pub fn new(ix: Option::<NodeIndex<u32>>) -> Self { Self { ix } }
}

#[derive(PartialEq, Debug)]
pub enum NodeCompleteness {
    Plain,
    OneParent,
    ParentsComplete,
    SiblingsComplete,
    ChildrenComplete
}

pub const NEW_NODE_STATUS: NodeCompleteness = NodeCompleteness::Plain;

#[derive(Debug, PartialEq)]
pub enum OutputAction {
    AskFirstParent(String),
    AskSecondParent(String),
    AskIfSiblings(String),
    AskIfMoreSiblings(String),
    AskIfChildren(String),
    AskIfMoreChildren(String),
    NotifyError,
    NotifyComplete
}

#[derive(Debug)]
pub enum InputCommand<'a> {
    Text(&'a str),
    No
}

#[derive(Debug)]
pub enum OutputCommand {
    Prompt(String),
    PromptButtons(Vec<(ButtonCommand, String)>, String)
}