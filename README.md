# My Web Ring

My Web Ring, mwr for short, local-first rss reader that also supports non-feed webpages. 
MWR is not a typical rss reader: launching
`mwr` (or accessing it via http) will open your browser to a single pseudo-random page.

https://github.com/user-attachments/assets/3b8c5973-689d-429f-88e8-b36f0988d401

Why do it this way?

Occasionally I'll want a quick coding break. Usually I'll reach for something
toxic like Hacker News or reddit or if I'm really depressed, even Google News.
With MWR instead I'll just run it once or twice, see content from authors that
I actually want to read and save myself some mental illness. This is my
solution to doom-scrolling.

## Features

* Add RSS or static pages. Static pages never count as read and will
be included in the random page selection algorithm.

* Assign weights to sources. Sources with higher weight get their content
selected more often. Newer content is also weighted higher by default.

* Simple HTTP interface. Does nothing when accessed except redirect to the
selected page. Good for using MWR via your phone if it's running on a desktop
or server.

* All data is stored in a simple Sqlite database, so it's easy to backup or
move elsewhere.

* Completely local software.
