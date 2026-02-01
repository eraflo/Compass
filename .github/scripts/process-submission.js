const fs = require('fs');
const path = require('path');

// 1. Get the Issue Body from the environment
const issueBody = process.env.ISSUE_BODY;
if (!issueBody) {
  console.error("No ISSUE_BODY found in environment.");
  process.exit(1);
}

// 2. Parse the Issue Form (Markdown)
// We look for sections like "### Runbook Name\n\nmy-name"
function extractValue(header) {
  const regex = new RegExp(`### ${header}\\s+([\\s\\S]*?)(?=###|$)`, 'i');
  const match = issueBody.match(regex);
  return match ? match[1].trim() : null;
}

const name = extractValue('Runbook Name');
const description = extractValue('Description');
const url = extractValue('Raw URL');
const tagsRaw = extractValue('Tags');
const authorRaw = extractValue('Author (Optional)');

if (!name || !url || !description) {
  console.error("Missing required fields (Name, URL, or Description).");
  process.exit(1);
}

// 3. Prepare the new entry
const newEntry = {
  name: name,
  description: description,
  url: url,
  tags: tagsRaw ? tagsRaw.split(',').map(t => t.trim()).filter(t => t.length > 0) : []
};

if (authorRaw && authorRaw.toLowerCase() !== 'none') {
    newEntry.author = authorRaw;
}

// 4. Update registry.json
const registryPath = path.join(__dirname, '../../registry.json');
let registry = [];

try {
  const content = fs.readFileSync(registryPath, 'utf8');
  registry = JSON.parse(content);
} catch (e) {
  console.error("Failed to read registry.json", e);
  process.exit(1);
}

// Check for duplicates
if (registry.find(r => r.name === newEntry.name)) {
  console.error(`Runbook with name '${newEntry.name}' already exists.`);
  process.exit(1);
}

// Add and Sort
registry.push(newEntry);
// Optional: Sort alphabetically
registry.sort((a, b) => a.name.localeCompare(b.name));

// 5. Save
fs.writeFileSync(registryPath, JSON.stringify(registry, null, 2));
console.log(`Added ${newEntry.name} to registry.`);
