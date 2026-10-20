// Text flows: ordered chains of frames that long text runs through, and the story they carry
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::frame::Run;
use crate::ids::{FlowId, FrameId, StyleId};
use crate::runs::text_of;

/// One paragraph of a flow's story: its words as runs, in a paragraph style of its own when it has
/// one and in the style of the flow's first frame when it has not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase")]
pub struct StoryPara {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleId>,
    pub inline: Vec<Run>,
}

impl StoryPara {
    /// A paragraph of plain words in the frame's style.
    pub fn plain(text: &str) -> Self {
        Self {
            style: None,
            inline: vec![Run {
                text: text.to_owned(),
                style: None,
                bold: false,
                italic: false,
                font: None,
                colour: None,
            }],
        }
    }

    /// The paragraph's words as one string.
    pub fn text(&self) -> String {
        text_of(&self.inline)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../packages/protocol/src/generated/")]
#[serde(rename_all = "camelCase", from = "FlowRepr")]
pub struct Flow {
    pub id: FlowId,
    /// The story, paragraph by paragraph. Paragraphs are named by their place in the list.
    pub paragraphs: Vec<StoryPara>,
    pub frames: Vec<FrameId>,
    #[serde(default = "yes")]
    pub auto_add_pages: bool,
}

impl Flow {
    /// A flow of plain paragraphs, one for each line of `text`.
    pub fn from_text(id: FlowId, text: &str, frames: Vec<FrameId>) -> Self {
        Self {
            id,
            paragraphs: text.split('\n').map(StoryPara::plain).collect(),
            frames,
            auto_add_pages: true,
        }
    }

    /// The story as plain text, a line for each paragraph.
    pub fn text(&self) -> String {
        self.paragraphs
            .iter()
            .map(StoryPara::text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// What a flow is read from: the paragraphs, or the `source` text older books kept instead.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlowRepr {
    id: FlowId,
    #[serde(default)]
    paragraphs: Vec<StoryPara>,
    #[serde(default)]
    source: Option<String>,
    frames: Vec<FrameId>,
    #[serde(default = "yes")]
    auto_add_pages: bool,
}

impl From<FlowRepr> for Flow {
    fn from(repr: FlowRepr) -> Self {
        let paragraphs = match repr.source {
            // A path to a text file in the package names no words of its own.
            Some(source) if repr.paragraphs.is_empty() && !is_package_path(&source) => {
                source.split('\n').map(StoryPara::plain).collect()
            }
            _ => repr.paragraphs,
        };
        Flow {
            id: repr.id,
            paragraphs,
            frames: repr.frames,
            auto_add_pages: repr.auto_add_pages,
        }
    }
}

fn is_package_path(text: &str) -> bool {
    text.starts_with("text/") && text.ends_with(".md") && !text.contains('\n')
}

fn yes() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(json: serde_json::Value) -> Flow {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn an_older_flow_with_source_text_becomes_a_paragraph_for_each_line() {
        let flow = read(serde_json::json!({
            "id": "f", "source": "One\nTwo", "frames": ["a"]
        }));
        assert_eq!(flow.text(), "One\nTwo");
        assert_eq!(flow.paragraphs.len(), 2);
        assert!(flow.auto_add_pages);
    }

    #[test]
    fn a_source_that_is_a_path_to_a_package_file_carries_no_words() {
        let flow = read(serde_json::json!({
            "id": "f", "source": "text/flow-main.md", "frames": ["a"]
        }));
        assert!(flow.paragraphs.is_empty());
    }

    #[test]
    fn paragraphs_are_written_and_read_back_the_same() {
        let flow = Flow::from_text(FlowId("f".into()), "One\nTwo", vec![FrameId("a".into())]);
        let json = serde_json::to_value(&flow).unwrap();
        assert!(json.get("source").is_none());
        assert_eq!(read(json), flow);
    }
}
