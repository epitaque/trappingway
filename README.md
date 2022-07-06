# trappingway
Discord bot that displays FFXIV listings

## How to run this bot
1. Create a discord application, and within that application a discord bot. Note the bot api token. Enable message content intent on the "bot" page of the discord developer portal. Create the bot invite link and invite the bot to your discord server, with the send messages and application commands permissions.
2. Install rust.
3. Run these commands:
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
4. In your discord server, type @(your bot name) register. Click one of the green buttons. This is to register the slash command, `display_xivpfs`.
6. Type /display_xivpfs and some command parameters should autocomplete for you.
