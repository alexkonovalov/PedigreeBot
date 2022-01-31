<h3 align="center">Pedigree Bot</h3>

<p align="center"> 🤖 Telegram bot that builds a directed graph which represents family tree by means of dialogue
    <br> 
</p>

## Table of Contents

- [Usage](#usage)
- [Built Using](#built_using)
- [Authors](#authors)

## 🎈 Usage <a name = "usage"></a>

<div align="center">
  <kbd>
    <img src=./media/DEMO.gif />
  </kbd>
</div>
<br/>

To use the bot, find the bot in a search bar by typing it's name:

```
@PedigreeBot
```

OR just open the the [@PedigreBot link](https://t.me/pedigreebot)

To start the dialog type
 
```
/start
```

command. 

The bot will then ask you questions about the family members.
Continue answering until the bot sends you the following message:

> "We asked enough! you can get your pedigree chart by performing /finish command"

Then type

```
/finish
```
.

OR finish earlier with the aforementioned command and receieve an incomplete tree.

### To run a local demo

1. Copy `.env.example` to `.env`
2. Set up graphviz by installing `apt-get install -y graphviz`
3. Set up your telegram demo account through [@BotFather](https://t.me/botfather)
4. Save telgram token in `.env`
5. Set up https tunneling to your local machine
6. Save `port` and `server_url` in `.env`

## ⛏️ Built Using <a name = "built_using"></a>

- [Teloxide](https://docs.rs/teloxide/latest/teloxide/) - An elegant Telegram bots framework for Rust
- [Petgraph](https://docs.rs/petgraph/latest/petgraph/) - Graph data structure library

## ✍️ Authors <a name = "authors"></a>

- [@alexkonovalov](https://github.com/alexkonovalov) - Idea & Work

