# trappingway
Discord bot that displays FFXIV listings. Goal is to display information to make the party finder experience better (as in, having pfs fill more quickly).

![image](https://user-images.githubusercontent.com/7350617/178365515-c3431fbe-bd8b-47ac-9d07-f60624ed8b1d.png)

If you have a large discord server you can send a dm to Epitaque#3378, and I'll send you the bot invite link. Unfortunately, due to discord ratelimits, I can't let everyone add this bot to their server. However, you can make a clone by following the below instructions.

## How this works
It scrapes [https://xivpf.com/listings](https://xivpf.com/listings) for current listings. xivpf gets its data from people who have the Remote Party Finder plugin on Dalamud.

## How to run this bot
1. Create a discord application, and within that application a discord bot. Note the bot api token. Enable message content intent on the "bot" page of the discord developer portal. Create the bot invite link and invite the bot to your discord server, with the send messages and application commands permissions.
2. Install rust.
3. Add the emojis to a server that this bot is in. Edit xiv_util.rs line 110 with the emoji codes (can be found by sending the message \<the emoji i.e. :dps:> in a discord channel with the emoji).
4. Run these commands:
```
git clone git@github.com:epitaque/trappingway.git
cd trappingway
cargo install sqlx-cli

IF ON WINDOWS POWERSHELL
$env:DISCORD_TOKEN="<your discord api token>"
$env:DATABASE_URL="sqlite:database.sqlite"
$env:MAX_LISTINGS_IN_POST=8

IF ON LINUX
DISCORD_TOKEN=<your discord api token>
DATABASE_URL=sqlite:database.sqlite
MAX_LISTINGS_IN_POST=8

sqlx database create
sqlx migrate run

cargo install
cargo build --release
./target/release/ffxiv_pf_bot<.exe if on windows>
```
5. In your discord server, type @(your bot name) register. Click one of the green buttons. This is to register the slash command, `display_xivpfs`.
6. Type /display_xivpfs and some command parameters should autocomplete for you.
7. Please consider not changing the update interval, as the owner of xivpf.com probably doesn't want a bunch of bots scraping on a frequent interval. They told me 5 minutes was an acceptable interval.

## Other projects
Looks like Veraticus made a discord bot that does a similar thing in Go. [Link](https://github.com/Veraticus/trappingway).
