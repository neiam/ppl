# PPL

A Rust CLI tool for managing and tracking significant dates for the people in your life.

## Features

Just enough to get by.

### Storage

Shipping a syncable database for the following information

- [x] PPl 
  - [x] Contact information
  - [x] Tiers
  - [x] Traits
  - [x] Significant Dates
- [x] Tier Defaults ( Emoji + Color )
- [x] Trait Defaults ( Emoji + Color )
- [x] Reminder Configuration

### MOTD

Put a message of the day in your bashrc to remind you of important dates.
```ppl motd```

![motd.png](screenshots/motd.png)

### TUI

A Terminal UI for managing all the various stored information in the database.

#### Default View

PPL List, shows all configured ppl, their tiers, traits and information

![default.png](screenshots/default.png)


#### Calendar View

Upcoming Anniversaries

![calendar.png](screenshots/calendar.png)

#### Tier View

Add and Modify Tiers and Defaults

![tiers.png](screenshots/tiers.png)

### Trait View

Add and Modify Traits and Defaults

![img.png](screenshots/traits.png)