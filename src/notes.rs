#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Note {
    pub id: u64,
    pub text: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Note {
    pub(crate) fn title(&self) -> &str {
        self.text
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(str::trim)
            .unwrap_or("Untitled note")
    }

    pub(crate) fn preview(&self) -> String {
        let mut lines = self.text.lines().filter(|line| !line.trim().is_empty());
        lines.next();
        let preview = lines.take(2).map(str::trim).collect::<Vec<_>>().join(" · ");
        if preview.is_empty() {
            "Open this note to read or edit it.".into()
        } else {
            preview
        }
    }
}

// Produce a candidate snapshot. The app only adopts it after a successful disk
// write, so failed saves never advance a card's saved date or lose its old text.
pub(crate) fn edited_notes(
    notes: &[Note],
    active: Option<u64>,
    text: &str,
    now: u64,
) -> (Vec<Note>, Option<u64>) {
    let mut result = notes.to_vec();
    let mut active = active;
    if let Some(note) = active.and_then(|id| result.iter_mut().find(|note| note.id == id)) {
        if note.text != text || note.updated_at == 0 {
            note.text = text.to_string();
            note.updated_at = now;
        }
    } else if !text.trim().is_empty() {
        let id = notes
            .iter()
            .map(|note| note.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        result.push(Note {
            id,
            text: text.to_string(),
            created_at: now,
            updated_at: now,
        });
        active = Some(id);
    }
    result.sort_by_key(|note| std::cmp::Reverse((note.updated_at, note.id)));
    (result, active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_notes_keep_previous_cards_and_have_unique_ids() {
        let (notes, first) = edited_notes(&[], None, "First\nDetails", 100);
        let (notes, second) = edited_notes(&notes, None, "Second", 100);
        assert_ne!(first, second);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[1].text, "First\nDetails");
        assert_eq!((notes[0].created_at, notes[0].updated_at), (100, 100));
    }

    #[test]
    fn editing_changes_one_card_and_preserves_creation_date() {
        let (notes, first) = edited_notes(&[], None, "First", 100);
        let (notes, _) = edited_notes(&notes, None, "Second", 110);
        let (edited, _) = edited_notes(&notes, first, "Updated first", 120);
        assert_eq!(edited[0].text, "Updated first");
        assert_eq!((edited[0].created_at, edited[0].updated_at), (100, 120));
        assert_eq!(edited[1], notes[0]);
        assert_eq!(notes[1].text, "First");
        assert_eq!(edited_notes(&edited, first, "Updated first", 130).0, edited);
    }

    #[test]
    fn empty_new_draft_does_not_add_a_card_and_previews_are_readable() {
        assert!(edited_notes(&[], None, " \r\n ", 100).0.is_empty());
        let note = Note {
            id: 1,
            text: "\n Meeting notes\r\n First item\r\nSecond item".into(),
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(note.title(), "Meeting notes");
        assert_eq!(note.preview(), "First item · Second item");
    }

    #[test]
    fn saving_an_imported_note_records_save_time_without_inventing_creation_time() {
        let imported = Note {
            id: 1,
            text: "Legacy note".into(),
            created_at: 0,
            updated_at: 0,
        };
        let (notes, _) = edited_notes(&[imported], Some(1), "Legacy note", 120);
        assert_eq!((notes[0].created_at, notes[0].updated_at), (0, 120));
    }
}
