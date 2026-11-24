# About
This document describes the reason this project exists and my background (TheChiefu) to better
help project readers and contributers understand the decisions made during the development of it.

## The Project
The impetus of this project was that I wanted a self hosted chat app my friends and I could use without
the worry of a company or anyone else being able to have access to our private conversations and data.
Especially after the privacy scare Discord had when they introduced age verification around early 2026.
I tried alternatives like Spacebar/Fermo, Fluxer, Stoat, TeamSpeak, Matrix, etc. While all of those are
perfectly fine apps, they all either didn't have features I wanted or had way too many that I didn't need 
(they also sometimes had way too much setup involved).

Around mid 2026, I started to see if I could make a simple chat app to replace Discord for my group.
I thought about using Dart/Flutter, Python/Qt, Rust/Slint, but never felt like any of them would work for the group's needs.
I needed to have a client work for Linux, Windows, iOS, and Android, so I eventually settled on a bare rust server backend
with a web front-end.

The reason why I picked Rust as the backend and TypeScript as the frontend was that I wanted to learn Rust
in general (I never had a real reason to) and TypeScript has type checking built in which makes doing web more more managable to me.
Plus it's well supported and follow Javascript (keeps up with web standards).

I wanted something simple I could just deploy with a single script instead of setting up a bunch of
services and files in order to get a chat up and running. The problem I saw with most Discord alternatives was that
they were trying to emulate the Discord environment, in that there is a single server with a CDN multiple "guilds" and channels in those "guilds".
Then they added clients in the mix that then needed to point to the correct hosted server (some didn't even let you self host) and it was
honestly a mess.

So I decided that the entire server management needed to take place in the executable location and be self contained.
In that I have a simple Sqlite DB file for credentials, permissions, etc; files stored in a `files` folder, and any configuration
is stored next to the file. It has the additional benefit that if I want to re-deploy or remove the server it's just deleting a directory,
no stray files or random services I have to remember to remove.

For the client I had two approaches, one (which is done and working) was a static web page that reacts to events and the other which is a
dedicated client that reacts similarly but doesn't require a web server. The web client is for the broadest compatability so that most users
who want to simply chat can just use it without any issue (as long as someone hosts). The reason it is a single page is because it makes developing
easier and doesn't require knowing about which subpage or section a user is on in the client. The downside is that it requires someone to host a web
server and manage that. The dedicated client is in development and is supposed to make it so someone deploying a server can do just that
and nothing else. Tell people your IP or domain and the client connects.

Overall, making it so as a deployer it's just run the server executable and you're good; deploy the web client if you have a domain or let
users use the dedicated client (and they can connect to any server). No extra services, no background processes, just the server at the bare minimum.
I've added compiler flags to people who want to add or remove features not relevant to them, since like I said earlier this repo was made for my
friends' group specific needs so I don't think everyone would want every feature I have added.

## Me (TheChiefu)
I am adding this section mainly as an explaination of how background effects the development of this project.

As of writing this, I have 6 years of software development experience, a CS undergrad degree, and finishing up a CS graduate program.
At work I do GUI development with Python/Qt, as well as do testing (unit, functional, integration, etc); making an application that deploys both to
Windows and Linux. Mostly dealing with large requirement repositories in multiple services (Codebeamer, Jira, IBM ELM, etc). I have previously worked
at a large company with a devops and automated deployment background.

In terms of personal projects, they have all more or less been game development. Unity, Godot, and direct (C++/C#) CLI games at a small scale
(1-5 or so users/players). Mostly 3D related and am most knowledgable in the UI/Input systems side of development. I have some small web development
skills (made static sites and small reactive ones in Java/Springboot) as well as used SQLite in the past for these, but nothing extensive.

I prefer strongly typed languages since it helps keep data consistent and easier to navigate where data is going in terms of debugging. It
also helps in IDEs where making a change can update something everywhere it's used without having to manually track down or write a script.

All in all, the point I am trying to get at is that I have some semblance of a programming background and that despite using AI in this project
I am not using it aimlessly or accepting anything it gives me. I am very particular with code and the design of the codebase, especially in terms of
trying to maintain layers so there is no dependency issues or overlapping responsibility of code.