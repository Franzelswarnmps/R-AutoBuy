# About
This bot was created to allow for easy automation of common browser tasks.
All configuration of the tool is done inside sites.toml, include all the
browser steps.

The bot was originally made to automate buying a 5900X off Amazon,
but it can probably be used for other things.

# Known Issues
* Entering a frame and clicking a button which causes a page change
will cause an unrecoverable error.

# Todo
* Add a config field for geckodriver executable location

# Setup
* Clone the repo locally
* create a firefox profile to be used by the tool. Login to whatever sites 
ahead of time on this profile
* On the profile, set a specific marionette port
* In sites.toml, insert firefox profile, and screenshot paths, and the marionette port
you chose
* Download geckodriver.exe and place it in the root of the repo
* cargo run

# Configuration
You will probably want to change the product url if you're using this to buy
off amazon. If you're using this tool for some new automation, the supplied
sites.toml should give a good example of the capabilities of the bot.
