use std::io::{IsTerminal, Read};
use std::{fs, path::PathBuf};

use rand::RngExt;
use tandem_core::ports::TicketStore;
use tandem_core::ticket::TicketId;
use tandem_storage::{FileTicketStore, TandemConfig};

pub(crate) const CROCKFORD_BASE32: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub(crate) const DEFAULT_CONTENT_TEMPLATE: &str = concat!(
    "## Context\n\n",
    "What problem are we solving? What area of the repo or behavior is affected?\n\n",
    "## Goal\n\n",
    "What outcome should exist when this ticket is done?\n\n",
    "## Open Questions\n\n",
    "- [ ] Question or ambiguity 1\n",
    "- [ ] Question or ambiguity 2\n\n",
    "## Acceptance\n\n",
    "- [ ] Observable outcome 1\n",
    "- [ ] Observable outcome 2\n\n",
    "## Ready When\n\n",
    "- [ ] Scope is clear\n",
    "- [ ] Dependencies are known\n",
    "- [ ] Open questions are resolved or explicitly deferred\n",
    "- [ ] Acceptance is specific enough for implementation\n"
);

pub(crate) const DEFINITION_TAG_READY: &str = "definition:ready";
pub(crate) const DEFINITION_TAG_QUESTIONS: &str = "definition:questions";

pub(crate) fn ticket_content_path(id: &TicketId) -> String {
    format!(".tndm/tickets/{}/content.md", id)
}

/// Strip fractional seconds from an RFC 3339 timestamp and replace T with a space.
pub(crate) fn format_timestamp(raw: &str) -> String {
    if let Some(dot) = raw.find('.') {
        let rest = &raw[dot..];
        let tz_end = rest.find(['Z', '+', '-']).unwrap_or(rest.len());
        format!("{}{}", &raw[..dot], &rest[tz_end..]).replace('T', " ")
    } else {
        raw.replace('T', " ")
    }
}

pub(crate) fn generate_ticket_id(
    store: &FileTicketStore,
    prefix: &str,
) -> anyhow::Result<TicketId> {
    let mut rng = rand::rng();

    loop {
        let suffix: String = (0..6)
            .map(|_| {
                let index = rng.random_range(0..CROCKFORD_BASE32.len());
                CROCKFORD_BASE32[index] as char
            })
            .collect();

        let candidate = TicketId::parse(format!("{prefix}-{suffix}"))?;
        let exists = store
            .ticket_exists(&candidate)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if !exists {
            return Ok(candidate);
        }
    }
}

pub(crate) fn read_stdin_if_no_flags(no_explicit: bool) -> anyhow::Result<Option<String>> {
    if no_explicit && !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(buf))
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn load_ticket_content(
    content_file: Option<PathBuf>,
    content: Option<String>,
    stdin_content: Option<String>,
    config: &TandemConfig,
) -> anyhow::Result<String> {
    if let Some(path) = content_file {
        return fs::read_to_string(path).map_err(|error| anyhow::anyhow!("{error}"));
    }

    if let Some(value) = content {
        return Ok(value);
    }

    if let Some(value) = stdin_content {
        return Ok(value);
    }

    if !config.content_template.is_empty() {
        return Ok(config.content_template.clone());
    }

    Ok(DEFAULT_CONTENT_TEMPLATE.to_string())
}
