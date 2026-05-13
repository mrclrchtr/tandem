use std::io::IsTerminal;
use std::sync::LazyLock;

use serde::Serialize;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{LinesWithEndings, as_24_bit_terminal_escaped},
};
use tandem_core::ticket::{Ticket, TicketId, TicketStatus};

use super::util::format_timestamp;

#[derive(Serialize)]
pub(crate) struct TicketJsonEntry<'a> {
    #[serde(flatten)]
    pub(crate) meta: &'a tandem_core::ticket::TicketMeta,
    #[serde(flatten)]
    pub(crate) state: &'a tandem_core::ticket::TicketState,
    pub(crate) content_path: String,
}

#[derive(Serialize)]
pub(crate) struct TicketJson<'a> {
    pub(crate) schema_version: u64,
    #[serde(flatten)]
    pub(crate) ticket: TicketJsonEntry<'a>,
}

#[derive(Serialize)]
pub(crate) struct TicketListJson<'a> {
    pub(crate) schema_version: u64,
    pub(crate) tickets: Vec<TicketJsonEntry<'a>>,
}

/// Lazy-loaded syntax definitions for code highlighting (~35 MB).
pub(crate) static SYNTAX_SET: LazyLock<SyntaxSet> =
    LazyLock::new(SyntaxSet::load_defaults_newlines);
pub(crate) static THEME: LazyLock<syntect::highlighting::Theme> = LazyLock::new(|| {
    let ts = ThemeSet::load_defaults();
    ts.themes["base16-ocean.dark"].clone()
});

pub(crate) fn print_ticket_human(ticket: &Ticket) {
    let use_color = std::io::stdout().is_terminal();
    let (b, r, g, y, bl, n) = if use_color {
        (
            "\x1b[1m", "\x1b[31m", "\x1b[32m", "\x1b[33m", "\x1b[34m", "\x1b[0m",
        )
    } else {
        ("", "", "", "", "", "")
    };

    let status_color = match ticket.state.status {
        TicketStatus::Done => g,
        TicketStatus::InProgress => bl,
        TicketStatus::Blocked => r,
        TicketStatus::Todo => y,
    };

    let sep = format!("  {}", "─".repeat(46));

    // Header
    println!("  {b}{}{n} · {}", ticket.meta.id, ticket.meta.title);
    println!("{sep}");
    println!();

    // Metadata
    println!(
        "  {b}Status     {n} · {sc}{}{n}",
        ticket.state.status.as_str(),
        sc = status_color
    );
    println!("  {b}Priority   {n} · {}", ticket.meta.priority);
    println!("  {b}Type       {n} · {}", ticket.meta.ticket_type);

    if let Some(effort) = ticket.meta.effort {
        println!("  {b}Effort     {n} · {effort}");
    }

    if !ticket.meta.tags.is_empty() {
        println!("  {b}Tags       {n} · {}", ticket.meta.tags.join(", "));
    }

    if !ticket.meta.depends_on.is_empty() {
        let deps: Vec<&str> = ticket
            .meta
            .depends_on
            .iter()
            .map(TicketId::as_str)
            .collect();
        println!("  {b}Depends on {n} · {}", deps.join(", "));
    }

    println!();
    let ts = format_timestamp(&ticket.state.updated_at);
    println!("  {b}Updated    {n} · {ts} (rev {})", ticket.state.revision);

    // Content section
    println!();
    println!("{sep}");
    println!("  {b}Content{n}");
    println!("{sep}");

    let (tw, _) = termimad::terminal_size();
    let cw = if tw > 4 { (tw - 2) as usize } else { 78 };

    if !use_color {
        // Non-TTY: preserve raw content for scripting and tests
        for line in ticket.content.lines() {
            println!("  {line}");
        }
    } else if !ticket.content.contains("```") {
        // TTY, no fenced code blocks: render everything with termimad
        let mut skin = termimad::MadSkin::default();
        skin.inline_code.object_style.background_color = None;
        skin.code_block.compound_style.object_style.background_color = None;
        let rendered = skin.text(&ticket.content, Some(cw)).to_string();
        for line in rendered.lines() {
            println!("  {line}");
        }
    } else {
        // TTY with fenced code blocks: syntax-highlight code, termimad for the rest
        let mut skin = termimad::MadSkin::default();
        skin.inline_code.object_style.background_color = None;
        skin.code_block.compound_style.object_style.background_color = None;

        let parts: Vec<&str> = ticket.content.split("```").collect();
        for (i, part) in parts.iter().enumerate() {
            if i % 2 == 0 {
                // Non-code segment — render with termimad
                if part.is_empty() {
                    continue;
                }
                let rendered = skin.text(part, Some(cw)).to_string();
                for line in rendered.lines() {
                    println!("  {line}");
                }
            } else if let Some(newline) = part.find('\n') {
                let lang = part[..newline].trim();
                let code = &part[newline + 1..];
                let code = code.strip_suffix('\n').unwrap_or(code);

                if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(lang) {
                    // Syntax-highlight this code block
                    let mut h = HighlightLines::new(syntax, &THEME);
                    for line in LinesWithEndings::from(code) {
                        if let Ok(ranges) = h.highlight_line(line, &SYNTAX_SET) {
                            print!("  {}", as_24_bit_terminal_escaped(&ranges, false));
                        } else {
                            print!("  {line}");
                        }
                    }
                } else {
                    // Unrecognized language — let termimad style the block
                    let block = format!("```{lang}\n{code}```");
                    let rendered = skin.text(&block, Some(cw)).to_string();
                    for line in rendered.lines() {
                        println!("  {line}");
                    }
                }
            }
        }
    }
}
