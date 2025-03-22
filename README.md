# My Web Ring

My Web Ring, mwr for short, local-first rss reader that also supports single
sites. MWR is not a typical rss reader - you won't be able to see
all your feeds and their hundreds of unread items. Instead you simply launch
`mwr` (or access it via http) and you are redirected to a pseudo-random page that
you haven't read yet.
![Screenshot From 2025-03-22 16-21-27](https://github.com/user-attachments/assets/5496e588-c18b-4b34-926d-8a1c4bf1afb2)

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
