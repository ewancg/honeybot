# honeybot
A very simple Discord bot which acts on users who post in "honeypot" channels.

A "honeypot" channel is designed to catch spam accounts which post indiscriminately in every channel, which is becoming more common for things like crypto scams, desparate job hunters, and people who don't know how to read.

## Setup

All inputs are sourced via. environment variables. They can also be specified as long arguments. Consult `--help` for the details. This is a sample configuration:

```
export DISCORD_TOKEN="XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
export WATCHLIST_CHANNEL_IDS="00000000000000000000,1111111111111111111"
export WHITELIST_USER_IDS="2222222222222222222"
export WHITELIST_ROLE_IDS="3333333333333333333"
export LOGGING_CHANNEL_ID="4444444444444444444"
export VIOLATION_ACTION="ban"
export ADMIN_ROLE_ID="5555555555555555555"
export BAN_DELETE_MESSAGES_X_DAYS_PRIOR=1
```

The bot requires the following permissions:
  - "Ban Members"
  - "Kick Members"
  - "Manage Messages"

You will also want to give your application the "bot" scope on the "installation" page of the Discord developer portal.
