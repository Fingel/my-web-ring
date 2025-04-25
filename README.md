# My Web Ring

My Web Ring, mwr for short, is a local-first RSS reader that also supports non-feed webpages and youtube channels.
MWR is not a typical RSS reader: launching
`mwr` (or accessing it via http) will open your browser to an unread pseudo-random page.

https://github.com/user-attachments/assets/3b8c5973-689d-429f-88e8-b36f0988d401

# Motivation

Tired of doomscrolling on websites like Hacker News, I wondered if there could
be a better way. I came to 3 realizations:

1. Most content I actually care to read is already on blogs that provide feeds.
2. I hate RSS readers.
3. I do most of my exploratory reading during downtime/procrastination.

MWR works for me because it surfaces content pseudo-randomly that I actually
care about. It has completely replaced  doomscrolling for me.

## Why Not an RSS Reader?

I hate how RSS readers tend to make me feel behind: It's too easy for me to see
the unread count and feel like I need to keep up. With MWR there is no pressure to read
everything, and the page selection algorithm still favors newer content (more on that below).


# Page Selection Algorithm

What page you get from MWR when you ask for one is determined by a simple algorithm:

* All sources (feeds or websites) start at weight 10: you can change weight on a source by source basis.
* Newer content is weighted higher than older content.
* Non-feed websites become "unread" each time you launch MWR, also start at weight 10, and have
a "newness" score of 5 days.

# Installation
Binaries are available from the [releases](https://github.com/Fingel/my-web-ring/releases) page.

You may need to install `sqlite3`.

# HTTP interface
MWR includes a simple HTTP server so that you can use it without the terminal (in other words: on your
phone.) Running `mwr server` will start it on port  8090. Accessing the server simply returns redirects to
pages. I may expand this in the future.

# Options

```bash
❯ mwr --help
Usage: mwr [COMMAND]

Commands:
  run        Select a page and start the CLI interface (default)
  open       Select a page from a specific source
  pull       Fetch new pages (normally runs in the background on launch)
  list       List all sources
  add        Add a new source
  mark-read  Mark source as read
  delete     Delete a source
  backup     Backup sources and pages to stdout
  restore    Restore sources and pages from stdin
  server     Start the HTTP server
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
# Adding Youtube channels
Youtube channels have secret RSS feeds:

https://www.youtube.com/feeds/videos.xml?channel_id=<channel_id>

You can find the channel ID by clicking "more" in the channel description,
share channel, copy channel ID.
