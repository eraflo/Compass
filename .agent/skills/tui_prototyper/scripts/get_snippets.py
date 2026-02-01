import sys

SNIPPETS = {
    "layout": "let chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref()).split(f.size());",
    "block": "Block::default().title(\"Step View\").borders(Borders::ALL).border_type(BorderType::Rounded)",
    "list": "let items: Vec<ListItem> = steps.iter().map(|s| ListItem::new(s.title.as_str())).collect();"
}

if __name__ == "__main__":
    key = sys.argv[1] if len(sys.argv) > 1 else ""
    print(SNIPPETS.get(key, "Available snippets: layout, block, list"))