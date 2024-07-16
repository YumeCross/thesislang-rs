use std::process::exit;

use ariadne::{Fmt, Label, Report, ReportKind, Source};

use crate::seq;
use crate::parser::{SourcePos, SrcInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidSyntax,
    FreeIdentifier,
    TypeMismatch
}

impl ErrorKind {
    pub fn to_error_code(&self) -> &str {
        match &self {
            Self::InvalidSyntax => "E01",
            Self::FreeIdentifier => "E02",
            Self::TypeMismatch => "E03"
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    span: std::ops::Range<usize>,
    labels: Vec<Label<(String, std::ops::Range<usize>)>>
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, message: "".to_string(), span: 0..0, labels: vec![] }
    }

    pub fn kind(&self) -> ErrorKind { self.kind }

    pub fn message(&self) -> &String { &self.message }

    pub fn with_label(mut self, label: Label<(String, std::ops::Range<usize>)>) -> Self {
        seq!(self.labels.push(label), self)
    }

    pub fn with_message(mut self, content: String) -> Self {
        seq!(self.message = content, self)
    }

    pub fn with_span(mut self, span: std::ops::Range<usize>) -> Self {
        seq!(self.span = span, self)
    }

    pub fn report_error(self, src: &SrcInfo, pos: SourcePos, label: String) -> ! {
        // let kind = format!("{:?}", self.kind);
        // To make it appear like rust-style error.
        print!("{}", "error".fg(ariadne::Color::Red));

        let mut builder = 
        Report::build(ReportKind::Custom("\x08", ariadne::Color::Red), &src.id, pos.i())
            .with_code(self.kind.to_error_code())
            .with_message(self.message())
            .with_label(
                Label::new((src.id.clone(), self.span.clone()))
                    .with_message(label)
                    .with_color(ariadne::Color::Red)
            );

        for label in self.labels {
            builder = builder.with_label(label);
        }

        builder
            .finish()
            .print((src.id.clone(), Source::from(&src.text)))
            .unwrap();
        
        exit(1)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl core::error::Error for Error {}

impl<S: Into<String>> From<(ErrorKind, S)> for Error {
    fn from(value: (ErrorKind, S)) -> Self {
        Self { kind: value.0, message: value.1.into(), span: 0..0, labels: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn error_to_string() {
        use super::ErrorKind::*;
        assert_eq!(Error::new(InvalidSyntax).to_string(), "InvalidSyntax: ");
    }
}
