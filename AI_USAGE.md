# AI Transparency

## Overview
This document is fully outline and be transparent in how AI was used in this project.
The reason I am making this is because I (and I assume others) have some reservations about
using a project with any AI usage in it; primarily due to security, privacy, and performance concerns.
So, I wanted to describe how I used AI in this project to give any users/readers information about
whether or not they want to use this project or not. Additionally, I am writing this document
without any AI assistance to try to drive home the point that I actually care about this project
and the quality of it.

## About the Project and Me
This sections can mostly be skipped if you don't care about the how and why the project
came about, nor about my software background. Go to [Decisions](#decisions) to skip over this section.
This is for readers who want to be more critial in the project's AI usage and my ability to manage the project.

The impetus of this project was that I wanted a self hosted chat app my friends and I could use without
the worry of a company or anyone else being able to have access to our private conversations and data.
Especially after the privacy scare Discord had when they introduced age verification in (around early 2026).
I tried alternatives like Spacebar/Fermo, Fluxer, Stoat, TeamSpeak, Matrix, etc; and while all of those are
perfectly fine apps, they all either didn't have features I wanted or had way too many that I didn't need, or 
they had way too much setup involved to be worth the process of bothering to set them up.

I jokingly said that I would just make my own chat instead since I'd at least be able to add what we need.
Mid 2026, I started to actually try to see if I could make a simple chat app to see if the idea would work.
I thought about Dart/Flutter, Python/Qt, Rust/Slint, but could never nail anything down that felt like it would
work for the group's needs. Since I needed to have it work for Linux, Windows, iOS, and Android; I eventually settled
on a bare rust server backend with a web front-end.

There were two main reasons why I picked Rust as the backend and TS as the frontend. One, I wanted to learn Rust
in general but never had a real reason to. Two, web had the least friction in terms of getting a client for everyone
to use, especially since I had no interest in making an Apple developer account and paying $100 for like 2 people
to use a mobile app.

In terms of my background, I have 6 years of software development experience (as of writing this) and a CS degree
(undergrad and post-grad). I mostly work on GUI development with Python/Qt and also do testing (unit, functional, integration, etc);
and have worked in the past with DevOps and automated deployments. Personal project-wise it's all game development.
I have very little web and database background, so going in that is a weak area that I've had to learn about while
making this.

## Decisions
This section is about how certain decisions were made and which ones were made either: not at all, partially, or completely by AI.

### Not At All
- Rust as the server
- TypeScript as the frontend langauge on the web client
- Hosting methods (reverse proxy, TLS, etc)
- Layout of project/code organizationally
- CLI/GUI model split and interaction between dependency independence (ie. API doesn't know about GUI/CLI, types don't know about API, etc)
- REST routing endpoints
- File validation and limits (trying to keep configuration user dependent, not random limits I decide)
- Server implementation details (deciding that rooms for example are no different from the concept of a "DM", room types, etc)
  - Entire layout and format of the server I completely decided. Meaning I wanted a flat server with rooms, room scoped calls, per user permissions, admin control, etc.
- Listener System (for clients), in that I wanted UI completely or mostly detached from logic since a user's details should have no hard dependency on the UI

### Partially
- "bugs" reported by AI, and making a list for human review (on whether or not to fix or skip)
- SQL files (their data-type structure), human review on names and real use case testing (what tables can be split or removed outright)
- Comment/Docstring details (human review and/or edited, I try to keep direct and short)
- What frontend library to use, I was set on TypeScript for scripting but not sure for the layout and presentation. AI found Svelete/Vite and after reviewing I found it suitable.
- DB Type, I was set on simple and SQL but unsure if PostGres or SQLite was the better choice. After back/forth and more research I found that despite PostGres having more verbose types that it was more setup needed than SQLite for end users.

### Completely
- Which crates to use in the server (human oversight, reviewed packages for overlap)
- WebRTC implementation (completely vibe-coded)
- Server socket implementation (human oversight, not strong background in WebSockets)
- Xbox/Push services (human oversight, REST endpoints AI discovered) 

### How decisions are made and code reviewed
Given my background (professional software developer and CS degree) I constantly was critial of duplicate or "this fixes the problem for the initial prompt" code
since I notice that AI has a particular bias towards "compilable" code that "works" for a particular problem, but doesn't scale well or uses "tricks" to get something working.
I have a particular goal in mind with code this project (short, concise, and reusable) and if code doesn't meet that criteria I tend to either reprompt to get new results or take
existing generated blocks and manually reorganize it myself.

For example, when creating the web client I noticed that in order to display a user's username in a room member column
the member component had to take in members, and the member row took in a member type, and copied it plain. So if a user changed their name (or anything else) the method making
that change would need to have direct access to the member column and row to update it, as well as the identity tab, and everywhere else the user's details would need to be updated.
If I had no reviewed the code the AI generated code would have created multiple layers of dependencies, making it almost impossible to change or add new details to a user. Having a
"human in the loop" helped remove that issue and I went and started implementing the subscriber/listener code before pointing the AI agent in the new direction.

Additionally, I am constantly critial of any decision made that seems like it's a particular edit to fix a bug or "issue". What I mean is that if some code is not behaving correctly,
I shouldn't need to edit 10 files and 100 lines of code to fix it; unless the entire design was faulty to begin with. Most of these suggestions get outright rejected and I start with a
new prompt or implement myself.

Memory and warnings are two other areas I am critial about, with web particularly I try to squash as many warnings as I can in order to clean up and patch areas I did not know
existed in the code. With memory I run in debug mode with the server to see if any actions cause any memory leaks or if any task is taking longer than it reasonbly should. I tend to
have agents run though a couple of thier own tests to vaguely check if anything obvious is faulty.

In terms of bug finding and fixing I occasionally will prompt an agent to find them in the repo. From what I'd find about 50-60% of the reported "bugs" are nonsense or pedantic and
ignore them, and the rest I manually review to see if there is an actual bug or unexpected behavior. I also found that asking "for" bugs will always generate a response from an agent
of it finding bugs, even if you give it clean code.

When it comes to decision making (ie new featues or code) I tend to use broad ideas with no code generation at first to workshop ideas, then get more specific, dervive a plan, then
have an agent generate a report/document on it. I review the document several times and edit it if I find anything incorrect and if I find it to be fine I create a new session for an
agent to implement. The size of these features varies, but I err on the side of smaller changes because it leaves more chance for me to stop and fix or change anything.

AI Usage goes as follows:
- Known bug fixing and simple feature implementation (entirely manual)
- Medium feature implementation (human derived plan, AI review, AI plan, human review, AI code generation, human review, 2 previous steps until done)
- Large feature implementation (two paths)
  - Break into smaller pieces (make into medium feature implemenation)
  - Entirely AI generate and manual test feature to see if it is what I want, then revert code changes and write plan on feature

### Code Style
Like most agents I have MD files in my project root to make sure that code styling is being followed and that known commands work for server hosting, web building, client building, etc.

The comment style I typically follow is, short concise comments that describe the code as-is; not it's intent or extrapolating on what it may or may not do. Method docstrings can describe
intent and usage, but they should be generalize and not describe exactly what the method is doing outright. Inline comments are the job of describing the code beneath it, methods shouldn't
be paragraphs that are already self-evident. I typically like to follow the language's recommended comment style, but sometimes (ie rust) it feels overbearing and too verbose so I tend to stick
to simple `///` comments for docstrings and `//` for inline code.

In terms of code itself, obviously follow the language's indentation and bracket style but also I prefer readable code over dense code. What I mean is that while lamdas can be great at making
code smaller, it sometimes is harder to read and harder to test. Even if the dense code is "faster" it means I am slower at reading and parsing it. Similarly, with a language like Rust I typically
don't like using lifetimes as it makes following and managing memory harder (at least to me), so I tend to avoid it unless forced. I am fine wasting some memory if code is easier to read if passing
by copy instead of reference (though Rust does makes it more straightforward compared to a language like python).

With file size, I tend to split if the contents of a file get a bit muddy (ie multiple different methods doing different things makes a file name harder to understand) or if a file is long (500+)
when it could be split into component parts (ie. server config could be one long file, but multiple smaller files based on sections is easier to read). Similary folder/module structure follows
the same structure, named after what it does and split into subfolders of relevant names.


## Agents Used
- Claude (Opus 5, Sonnet 5.5, Opus 5.5)
- Qwen (3.6 27B, 3.6 35B-A3B, 3.8 27B)
