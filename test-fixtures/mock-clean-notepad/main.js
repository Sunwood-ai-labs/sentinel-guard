// Mock clean local-only notepad app (should score CLEAN / 0 with zero false positives)
// Reference doc link only (should be ignored as noise): https://example.com/docs/telemetry

export class LocalNotepad {
  constructor() {
    this.notes = [];
  }

  addNote(title, body) {
    const entry = {
      id: Date.now(),
      title: String(title),
      body: String(body),
      updatedAt: new Date().toISOString(),
    };
    this.notes.push(entry);
    return entry;
  }

  listNotes() {
    return [...this.notes];
  }

  search(keyword) {
    const q = String(keyword).toLowerCase();
    return this.notes.filter(
      (n) =>
        n.title.toLowerCase().includes(q) || n.body.toLowerCase().includes(q)
    );
  }
}

// Padding to exceed 4KB bundle threshold
export const CLEAN_APP_PADDING = `
${"// clean-local-editor-chunk-0011223344556677\n".repeat(100)}
`;
