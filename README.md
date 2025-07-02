# stash

an instant, focused note-taking tool that actually makes sense.

tired of complex note apps that get in your way? stash is a minimal, keyboard-driven cli tool that lets you capture, organize, and find your thoughts instantly. think of it as your digital brain's cache layer.

## what is this?

stash is a command-line note-taking tool built in rust that combines:

- ⚡ instant note capture from anywhere in your terminal
- 🔍 powerful search with tags, projects, and fuzzy matching
- 🤖 ai-powered natural language search
- 📝 beautiful terminal ui for browsing and editing
- 🏷️ automatic tag and project extraction
- 💾 markdown files stored locally (your data, your control)

no cloud. no electron. no subscriptions. just fast, local notes that sync however you want.

## why the terminal?

when you need to capture a thought quickly - in a meeting, while coding, during research - you don't want to break flow by opening another app or hunting through browser tabs.

your terminal is always there, always fast. `stash add "quick idea"` and you're back to what you were doing instantly. no context switching, no waiting, no app sprawl.

terminal means keyboard-driven, actually fast, and integrates with however you work. just like `git stash` for code, but for thoughts.

stash isn't for everybody. it's for people who like to move fast and keep things tidy. it runs in the terminal, so if you're comfortable with keyboard shortcuts and minimal interfaces, you'll feel right at home. no distractions, just a clean, frictionless way to capture your thoughts.

## installation

### the easy way (recommended)

```bash
cargo install --git https://github.com/danielarbabian/stash
```

### from source

```bash
git clone https://github.com/danielarbabian/stash
cd stash
cargo build --release
sudo cp target/release/stash /usr/local/bin/
```

### make it feel native

add this to your shell config (`.zshrc`, `.bashrc`, etc):

```bash
alias s="stash"
alias sa="stash add"
alias sn="stash new"
alias ss="stash search"
```

now you can just type `s` to open stash, `sa "quick note"` to add something, etc.

## features that actually matter

### instant capture

```bash
stash add "the key insight from that design talk: start with the problem, not the solution #design"
stash add "check out that paper on distributed systems someone mentioned" --title "to read"
```

### smart search

```bash
# basic text search
stash search "rust async"

# search by tags and projects
stash search "#rust +webapp"

# exclude stuff you don't want
stash search "javascript -#old"

# natural language with ai
stash ai "show me all my rust learning notes from last week"
```

### beautiful tui

```bash
# launches the full interactive interface
stash
```

navigate with vim keys, edit notes in place, and see everything organized exactly how you think.

### automatic organization

- `#tags` get extracted automatically from your notes
- `+projects` too
- links between notes work like you'd expect
- search everything instantly with fuzzy matching

## setup

### first run

```bash
stash
```

stash will create `~/stash/` for your notes automatically. that's it.

### ai setup (optional but recommended)

1. open the tui with `stash`
2. press `s` for settings
3. add your openai api key
4. enjoy natural language search like "find my notes about that bug from yesterday"
5. use ai rewriting in the tui editor - let ai clean up your thoughts

the ai stuff is completely optional - stash works great without it.

## usage

### quick examples

```bash
# add a note
stash add "learned about rust's ownership model today #rust #learning"

# search for rust notes
stash search "#rust"

# search in a specific project
stash search "+webapp auth"

# ai search (if configured)
stash ai "what did i learn about databases last month?"

# open the full ui
stash

# get help
stash --help
```

### the tui (terminal ui)

run `stash` to open the full interface:

- `j/k` or `↑/↓` - navigate notes
- `enter` - view/edit note
- `/` - search
- `n` - new note
- `d` - delete note
- `t` - filter by tag
- `p` - filter by project
- `s` - settings
- `q` - quit

### note format

notes are just markdown files with yaml frontmatter. you can edit them in any editor:

```markdown
---
id: 123e4567-e89b-12d3-a456-426614174000
title: 'my awesome note'
tags: ['rust', 'programming']
projects: ['webapp']
created: 2025-07-02T11:38:47Z
---

# my note content

this is where your actual note content goes. you can use all the markdown you want.

reference other notes, add code blocks, whatever. stash will automatically:

- extract #hashtags as tags
- recognize +project references
- index everything for search
```

### search like a human

the search is powerful:

```bash
# text search
stash search "async await"

# tag search
stash search "#rust"
stash search "#rust #async"

# project search
stash search "+webapp"
stash search "+webapp +backend"

# combinations
stash search "error handling #rust +webapp"

# exclusions
stash search "#javascript -#old -#deprecated"

# case sensitive
stash search --case-sensitive "API"

# list everything
stash search --list-tags
stash search --list-projects
```

### ai search

if you've set up an openai api key:

```bash
stash ai "show me notes about rust error handling"
stash ai "what did i write about databases last week?"
stash ai "find my todos for the webapp project"
stash ai "notes about that authentication bug"
```

it translates your human language into proper search queries automatically.

## configuration

stash stores config at `~/.stash/config.json`:

```json
{
  "openai_api_key": "your-key-here",
  "ai_enabled": true,
  "ai_prompt_style": "custom",
  "custom_ai_prompt": "prompt"
}
```

you can edit this directly or use the tui settings (`s` key).

## file organization

```
~/.stash/
├── notes/
│   ├── 20240115-1030-my-note.md
│   ├── 20240115-1145-another-note.md
│   └── ...
└── config.json
```

notes are stored as individual markdown files. you can:

- edit them in your favorite editor
- version control with git
- sync with dropbox/icloud/etc
- backup however you want

it's just files. no lock-in, no proprietary formats.

## tips & tricks

### rapid capture

set up shell aliases for instant note taking:

```bash
alias idea="stash add"
alias todo="stash add --title todo"
alias til="stash add --title 'today i learned'"
```

### project organization

use `+project` syntax in your notes:

```markdown
working on the +webapp authentication flow.
need to check the +database schema for user roles.
```

stash automatically extracts these as projects for filtering.

### daily notes

```bash
stash add "$(date +%Y-%m-%d) daily standup notes" --title "daily-$(date +%Y-%m-%d)"
```

### git integration

```bash
cd ~/.stash
git init
git add .
git commit -m "initial stash"
```

now your notes are version controlled.

### cross-platform sync

since it's just markdown files:

- use git for version control sync
- dropbox/icloud for simple sync
- or whatever floats your boat

## why stash?

because every other note-taking app is either:

- too complex
- too simple (plain text files with bad search, organisation, etc)
- too cloudy
- too slow
- too mouse-heavy (keyboard >> mouse)

stash is built by someone who actually takes notes for a living. it's fast, local, keyboard driven, and gets out of your way.

## contributing

found a bug? have an idea?

1. check if there's already an issue
2. if not, open one with details
3. or better yet, send a pr

this is a small project but contributions are welcome.

## license

mit license

## troubleshooting

### "stash: command not found"

make sure `~/.cargo/bin` is in your `$PATH`, or copy the binary to `/usr/local/bin/`

### "no stash directory found"

run `stash add "first note"` to initialize, or just run `stash` and it'll create the directory

### ai search not working

1. check you have an openai api key set (`stash` -> `s` -> add key)
2. make sure you have internet connection
3. verify your api key has credits

### notes not syncing

stash doesn't sync by default - it's just local files. set up git, dropbox, or whatever sync solution you prefer

---

built with ❤️ and redbull. if this saves you time, consider starring the repo.
