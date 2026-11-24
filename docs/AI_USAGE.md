# AI Usage

## Overview
This document is fully outline and be transparent in how AI was used in this project.
The reason I am making this is because I (and I assume others) have some reservations about
using a project with any AI usage in it; primarily due to security, privacy, and performance concerns.
So, I wanted to describe how I used AI in this project to give any users/readers information about
whether or not they want to use this project or not. Additionally, I am writing this document
without any AI assistance to try to drive home the point that I actually care about this project
and the quality of it.

## About the Project and Me
This sections can be skipped if you don't care about the how and why the project
came about, nor about my software background. Go to [decisions](#decisions) to skip over this section.
This is for readers who want to be more critial in the project's AI usage and my ability to manage the project.

The impetus of this project was that I wanted a self hosted chat app my friends and I could use without
the worry of a company or anyone else being able to have access to our private conversations and data.
Especially after the privacy scare Discord had when they introduced age verification in (around early 2026).
I tried alternatives like Spacebar/Fermo, Fluxer, Stoat, TeamSpeak, Matrix, etc; and while all of those are
perfectly fine apps, they all either didn't have features I wanted or had way too many that I didn't need 
(they also sometimes had way too much setup involved).

Mid 2026, I started to try and see if I could make a simple chat app. I thought about Dart/Flutter,
Python/Qt, Rust/Slint, but could never nail anything down that felt like it would work for the group's needs.
Since I needed to have it work for Linux, Windows, iOS, and Android; I eventually settled on a bare rust
server backend with a web front-end.

There were two main reasons why I picked Rust as the backend and TS as the frontend. One, I wanted to learn Rust
in general but never had a real reason to. Two, TypeScript has type checking built in and follows JS to keep
up with web.

In terms of my background, I have 6 years of software development experience (as of writing this) and CS degrees
(undergrad and post-grad). At work I work on GUI development with Python/Qt and also do testing (unit, functional, integration, etc);
and have worked in the past in devops/automated deployments. In terms of personal projects it's all game development.
I have very little web and database background, so going in that is a weak area that I've had to learn about while
making this.

## Decisions
This section is about how certain decisions were made and which ones were made either: with no AI at all, partially AI guided, or completely done by AI.

### No AI at all
- Deciding Rust as the server language
- Deciding TypeScript as the frontend langauge on the web client
- Hosting decisions (allow/use reverse proxy, using TLS, etc.)
- Organization of project/code
- CLI/GUI model split and interaction between dependency independence
  - API doesn't know about GUI/CLI
  - Types don't know about API
  - Separation of actions for better code management
- REST routing endpoints
- File validation and limits (trying to keep configuration user dependent, not random limits I arbitrarily decide)
- Server implementation details:
  - Deciding that rooms for example are no different from the concept of a "DM", since DMs are just hidden 2-party rooms
  - That rooms have types (Public or Private/Hidden)
  - Game activity reporting on different platforms
  - Flat server layout with rooms that determine membership interactions, rather than a Discord-like "Guild" + Channel split
  - Room scoped calls instead of server-wide call
  - Per room user permissions
  - Etc.
- Listener System
  - I wanted UI to be detached from logic since a user's details should have no hard dependency on the UI and vice-versa

### Partially guided by AI
- "bugs" reported by AI, and making a list for human review
  - Lists were reviewed on whether or not to fix or skip items reported
- SQL files (their data-type structure) and human review of:
  - Names
  - Real use case testing (what tables can be split or removed outright)
- Comment/Docstring details (human review and/or edited)
- What frontend library to use:
  - I was set on TypeScript for scripting but not sure what library to use for the layout and presentation
  - AI found Svelete/Vite and after reviewing I found it suitable
- Database Type:
  - I was set on simple and SQL but unsure if PostGres or SQLite was the better choice
  - After back/forth I found that despite PostGres having more verbose types that it was more setup needed than SQLite for end users

### Completely made by AI
- Which crates to use in the server (human oversight):
  - I have a hard rule that no new dependencies should be added unless strictly necessary for a feature
  - Re-implementing something from scratch would be easier to maintain if it is smaller than pulling in a crate with tons of non-needed code
  - Checked for overlapping packages to reduce depdencencies
  - Setup `--features` flag so users can remove packages/services they do not need from build
- WebRTC implementation (completely vibe-coded):
  - Will eventually revisit (or outright remove), this was all done with little review since I wanted a quick way to get video/screenshare calls working
- Server socket implementation:
  - I am not too familiar with web sockets so I had to do some research while reviewing, but I mostly followed the recommended code edits for message/activity reporting
- Xbox/Push services (human oversight):
  - AI found endpoints I did not know existed (for xbox) and I found it useful for it to determine how to get Xbox profile status (it did also suggest websocket for this)
  - Told it to provide URLs as documentation and where it found relevant endpoints so that I can review them
  - Push service is still mostly a blackbox to me but a must for mobile users, so I let that code be AI driven (also told it to provide spec and URL in comments)

### How decisions are made and code reviewed

#### Dealing with over-enthusiatic edits
Given my background (professional software developer and CS degree) I constantly was critial of duplicate or "this fixes the problem for the initial
prompt" code since I notice that AI has a particular bias towards "compilable" code that "works" for a particular problem, but doesn't scale well or
uses "tricks" to get something working. I have a particular goal in mind with code this project (short, concise, and reusable) and if code doesn't
meet that criteria I tend to either reprompt to get new results or take an existing generated block and manually reorganize it myself.

For example, when creating the web client I noticed that in order to display a user's display name in a room member column,
the member component had to take in members, and the member row had to take in a member object, and that it copied the value plain.
So if a user changed their name (or anything else) the method making that change would need to have direct access to the member column,
the row to update it, the identity tab, **and** everywhere else the user's details would need to be updated. If I had not reviewed the AI
generated code it would have created multiple layers of dependencies. Which in of itself would have made it almost impossible to change or add new
details to a user without changing everything it touches. Having a "human in the loop" helped remove that issue and I started implementing the
subscriber/listener code before pointing the AI agent in the new direction.

I am constantly critial of any decision made that seems like it's a particular edit to fix a single bug or "issue".
What I mean is that if some code is not behaving correctly, I shouldn't need to edit 10 files and 100 lines of code to fix it;
unless of course the entire design was faulty to begin with. Most of these suggestions get outright rejected and I start with a
new prompt/session or I implement myself.

#### Scoping changes
In terms of bug finding I occasionally will prompt an agent to find any in the repo. From what I've found about 50-60% of the reported "bugs" are
nonsense or pedantic (called an error but isn't) and I ignore them. The rest I manually review to see if there is an actual bug or unexpected behavior,
but found that asking "for" bugs will always generate a response from an agent of it finding bugs. So I tend to be skeptical of reported bugs, though
it does occasionally find some so it is somewhat useful.

When it comes to decision making (ie. new featues or code) I use broad ideas with no code generation at first to brainstorm broad ideas I want to
narrow down. Use it to get more specific, dervive a plan, then have an agent generate a document on it. I then review the document several times and
edit it if I find anything incorrect, then create a new session for an agent to implement the details. The size of these features varies, but I err on
the side of smaller changes because it leaves more chance for me to stop and fix or change anything. I prefer line by line edit instead of invisible
`sed` edits since it allows me to visually look at diffs before any real code change occurs. This has helped me out some many times since
it lets me stop an agent before it starts going in a completely wrong direction.

AI Usage goes as follows:
- Known bug fixing and simple feature implementation (entirely manual)
- Medium feature implementation (human derived plan, AI review, AI plan, human review, AI code generation, human review, 2 previous steps until done)
- Large feature implementation (two paths)
  - Break into smaller pieces (make into medium feature implemenation)
  - Entirely AI generate and manual test feature to see if it is what I want, then revert code changes and write plan on feature

## Final Notes
Using AI agents has help speed up the project in terms of getting it to a deployable state, as someone who does software development full time and as a
hobby, a part of me wants to do this 100% with no AI. However, since it's used by not just me but also as a real messaging app for my friends I do
defer parts that I want "just done" to be AI coded or assisted to push a feature to useful completion. I am not completely happy about doing this, but
also try to find a balance of reasonable time spent researching, implementing, and creating a feature versus just getting something basic out that can
be immediately used. Just like production work, sometimes getting something out there is better than spending an infinite amount of time "perfecting"
code.

I have often wondered too, how much time am I wasting deferring code implementation to an AI agent and critiquing it versus doing it on my own. I find
AI useful for getting started on a topic and manually diving deeper (I didn't even know Svelte existed before an agent suggested it). But I've noticed
I spend more time than should reviewing AI code than if I coded something myself. Changing the balance from code production and review towards code
review and decision fatigue. But that's more so something I'll have to worry about rather than someone reviewing this repo, but it aligns with research
that states that agentic coding doesn't accelerate coding, more so shifts time in different directions.

## Agents Used
- Claude (Opus 5, Sonnet 5.5, Opus 5.5)
- Qwen (3.6 27B, 3.6 35B-A3B, 3.8 27B)
